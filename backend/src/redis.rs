use crate::config::RedisRuntimeConfig;
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use shared::AuditLog;
use std::{sync::OnceLock, time::Duration};
use tower_sessions::{
    session::{Id, Record},
    session_store, MemoryStore, SessionStore,
};
use tower_sessions_redis_store::{
    fred::{
        clients::Pool,
        interfaces::{ClientLike, KeysInterface, StreamsInterface},
        prelude::{Builder, Config as FredRedisConfig, Expiration, ReconnectPolicy},
    },
    RedisStore,
};

#[derive(Debug, Clone)]
pub enum AppSessionStore {
    Memory(MemoryStore),
    Redis(RedisStore<Pool>),
}

#[async_trait]
impl SessionStore for AppSessionStore {
    async fn create(&self, record: &mut Record) -> session_store::Result<()> {
        match self {
            AppSessionStore::Memory(store) => store.create(record).await,
            AppSessionStore::Redis(store) => store.create(record).await,
        }
    }

    async fn save(&self, record: &Record) -> session_store::Result<()> {
        match self {
            AppSessionStore::Memory(store) => store.save(record).await,
            AppSessionStore::Redis(store) => store.save(record).await,
        }
    }

    async fn load(&self, session_id: &Id) -> session_store::Result<Option<Record>> {
        match self {
            AppSessionStore::Memory(store) => store.load(session_id).await,
            AppSessionStore::Redis(store) => store.load(session_id).await,
        }
    }

    async fn delete(&self, session_id: &Id) -> session_store::Result<()> {
        match self {
            AppSessionStore::Memory(store) => store.delete(session_id).await,
            AppSessionStore::Redis(store) => store.delete(session_id).await,
        }
    }
}

#[derive(Clone)]
struct RedisRuntime {
    pool: Pool,
    config: RedisRuntimeConfig,
}

#[derive(Debug, Clone, Serialize)]
pub struct RedisHealth {
    pub enabled: bool,
    pub connected: bool,
    pub session_store_enabled: bool,
    pub event_stream_enabled: bool,
    pub rate_limit_enabled: bool,
    pub cache_enabled: bool,
    pub event_stream_name: Option<String>,
    pub session_backend: String,
}

static MEMORY_STORE: OnceLock<MemoryStore> = OnceLock::new();
static REDIS_CONFIG: OnceLock<RedisRuntimeConfig> = OnceLock::new();
static REDIS_RUNTIME: OnceLock<RedisRuntime> = OnceLock::new();

fn memory_store() -> MemoryStore {
    MEMORY_STORE.get_or_init(MemoryStore::default).clone()
}

fn current_config() -> RedisRuntimeConfig {
    REDIS_CONFIG
        .get()
        .cloned()
        .unwrap_or_else(RedisRuntimeConfig::from_env)
}

pub fn redis_config() -> RedisRuntimeConfig {
    current_config()
}

pub async fn init_redis_runtime(config: RedisRuntimeConfig) -> Result<(), String> {
    let _ = REDIS_CONFIG.set(config.clone());

    if !config.enabled {
        tracing::info!("Redis integration disabled; using in-memory session storage");
        return Ok(());
    }

    if REDIS_RUNTIME.get().is_some() {
        return Ok(());
    }

    let redis_config = FredRedisConfig::from_url(&config.url)
        .map_err(|error| format!("Invalid REDIS_URL {}: {}", config.url, error))?;

    let pool = Builder::from_config(redis_config)
        .with_connection_config(|connection| {
            connection.connection_timeout = Duration::from_millis(config.connection_timeout_ms);
        })
        .set_policy(ReconnectPolicy::new_exponential(0, 100, 30_000, 2))
        .build_pool(config.pool_size)
        .map_err(|error| format!("Failed to create Redis pool: {}", error))?;

    pool.init()
        .await
        .map_err(|error| format!("Failed to connect to Redis: {}", error))?;

    REDIS_RUNTIME
        .set(RedisRuntime {
            pool,
            config: config.clone(),
        })
        .map_err(|_| "Redis runtime already initialized".to_string())?;

    tracing::info!(
        "Redis initialized: session_store={}, event_stream={}, stream={}",
        config.session_store_enabled,
        config.event_stream_enabled,
        config.event_stream_name
    );

    Ok(())
}

pub fn session_store() -> AppSessionStore {
    match REDIS_RUNTIME.get() {
        Some(runtime) if runtime.config.session_store_enabled => {
            AppSessionStore::Redis(RedisStore::new(runtime.pool.clone()))
        }
        _ => AppSessionStore::Memory(memory_store()),
    }
}

pub fn redis_health() -> RedisHealth {
    let config = current_config();
    let connected = REDIS_RUNTIME.get().is_some();
    let session_backend = if connected && config.session_store_enabled {
        "redis"
    } else {
        "memory"
    };

    RedisHealth {
        enabled: config.enabled,
        connected,
        session_store_enabled: config.enabled && config.session_store_enabled,
        event_stream_enabled: config.enabled && config.event_stream_enabled,
        rate_limit_enabled: config.enabled && config.rate_limit_enabled,
        cache_enabled: config.enabled && config.cache_enabled,
        event_stream_name: if config.enabled && config.event_stream_enabled {
            Some(config.event_stream_name)
        } else {
            None
        },
        session_backend: session_backend.to_string(),
    }
}

pub async fn cache_get_json<T>(key: &str) -> Result<Option<T>, String>
where
    T: DeserializeOwned,
{
    let Some(runtime) = REDIS_RUNTIME.get() else {
        return Ok(None);
    };
    if !runtime.config.cache_enabled {
        return Ok(None);
    }

    let payload: Option<String> = runtime
        .pool
        .get(key)
        .await
        .map_err(|error| format!("Failed to read Redis cache {}: {}", key, error))?;

    payload
        .map(|value| {
            serde_json::from_str::<T>(&value)
                .map_err(|error| format!("Failed to deserialize Redis cache {}: {}", key, error))
        })
        .transpose()
}

pub async fn cache_set_json<T>(key: &str, value: &T, ttl_secs: u64) -> Result<(), String>
where
    T: Serialize,
{
    let Some(runtime) = REDIS_RUNTIME.get() else {
        return Ok(());
    };
    if !runtime.config.cache_enabled || ttl_secs == 0 {
        return Ok(());
    }

    let payload = serde_json::to_string(value)
        .map_err(|error| format!("Failed to serialize Redis cache {}: {}", key, error))?;

    let _: () = runtime
        .pool
        .set(
            key,
            payload,
            Some(Expiration::EX(ttl_secs as i64)),
            None,
            false,
        )
        .await
        .map_err(|error| format!("Failed to write Redis cache {}: {}", key, error))?;
    Ok(())
}

pub async fn cache_delete(key: &str) -> Result<(), String> {
    let Some(runtime) = REDIS_RUNTIME.get() else {
        return Ok(());
    };
    if !runtime.config.cache_enabled {
        return Ok(());
    }

    let _: i64 = runtime
        .pool
        .del(key)
        .await
        .map_err(|error| format!("Failed to delete Redis cache {}: {}", key, error))?;
    Ok(())
}

fn sanitize_key_component(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' ) {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

pub async fn check_rate_limit(
    client_id: &str,
    requests_per_minute: u32,
    block_duration_seconds: u64,
) -> Result<Option<()>, String> {
    let Some(runtime) = REDIS_RUNTIME.get() else {
        return Ok(None);
    };
    if !runtime.config.rate_limit_enabled {
        return Ok(None);
    }

    let client_key = sanitize_key_component(client_id);
    let count_key = format!("{}:count:{}", runtime.config.rate_limit_prefix, client_key);
    let block_key = format!("{}:block:{}", runtime.config.rate_limit_prefix, client_key);

    let blocked: i64 = runtime
        .pool
        .exists(vec![block_key.clone()])
        .await
        .map_err(|error| format!("Failed to read Redis rate limit block key: {}", error))?;

    if blocked > 0 {
        let ttl: i64 = runtime
            .pool
            .ttl(block_key)
            .await
            .map_err(|error| format!("Failed to read Redis rate limit TTL: {}", error))?;
        let remaining = if ttl > 0 {
            ttl as u64
        } else {
            block_duration_seconds
        };
        return Err(format!(
            "Rate limit exceeded. Try again in {} seconds.",
            remaining
        ));
    }

    let count: i64 = runtime
        .pool
        .incr(count_key.clone())
        .await
        .map_err(|error| format!("Failed to increment Redis rate limit counter: {}", error))?;

    if count == 1 {
        let _: bool = runtime
            .pool
            .expire(count_key, 60, None)
            .await
            .map_err(|error| format!("Failed to expire Redis rate limit counter: {}", error))?;
    }

    if count > requests_per_minute as i64 {
        let _: () = runtime
            .pool
            .set(
                block_key,
                "1",
                Some(Expiration::EX(block_duration_seconds as i64)),
                None,
                false,
            )
            .await
            .map_err(|error| format!("Failed to set Redis rate limit block key: {}", error))?;

        return Err(format!(
            "Rate limit exceeded ({} requests/minute). Blocked for {} seconds.",
            requests_per_minute, block_duration_seconds
        ));
    }

    Ok(Some(()))
}

pub fn publish_audit_event(log_entry: AuditLog) {
    let Some(runtime) = REDIS_RUNTIME.get().cloned() else {
        return;
    };

    if !runtime.config.event_stream_enabled {
        return;
    }

    tokio::spawn(async move {
        let payload = match serde_json::to_string(&log_entry) {
            Ok(payload) => payload,
            Err(error) => {
                tracing::warn!("Failed to serialize audit event for Redis stream: {}", error);
                return;
            }
        };

        let fields = vec![
            ("kind".to_string(), "audit_log".to_string()),
            ("action".to_string(), log_entry.action.clone()),
            ("username".to_string(), log_entry.username.clone()),
            ("target".to_string(), log_entry.target.clone()),
            ("timestamp".to_string(), log_entry.timestamp.to_rfc3339()),
            ("payload".to_string(), payload),
        ];

        let result: Result<String, _> = runtime.pool.xadd(
            runtime.config.event_stream_name.clone(),
            false,
            ("MAXLEN", "~", runtime.config.event_stream_max_len),
            "*",
            fields,
        )
        .await;

        if let Err(error) = result {
            tracing::warn!("Failed to publish audit event to Redis stream: {}", error);
        }
    });
}
