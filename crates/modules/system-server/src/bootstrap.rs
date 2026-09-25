use anyhow::{Context, bail};
use tracing::{info, warn};

use crate::SystemState;

pub async fn initialize(state: &SystemState) -> anyhow::Result<()> {
    ensure_identity_uuids(state).await?;
    let administrators = sqlx::query_scalar::<_, i64>(
        "SELECT count(*)
         FROM system_users u
         JOIN system_user_role ur ON ur.user_id = u.id AND ur.deleted = 0
         JOIN system_role r ON r.id = ur.role_id AND r.deleted = 0
         WHERE u.deleted = 0 AND u.status = 0 AND r.status = 0
           AND r.code = 'super_admin'",
    )
    .fetch_one(&state.pool)
    .await
    .context("failed to verify RustSet administrator")?;

    if administrators == 0 {
        bootstrap_administrator(state).await?;
        let administrators = sqlx::query_scalar::<_, i64>(
            "SELECT count(*)
             FROM system_users u
             JOIN system_user_role ur ON ur.user_id = u.id AND ur.deleted = 0
             JOIN system_role r ON r.id = ur.role_id AND r.deleted = 0
             WHERE u.deleted = 0 AND u.status = 0 AND r.status = 0
               AND r.code = 'super_admin'",
        )
        .fetch_one(&state.pool)
        .await
        .context("failed to re-verify RustSet administrator")?;
        if administrators == 0 {
            bail!("RustSet has no enabled super administrator");
        }
    } else {
        info!(administrators, "RustSet administrators are ready");
    }

    initialize_admin_password(state).await?;
    if let Err(error) = reseal_legacy_secrets(state).await {
        warn!(
            ?error,
            "failed to re-seal legacy secrets; keeping existing values"
        );
    }

    Ok(())
}

/// Create the initial super administrator from `BOOTSTRAP_ADMIN_USERNAME` /
/// `BOOTSTRAP_ADMIN_PASSWORD` when the database has none at all (for example a
/// baseline without seeded admins). The account is created inside the lowest
/// enabled tenant and linked to the `super_admin` role. An advisory lock keeps
/// concurrent gateway starts from inserting duplicates; `system_users` has no
/// unique constraint on `username`.
async fn bootstrap_administrator(state: &SystemState) -> anyhow::Result<()> {
    let username = std::env::var("BOOTSTRAP_ADMIN_USERNAME")
        .unwrap_or_else(|_| "admin".into())
        .trim()
        .to_string();
    let password = std::env::var("BOOTSTRAP_ADMIN_PASSWORD")
        .unwrap_or_default()
        .trim()
        .to_string();
    if username.is_empty() || password.is_empty() {
        warn!(
            "no enabled super administrator exists and BOOTSTRAP_ADMIN_PASSWORD is not set; \
             cannot bootstrap an administrator"
        );
        return Ok(());
    }

    let mut tx = state
        .pool
        .begin()
        .await
        .context("failed to begin admin bootstrap transaction")?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtext('rustset-bootstrap-admin'))")
        .execute(&mut *tx)
        .await
        .context("failed to lock admin bootstrap")?;

    let already = sqlx::query_scalar::<_, i64>(
        "SELECT count(*)
         FROM system_users u
         JOIN system_user_role ur ON ur.user_id = u.id AND ur.deleted = 0
         JOIN system_role r ON r.id = ur.role_id AND r.deleted = 0
         WHERE u.deleted = 0 AND u.status = 0 AND r.status = 0
           AND r.code = 'super_admin'",
    )
    .fetch_one(&mut *tx)
    .await
    .context("failed to re-check administrators under lock")?;
    if already > 0 {
        tx.commit()
            .await
            .context("failed to commit bootstrap check")?;
        info!("another administrator appeared concurrently; skipping bootstrap");
        return Ok(());
    }

    let tenant_id: Option<i64> = sqlx::query_scalar(
        "SELECT tenant.id
         FROM system_tenant tenant
         LEFT JOIN system_users existing
           ON existing.tenant_id = tenant.id
          AND existing.username = $1
          AND existing.deleted = 0
         WHERE tenant.deleted = 0 AND tenant.status = 0
         ORDER BY (existing.id IS NOT NULL) DESC, tenant.id
         LIMIT 1",
    )
    .bind(&username)
    .fetch_optional(&mut *tx)
    .await
    .context("failed to locate bootstrap tenant")?;
    let Some(tenant_id) = tenant_id else {
        bail!("no enabled system_tenant row to bootstrap an administrator into");
    };

    // A database with no enabled super administrator is already locked out;
    // bootstrap is the recovery path, so accept a disabled super_admin role
    // and re-enable it, and create the role when it is missing entirely.
    let role: Option<(i64, i16)> =
        sqlx::query_as("SELECT id, status FROM system_role WHERE code = 'super_admin' AND tenant_id = $1 AND deleted = 0 ORDER BY id LIMIT 1")
            .bind(tenant_id)
            .fetch_optional(&mut *tx)
            .await
            .context("failed to locate super_admin role")?;
    let role_id = match role {
        Some((id, status)) => {
            if status != 0 {
                sqlx::query(
                    "UPDATE system_role SET status = 0, updater = 'bootstrap' WHERE id = $1",
                )
                .bind(id)
                .execute(&mut *tx)
                .await
                .context("failed to re-enable super_admin role")?;
                info!(
                    role_id = id,
                    "re-enabled disabled super_admin role during bootstrap"
                );
            }
            id
        }
        None => {
            let id: i64 = sqlx::query_scalar(
                "INSERT INTO system_role (id, name, code, sort, data_scope, status, type, tenant_id, creator, updater)
                 VALUES (nextval('system_role_seq'), '超级管理员', 'super_admin', 1, 1, 0, 1, $1, 'bootstrap', 'bootstrap')
                 RETURNING id",
            )
            .bind(tenant_id)
            .fetch_one(&mut *tx)
            .await
            .context("failed to create super_admin role")?;
            info!(
                role_id = id,
                "created missing super_admin role during bootstrap"
            );
            id
        }
    };

    let hash = state
        .passwords
        .hash_secret(&password)
        .context("failed to hash bootstrap admin password")?;

    let inserted: Option<i64> = sqlx::query_scalar(
        "INSERT INTO system_users (id, username, password, nickname, status, tenant_id, creator, updater)
         SELECT nextval('system_users_seq'), $1, $2, $3, 0, $4, 'bootstrap', 'bootstrap'
         WHERE NOT EXISTS (
             SELECT 1 FROM system_users WHERE username = $1 AND deleted = 0
         )
         RETURNING id",
    )
    .bind(&username)
    .bind(&hash)
    .bind("Administrator")
    .bind(tenant_id)
    .fetch_optional(&mut *tx)
    .await
    .context("failed to insert bootstrap administrator")?;
    if let Some(id) = inserted {
        sqlx::query("UPDATE system_users SET identity_uuid = $2 WHERE id = $1")
            .bind(id)
            .bind(crate::infrastructure::identity_uuid_for(id))
            .execute(&mut *tx)
            .await
            .context("failed to assign bootstrap identity uuid")?;
    }
    let user_id = match inserted {
        Some(id) => id,
        None => sqlx::query_scalar::<_, i64>(
            "SELECT id FROM system_users WHERE username = $1 AND deleted = 0",
        )
        .bind(&username)
        .fetch_one(&mut *tx)
        .await
        .context("failed to look up existing bootstrap username")?,
    };

    sqlx::query(
        "INSERT INTO system_user_role (id, user_id, role_id, tenant_id, creator, updater)
         SELECT nextval('system_user_role_seq'), $1, $2, $3, 'bootstrap', 'bootstrap'
         WHERE NOT EXISTS (
             SELECT 1 FROM system_user_role
             WHERE user_id = $1 AND role_id = $2 AND deleted = 0
         )",
    )
    .bind(user_id)
    .bind(role_id)
    .bind(tenant_id)
    .execute(&mut *tx)
    .await
    .context("failed to assign super_admin role")?;

    tx.commit()
        .await
        .context("failed to commit admin bootstrap")?;
    info!(
        username,
        user_id, tenant_id, "bootstrapped initial administrator"
    );
    Ok(())
}

/// Set the initial password from `BOOTSTRAP_ADMIN_PASSWORD` while the account's
/// password column is still empty. The update is conditional on
/// `password = ''` so a password changed through the UI or by a concurrent
/// start is never overwritten, and the flow is idempotent across restarts.
async fn initialize_admin_password(state: &SystemState) -> anyhow::Result<()> {
    let username = std::env::var("BOOTSTRAP_ADMIN_USERNAME")
        .unwrap_or_else(|_| "admin".into())
        .trim()
        .to_string();
    if username.is_empty() {
        return Ok(());
    }

    let password = std::env::var("BOOTSTRAP_ADMIN_PASSWORD")
        .unwrap_or_default()
        .trim()
        .to_string();
    if password.is_empty() {
        // An empty password can only appear on restored snapshots; the normal
        // baseline seeds a real hash, so do not spam the log on every start.
        return Ok(());
    }

    let hash = state
        .passwords
        .hash_secret(&password)
        .context("failed to hash bootstrap admin password")?;

    let updated = sqlx::query(
        "UPDATE system_users SET password = $1, updater = 'bootstrap'
         WHERE username = $2 AND deleted = 0 AND status = 0 AND password = ''",
    )
    .bind(&hash)
    .bind(&username)
    .execute(&state.pool)
    .await
    .context("failed to persist bootstrap admin password")?
    .rows_affected();

    if updated == 1 {
        info!(username, "bootstrap admin password initialized");
    }
    Ok(())
}

/// Backfill `system_users.identity_uuid` (SM3-derived, 国密) for any user the
/// migrations could not cover — the derivation is computed in Rust, so seed
/// literals in migrations only cover the baseline users. Idempotent.
pub async fn ensure_identity_uuids(state: &SystemState) -> anyhow::Result<()> {
    let missing: Vec<i64> =
        sqlx::query_scalar("SELECT id FROM system_users WHERE identity_uuid IS NULL ORDER BY id")
            .fetch_all(&state.pool)
            .await
            .context("failed to list users without identity uuid")?;
    if missing.is_empty() {
        return Ok(());
    }
    let mut tx = state
        .pool
        .begin()
        .await
        .context("failed to begin identity uuid backfill")?;
    for id in missing {
        sqlx::query("UPDATE system_users SET identity_uuid = $2 WHERE id = $1")
            .bind(id)
            .bind(crate::infrastructure::identity_uuid_for(id))
            .execute(&mut *tx)
            .await
            .context("failed to backfill identity uuid")?;
    }
    tx.commit()
        .await
        .context("failed to commit identity uuid backfill")?;
    info!("backfilled SM3 identity uuids for users lacking one");
    Ok(())
}

fn sealing_secret() -> String {
    std::env::var("SECRET_ENCRYPTION_KEY")
        .or_else(|_| std::env::var("JWT_SECRET"))
        .unwrap_or_else(|_| "rustset-local-secret".to_owned())
}

/// One-shot startup pass: convert surviving legacy XOR `enc:v1:` sealed values
/// to SM4-CBC. Runs on every start and does nothing once no `enc:v1:` rows
/// remain. Values that cannot be decoded (e.g. sealed under a different key)
/// are left untouched and counted.
async fn reseal_legacy_secrets(state: &SystemState) -> anyhow::Result<()> {
    use rustset_framework_gm as gm;

    let secret = sealing_secret();
    let reseal = |sealed: &str| -> Option<String> {
        let plain = gm::legacy_v1_open(sealed, &secret)?;
        gm::sm4_seal(&plain, &secret).ok()
    };
    let mut converted = 0usize;
    let mut skipped = 0usize;

    let mail_rows: Vec<(i64, String)> = sqlx::query_as(
        "SELECT id, password FROM system_mail_account WHERE password LIKE 'enc:v1:%'",
    )
    .fetch_all(&state.pool)
    .await
    .context("failed to list legacy mail secrets")?;
    for (id, sealed) in mail_rows {
        match reseal(&sealed) {
            Some(resealed) => {
                sqlx::query("UPDATE system_mail_account SET password = $2 WHERE id = $1")
                    .bind(id)
                    .bind(resealed)
                    .execute(&state.pool)
                    .await
                    .context("failed to re-seal mail secret")?;
                converted += 1;
            }
            None => skipped += 1,
        }
    }

    let sms_rows: Vec<(i64, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT id, api_key, api_secret FROM system_sms_channel
         WHERE api_key LIKE 'enc:v1:%' OR api_secret LIKE 'enc:v1:%'",
    )
    .fetch_all(&state.pool)
    .await
    .context("failed to list legacy sms secrets")?;
    for (id, api_key, api_secret) in sms_rows {
        let key = match api_key.as_deref() {
            Some(value) if value.starts_with(gm::LEGACY_SEALED_PREFIX) => reseal(value),
            other => other.map(str::to_owned),
        };
        let api_key = match key {
            Some(value) => value,
            None => {
                skipped += 1;
                continue;
            }
        };
        let api_secret = match api_secret.as_deref() {
            Some(value) if value.starts_with(gm::LEGACY_SEALED_PREFIX) => reseal(value),
            other => other.map(str::to_owned),
        }
        .unwrap_or_default();
        sqlx::query("UPDATE system_sms_channel SET api_key = $2, api_secret = $3 WHERE id = $1")
            .bind(id)
            .bind(api_key)
            .bind(api_secret)
            .execute(&state.pool)
            .await
            .context("failed to re-seal sms secret")?;
        converted += 1;
    }

    let datasource_rows: Vec<(i64, String)> = sqlx::query_as(
        "SELECT id, password FROM infra_data_source_config WHERE password LIKE 'enc:v1:%'",
    )
    .fetch_all(&state.pool)
    .await
    .context("failed to list legacy datasource secrets")?;
    for (id, sealed) in datasource_rows {
        match reseal(&sealed) {
            Some(resealed) => {
                sqlx::query("UPDATE infra_data_source_config SET password = $2 WHERE id = $1")
                    .bind(id)
                    .bind(resealed)
                    .execute(&state.pool)
                    .await
                    .context("failed to re-seal datasource secret")?;
                converted += 1;
            }
            None => skipped += 1,
        }
    }

    let cloud_rows: Vec<(i64, String, String)> = sqlx::query_as(
        "SELECT id, access_key_id, access_key_secret FROM infra_cloud_provider_config
         WHERE access_key_id LIKE 'enc:v1:%' OR access_key_secret LIKE 'enc:v1:%'",
    )
    .fetch_all(&state.pool)
    .await
    .context("failed to list legacy cloud credentials")?;
    for (id, key_id, key_secret) in cloud_rows {
        let key = if key_id.starts_with(gm::LEGACY_SEALED_PREFIX) {
            reseal(&key_id)
        } else {
            Some(key_id)
        };
        let secret_value = if key_secret.starts_with(gm::LEGACY_SEALED_PREFIX) {
            reseal(&key_secret)
        } else {
            Some(key_secret)
        };
        match (key, secret_value) {
            (Some(key), Some(secret_value)) => {
                sqlx::query(
                    "UPDATE infra_cloud_provider_config
                     SET access_key_id = $2, access_key_secret = $3 WHERE id = $1",
                )
                .bind(id)
                .bind(key)
                .bind(secret_value)
                .execute(&state.pool)
                .await
                .context("failed to re-seal cloud credentials")?;
                converted += 1;
            }
            _ => skipped += 1,
        }
    }

    if converted > 0 || skipped > 0 {
        info!(converted, skipped, "re-sealed legacy secrets to SM4");
    }
    Ok(())
}
