use crate::{
    InfraState, QueryParams, TableSpec, id_param, ids_param, opt_str_field, soft_delete,
    table_create, table_get, table_page, table_update,
};
use axum::{
    Json, Router,
    extract::{Query, State},
    routing::{delete, get, post, put},
};
use chrono::Utc;
use rustset_framework_common::ApiResponse;
use rustset_framework_security::CurrentUser;
use rustset_framework_tofu::{
    CloudTarget, DEMO_PROVIDER, TofuExecutor, credential_env, provider_source, render_main_tf,
    render_tfvars,
};
use rustset_framework_web::AppError;
use serde_json::{Map, Value, json};
use sqlx::Row;
use std::collections::HashMap;
use tracing::warn;

const TICKET: TableSpec = TableSpec {
    table: "infra_resource_ticket",
    seq: "infra_resource_ticket_seq",
};

const APPROVAL_RULE: TableSpec = TableSpec {
    table: "infra_approval_rule",
    seq: "infra_approval_rule_seq",
};

pub fn routes() -> Router<InfraState> {
    Router::new()
        .route("/infra/resource-ticket/page", get(page))
        .route("/infra/resource-ticket/get", get(get_one))
        .route("/infra/resource-ticket/create", post(create))
        .route("/infra/resource-ticket/update", put(update))
        .route("/infra/resource-ticket/delete", delete(delete_one))
        .route("/infra/resource-ticket/delete-list", delete(delete_list))
        .route("/infra/resource-ticket/{id}/approve", post(approve))
        .route("/infra/resource-ticket/{id}/provision", post(provision))
        .route("/infra/resource-ticket/{id}/deliver", post(deliver))
        .route("/infra/approval-rule/page", get(rule_page))
        .route("/infra/approval-rule/list", get(rule_list))
        .route("/infra/approval-rule/create", post(rule_create))
        .route("/infra/approval-rule/update", put(rule_update))
        .route("/infra/approval-rule/delete", delete(rule_delete))
}

async fn page(
    State(s): State<InfraState>,
    Query(p): Query<QueryParams>,
) -> Result<Json<ApiResponse<crate::Page<Value>>>, AppError> {
    table_page(&s.pool, TICKET, p).await
}
async fn get_one(
    State(s): State<InfraState>,
    Query(p): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    table_get(&s.pool, TICKET, id_param(&p)?).await
}

async fn create(
    State(s): State<InfraState>,
    user: CurrentUser,
    Json(mut payload): Json<Value>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    if let Some(obj) = payload.as_object_mut() {
        if let Some(Value::Bool(enabled)) = obj.get("hasSecurityProduct").cloned() {
            obj.insert(
                "hasSecurityProduct".to_string(),
                Value::Number((enabled as i32).into()),
            );
        }
        obj.entry("ticketStatus".to_string())
            .or_insert(Value::String("pending_approval".to_string()));
        let now = Utc::now().format("%Y-%m-%d %H:%M").to_string();
        obj.entry("createTime".to_string())
            .or_insert(Value::String(now.clone()));
        obj.entry("updateTime".to_string())
            .or_insert(Value::String(now));
        obj.entry("deliveryStatus".to_string())
            .or_insert(Value::String("未交付".to_string()));
        obj.entry("ticketType".to_string())
            .or_insert(Value::String("create".to_string()));
        obj.entry("riskLevel".to_string())
            .or_insert(Value::String("normal".to_string()));
        obj.entry("approvalStage".to_string())
            .or_insert(Value::Number(1.into()));
        obj.entry("approvalTotal".to_string())
            .or_insert(Value::Number(1.into()));
        obj.entry("currentApprovalRole".to_string())
            .or_insert(Value::String("资源管理员".to_string()));
        obj.entry("createdBy".to_string())
            .or_insert(Value::String(user.username.clone()));
        obj.entry("applicantName".to_string())
            .or_insert(Value::String(user.username));
    }
    let Json(created) = table_create(&s.pool, TICKET, payload).await?;
    let Ok(id) = created.data.parse::<i64>() else {
        return Ok(Json(created));
    };
    if let Some(rule) = match_approval_rule(&s, id).await? {
        apply_auto_approval(&s, id, &rule).await?;
    }
    Ok(Json(created))
}

/// The first enabled rule whose resource type matches (empty = any) and
/// whose thresholds all cover the ticket.
async fn match_approval_rule(s: &InfraState, ticket_id: i64) -> Result<Option<Value>, AppError> {
    let ticket = crate::table_get_value(&s.pool, TICKET, ticket_id).await?;
    let rows = sqlx::query(
        "SELECT id, name, resource_type, max_cpu_cores, max_memory_gb, max_resource_count, auto_provision
         FROM infra_approval_rule WHERE deleted = 0 AND status = 0 ORDER BY id",
    )
    .fetch_all(&s.pool)
    .await
    .map_err(|_| AppError::internal("failed to read approval rules"))?;
    for row in rows {
        let resource_type: String = row.get("resource_type");
        let ticket_type = ticket
            .get("resourceType")
            .and_then(Value::as_str)
            .unwrap_or("");
        if !resource_type.is_empty() && resource_type != ticket_type {
            continue;
        }
        let within = |ticket_key: &str, column: &str| -> bool {
            let limit: Option<i32> = row.get(column);
            match limit {
                None => true,
                Some(limit) => ticket
                    .get(ticket_key)
                    .and_then(Value::as_i64)
                    .map(|value| value <= limit as i64)
                    .unwrap_or(true),
            }
        };
        if !within("cpuCores", "max_cpu_cores")
            || !within("memoryGb", "max_memory_gb")
            || !within("resourceCount", "max_resource_count")
        {
            continue;
        }
        let auto_provision: bool = row.get("auto_provision");
        return Ok(Some(json!({
            "id": row.get::<i64, _>("id"),
            "name": row.get::<String, _>("name"),
            "autoProvision": auto_provision,
        })));
    }
    Ok(None)
}

async fn apply_auto_approval(s: &InfraState, id: i64, rule: &Value) -> Result<(), AppError> {
    let now = Utc::now().format("%Y-%m-%d %H:%M").to_string();
    let rule_name = rule.get("name").and_then(Value::as_str).unwrap_or("rule");
    sqlx::query(
        "UPDATE infra_resource_ticket
         SET ticket_status='pending_provision', approver=$2, approve_time=$3,
             approve_comment=$4, update_time=now()
         WHERE id=$1 AND deleted=0",
    )
    .bind(id)
    .bind(format!("auto:{rule_name}"))
    .bind(&now)
    .bind(format!("命中自动审批规则 {rule_name}"))
    .execute(&s.pool)
    .await
    .map_err(|_| AppError::internal("failed to auto-approve"))?;
    if rule
        .get("autoProvision")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        // A failed auto-provision must not fail ticket creation; the
        // ticket stays pending_provision with the failure recorded.
        if let Err(error) = run_provision(s, id, &format!("auto:{rule_name}")).await {
            warn!(error = ?error, ticket = id, "auto-provision failed");
        }
    }
    Ok(())
}

async fn update(
    State(s): State<InfraState>,
    Json(mut p): Json<Value>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    if let Some(object) = p.as_object_mut()
        && let Some(Value::Bool(enabled)) = object.get("hasSecurityProduct").cloned()
    {
        object.insert(
            "hasSecurityProduct".to_string(),
            Value::Number((enabled as i32).into()),
        );
    }
    table_update(&s.pool, TICKET, p).await
}
async fn delete_one(
    State(s): State<InfraState>,
    Query(p): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    soft_delete(&s.pool, TICKET.table, id_param(&p)?).await
}
async fn delete_list(
    State(s): State<InfraState>,
    Query(p): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    crate::soft_delete_list(&s.pool, TICKET.table, ids_param(&p)).await
}

async fn approve(
    State(s): State<InfraState>,
    axum::extract::Path(id): axum::extract::Path<i64>,
    user: CurrentUser,
    Json(payload): Json<Value>,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    let approved = crate::bool_field(&payload, "approved", false);
    let now = Utc::now().format("%Y-%m-%d %H:%M").to_string();
    let new_status = if approved {
        "pending_provision"
    } else {
        "rejected"
    };
    let ticket = crate::table_get_value(&s.pool, TICKET, id).await?;
    if ticket.get("ticketStatus").and_then(|v| v.as_str()) != Some("pending_approval") {
        return Err(AppError::bad_request("只能审批待审批状态的工单"));
    }
    sqlx::query("UPDATE infra_resource_ticket SET ticket_status=$2, approver=$3, approve_time=$4, approve_comment=$5, update_time=now() WHERE id=$1 AND deleted=0")
        .bind(id).bind(new_status).bind(&user.username).bind(&now).bind(opt_str_field(&payload, "comment"))
        .execute(&s.pool).await.map_err(|_| AppError::internal("failed"))?;
    Ok(Json(ApiResponse::new(
        json!({"message": if approved { "工单审批通过" } else { "工单已拒绝" }, "id": id, "ticketStatus": new_status}),
    )))
}

async fn provision(
    State(s): State<InfraState>,
    axum::extract::Path(id): axum::extract::Path<i64>,
    user: CurrentUser,
    Json(_payload): Json<Value>,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    run_provision(&s, id, &user.username).await
}

/// Real provisioning: resolve cloud credentials, render the tofu workspace,
/// run init/apply, record logs/outputs on the ticket, write the result into
/// CMDB, and advance the ticket on success.
async fn run_provision(
    s: &InfraState,
    id: i64,
    operator: &str,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    let now = Utc::now().format("%Y-%m-%d %H:%M").to_string();
    let ticket = crate::table_get_value(&s.pool, TICKET, id).await?;
    let status = ticket
        .get("ticketStatus")
        .and_then(Value::as_str)
        .unwrap_or("");
    if status != "pending_provision" && status != "approved" {
        return Err(AppError::bad_request("只能配置待配置状态的工单"));
    }

    let cloud_category = ticket
        .get("cloudCategory")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEMO_PROVIDER)
        .to_string();
    let Some(source) = provider_source(&cloud_category) else {
        let reason = format!("cloud category {cloud_category:?} has no curated tofu template");
        record_failure(s, id, &reason).await?;
        return Err(AppError::bad_request(reason));
    };
    let region = ticket
        .get("cloudRegion")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

    // Credentials come from the cloud provider config bound to the
    // ticket's platform; the demo category needs none.
    let (access_key_id, access_key_secret) = if cloud_category == DEMO_PROVIDER {
        (String::new(), String::new())
    } else {
        match load_platform_credentials(s, &ticket).await? {
            Some(credentials) => credentials,
            None => {
                let reason = "no enabled cloud provider config for this ticket's platform; configure one first";
                record_failure(s, id, reason).await?;
                return Err(AppError::bad_request(reason));
            }
        }
    };
    let target = CloudTarget {
        cloud_category: cloud_category.clone(),
        provider_source: source.to_string(),
        region: region.clone(),
        access_key_id,
        access_key_secret,
    };

    let mut spec = Map::new();
    for (ticket_key, spec_key) in [
        ("ecsName", "ecs_name"),
        ("ecsType", "ecs_type"),
        ("ecsOs", "ecs_os"),
        ("cloudRegion", "cloud_region"),
        ("resourceCount", "resource_count"),
        ("cpuCores", "cpu_cores"),
        ("memoryGb", "memory_gb"),
        ("applicationName", "application_name"),
    ] {
        if let Some(value) = ticket.get(ticket_key).filter(|v| !v.is_null()) {
            spec.insert(spec_key.to_string(), value.clone());
        }
    }
    spec.insert("ticket_id".to_string(), json!(id.to_string()));
    let spec = Value::Object(spec);

    let executor = TofuExecutor::from_env();
    if !executor.available().await {
        let reason = "OpenTofu binary not found: install it (opentofu.org) and/or set TOFU_BINARY";
        record_failure(s, id, reason).await?;
        return Err(AppError::bad_request(reason));
    }

    let workspace_name = format!("ticket-{id}");
    let workspace = executor
        .ensure_workspace(&workspace_name)
        .map_err(AppError::bad_request)?;
    let main_tf = render_main_tf(&target, &spec).map_err(AppError::bad_request)?;
    TofuExecutor::write_file(&workspace, "main.tf", &main_tf).map_err(AppError::bad_request)?;
    TofuExecutor::write_file(
        &workspace,
        "terraform.tfvars.json",
        &render_tfvars(&spec, &region),
    )
    .map_err(AppError::bad_request)?;

    let mut log = String::new();
    let init = executor.init(&workspace).await;
    log.push_str(&init.combined_log());
    if !init.success {
        record_failure(s, id, &log).await?;
        return Err(AppError::bad_request(format!(
            "tofu init failed: {}",
            tail(&init.stderr)
        )));
    }

    let env = credential_env(&target);
    let apply = executor.apply(&env, &workspace).await;
    log.push_str(&apply.combined_log());
    if !apply.success {
        record_failure(s, id, &log).await?;
        return Err(AppError::bad_request(format!(
            "tofu apply failed: {}",
            tail(&apply.stderr)
        )));
    }

    let outputs = executor.outputs(&workspace).await;
    let tf_outputs = Value::Object(outputs.clone().into_iter().collect());
    sqlx::query(
        "UPDATE infra_resource_ticket
         SET ticket_status='pending_delivery', provisioner=$2, provision_time=$3,
             provision_details='OpenTofu apply 完成', apply_status='applied',
             apply_log=$4, tf_outputs=$5, tofu_workspace=$6, update_time=now()
         WHERE id=$1 AND deleted=0",
    )
    .bind(id)
    .bind(operator)
    .bind(&now)
    .bind(truncate_log(&log))
    .bind(&tf_outputs)
    .bind(&workspace_name)
    .execute(&s.pool)
    .await
    .map_err(|_| AppError::internal("failed"))?;

    upsert_cmdb_instance(s, id, &ticket, &cloud_category, &tf_outputs).await;

    Ok(Json(ApiResponse::new(
        json!({"message": "OpenTofu 配置完成", "id": id, "ticketStatus": "pending_delivery", "applyStatus": "applied", "outputs": tf_outputs}),
    )))
}

async fn record_failure(s: &InfraState, id: i64, log: &str) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE infra_resource_ticket SET apply_status='failed', apply_log=$2, update_time=now()
         WHERE id=$1 AND deleted=0",
    )
    .bind(id)
    .bind(truncate_log(log))
    .execute(&s.pool)
    .await
    .map_err(|_| AppError::internal("failed to record provision failure"))?;
    Ok(())
}

/// One enabled credential row for the ticket's cloud platform.
async fn load_platform_credentials(
    s: &InfraState,
    ticket: &Value,
) -> Result<Option<(String, String)>, AppError> {
    let platform_id = ticket
        .get("cloudPlatformId")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    let row = sqlx::query(
        "SELECT access_key_id, access_key_secret FROM infra_cloud_provider_config
         WHERE platform_id=$1 AND deleted=0 AND status='enabled'
         ORDER BY id LIMIT 1",
    )
    .bind(platform_id)
    .fetch_optional(&s.pool)
    .await
    .map_err(|_| AppError::internal("failed to read cloud credentials"))?;
    Ok(row.map(|row| {
        // Cloud credentials are sealed at rest (SM4, formerly XOR); the tofu
        // executor receives them via environment variables only.
        (
            crate::open_secret(&row.get::<String, _>("access_key_id")),
            crate::open_secret(&row.get::<String, _>("access_key_secret")),
        )
    }))
}

/// Upsert the provisioned resource into CMDB under model code
/// `cloud_<resource_type>` (created implicitly, key `ticket_id`).
async fn upsert_cmdb_instance(
    s: &InfraState,
    ticket_id: i64,
    ticket: &Value,
    cloud_category: &str,
    tf_outputs: &Value,
) {
    let resource_type = ticket
        .get("resourceType")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .unwrap_or("ecs");
    let model_code = format!("cloud_{resource_type}");
    let result = sqlx::query(
        "INSERT INTO cmdb_model (name, code, unique_key, creator, updater)
         VALUES ($1, $2, 'ticket_id', 'tofu', 'tofu')
         ON CONFLICT DO NOTHING RETURNING id",
    )
    .bind(format!("{resource_type}（云资源）"))
    .bind(&model_code)
    .fetch_optional(&s.pool)
    .await;
    let model_id: i64 = match result {
        Ok(Some(row)) => row.get("id"),
        Ok(None) => match sqlx::query_scalar::<_, i64>(
            "SELECT id FROM cmdb_model WHERE code=$1 AND deleted=0",
        )
        .bind(&model_code)
        .fetch_one(&s.pool)
        .await
        {
            Ok(id) => id,
            Err(_) => return,
        },
        Err(_) => return,
    };

    let mut attributes = Map::new();
    attributes.insert("ticket_id".to_string(), json!(ticket_id.to_string()));
    for (ticket_key, attr_key) in [
        ("ecsName", "ecs_name"),
        ("ecsType", "ecs_type"),
        ("cloudPlatformName", "cloud_platform"),
        ("cloudRegion", "cloud_region"),
        ("cpuCores", "cpu_cores"),
        ("memoryGb", "memory_gb"),
        ("resourceCount", "resource_count"),
    ] {
        if let Some(value) = ticket.get(ticket_key).filter(|v| !v.is_null()) {
            attributes.insert(attr_key.to_string(), value.clone());
        }
    }
    attributes.insert("cloud_category".to_string(), json!(cloud_category));
    attributes.insert("tf_outputs".to_string(), tf_outputs.clone());
    let payload = Value::Object(attributes);

    let updated = sqlx::query(
        "UPDATE cmdb_instance SET attributes = $3, updater='tofu', update_time=now()
         WHERE model_id=$1 AND deleted=0 AND attributes->>'ticket_id'=$2",
    )
    .bind(model_id)
    .bind(ticket_id.to_string())
    .bind(&payload)
    .execute(&s.pool)
    .await
    .map(|result| result.rows_affected())
    .unwrap_or(0);
    if updated == 0 {
        let _ = sqlx::query(
            "INSERT INTO cmdb_instance (model_id, attributes, creator, updater)
             VALUES ($1, $2, 'tofu', 'tofu')",
        )
        .bind(model_id)
        .bind(&payload)
        .execute(&s.pool)
        .await;
    }
}

fn truncate_log(log: &str) -> String {
    const LIMIT: usize = 60 * 1024;
    if log.len() <= LIMIT {
        return log.to_string();
    }
    format!("...\n{}", &log[log.len() - LIMIT..])
}

fn tail(text: &str) -> String {
    text.lines()
        .rev()
        .take(5)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("\n")
}

async fn rule_page(
    State(s): State<InfraState>,
    Query(p): Query<QueryParams>,
) -> Result<Json<ApiResponse<crate::Page<Value>>>, AppError> {
    table_page(&s.pool, APPROVAL_RULE, p).await
}
async fn rule_list(State(s): State<InfraState>) -> Result<Json<ApiResponse<Vec<Value>>>, AppError> {
    crate::table_list(&s.pool, APPROVAL_RULE).await
}
async fn rule_create(
    State(s): State<InfraState>,
    user: CurrentUser,
    Json(mut payload): Json<Value>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    if let Some(obj) = payload.as_object_mut() {
        obj.entry("createdBy".to_string())
            .or_insert(Value::String(user.username.clone()));
    }
    table_create(&s.pool, APPROVAL_RULE, payload).await
}
async fn rule_update(
    State(s): State<InfraState>,
    user: CurrentUser,
    Json(payload): Json<Value>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    table_update(&s.pool, APPROVAL_RULE, payload).await
}
async fn rule_delete(
    State(s): State<InfraState>,
    user: CurrentUser,
    Query(p): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    soft_delete(&s.pool, APPROVAL_RULE.table, id_param(&p)?).await
}

async fn deliver(
    State(s): State<InfraState>,
    axum::extract::Path(id): axum::extract::Path<i64>,
    user: CurrentUser,
    Json(payload): Json<Value>,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    let now = Utc::now().format("%Y-%m-%d %H:%M").to_string();
    let ticket = crate::table_get_value(&s.pool, TICKET, id).await?;
    if ticket.get("ticketStatus").and_then(|v| v.as_str()) != Some("pending_delivery") {
        return Err(AppError::bad_request("只能交付待交付状态的工单"));
    }
    sqlx::query("UPDATE infra_resource_ticket SET ticket_status='delivered', delivery_status='已交付', deliverer=$2, deliver_time=$3, deliver_comment=$4, update_time=now() WHERE id=$1 AND deleted=0")
        .bind(id).bind(&user.username).bind(&now).bind(opt_str_field(&payload, "comment"))
        .execute(&s.pool).await.map_err(|_| AppError::internal("failed"))?;
    Ok(Json(ApiResponse::new(
        json!({"message": "工单交付完成", "id": id, "ticketStatus": "delivered"}),
    )))
}
