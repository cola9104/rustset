use axum::{Json, Router, middleware::from_fn_with_state, routing::get};
use rustset_framework_common::{ApiResponse, ServiceConfig, health_route, init_tracing, serve};
use rustset_framework_database::{DatabaseConfig, connect, migrate};
use rustset_framework_redis::{RateLimitConfig, RateLimitState, RedisClient, RedisConfig};
use rustset_framework_security::{SecurityConfig, TokenService};
use rustset_framework_web::{AppError, WebConfig, apply_web_layers};
use serde::Serialize;
use tracing::warn;

mod audit;
mod openapi;

const SERVICE_NAME: &str = "gateway";

#[derive(Debug, Serialize)]
struct GatewayIndex {
    service: &'static str,
    modules: [&'static str; 4],
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing(SERVICE_NAME);

    let database = connect(&DatabaseConfig::from_env()?).await?;
    migrate(&database).await?;
    let redis = connect_redis().await;
    let tokens = TokenService::new(SecurityConfig::from_env()?);
    let system_state = rustset_system_server::SystemState::with_cache(
        database.clone(),
        tokens.clone(),
        redis.clone(),
    );
    let infra_state = rustset_infra_server::InfraState::new(database.clone());
    let ai_state = rustset_ai_server::AiState::new(database.clone(), tokens);
    let cmdb_state = rustset_cmdb_server::CmdbState {
        pool: database.clone(),
    };
    system_state.bootstrap().await?;
    let database_auth = system_state.database_auth_state();

    let mut app = Router::new()
        .route("/", get(index))
        .route("/openapi.json", get(openapi::document))
        .merge(rustset_system_server::routes(system_state))
        .merge(rustset_infra_server::routes(infra_state))
        .merge(rustset_ai_server::routes(ai_state))
        .merge(rustset_cmdb_server::routes(cmdb_state))
        .merge(health_route(SERVICE_NAME))
        .fallback(not_found)
        .layer(from_fn_with_state(
            audit::AuditState::new(database),
            audit::record,
        ))
        .layer(from_fn_with_state(
            database_auth,
            rustset_system_server::authenticate_from_database,
        ));

    if let Some(redis) = redis {
        app = app.layer(from_fn_with_state(
            RateLimitState::new(redis, RateLimitConfig::from_env()),
            rustset_framework_redis::rate_limit,
        ));
    }

    let app = apply_web_layers(app, WebConfig::from_env());

    serve(ServiceConfig::from_env(SERVICE_NAME, 8080), app).await
}

async fn connect_redis() -> Option<RedisClient> {
    let config = RedisConfig::from_env()?;
    match RedisClient::connect(&config).await {
        Ok(client) => Some(client),
        Err(error) => {
            warn!(%error, "redis is configured but unavailable; cache and rate limit disabled");
            None
        }
    }
}

async fn not_found() -> AppError {
    AppError::not_found("route not found")
}

async fn index() -> Json<ApiResponse<GatewayIndex>> {
    Json(ApiResponse::new(GatewayIndex {
        service: SERVICE_NAME,
        modules: ["system", "infra", "ai", "cmdb"],
    }))
}
