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
use rustset_framework_web::AppError;
use serde_json::{Value, json};
use std::collections::HashMap;

const TICKET: TableSpec = TableSpec {
    table: "infra_resource_ticket",
    seq: "infra_resource_ticket_seq",
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
        obj.entry("ticketStatus".to_string())
            .or_insert(Value::String("pending_approval".to_string()));
        let now = Utc::now().format("%Y-%m-%d %H:%M").to_string();
        obj.entry("createTime".to_string())
            .or_insert(Value::String(now.clone()));
        obj.entry("updateTime".to_string())
            .or_insert(Value::String(now));
        obj.entry("deliveryStatus".to_string())
            .or_insert(Value::String("未交付".to_string()));
        obj.entry("createdBy".to_string())
            .or_insert(Value::String(user.username.clone()));
        obj.entry("applicantName".to_string())
            .or_insert(Value::String(user.username));
    }
    table_create(&s.pool, TICKET, payload).await
}

async fn update(
    State(s): State<InfraState>,
    Json(p): Json<Value>,
) -> Result<Json<ApiResponse<()>>, AppError> {
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
    Json(payload): Json<Value>,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    let now = Utc::now().format("%Y-%m-%d %H:%M").to_string();
    let ticket = crate::table_get_value(&s.pool, TICKET, id).await?;
    let cs = ticket
        .get("ticketStatus")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if cs != "pending_provision" && cs != "approved" {
        return Err(AppError::bad_request("只能配置待配置状态的工单"));
    }
    sqlx::query("UPDATE infra_resource_ticket SET ticket_status='pending_delivery', provisioner=$2, provision_time=$3, provision_details=$4, update_time=now() WHERE id=$1 AND deleted=0")
        .bind(id).bind(&user.username).bind(&now).bind(opt_str_field(&payload, "details"))
        .execute(&s.pool).await.map_err(|_| AppError::internal("failed"))?;
    Ok(Json(ApiResponse::new(
        json!({"message": "工单配置完成", "id": id, "ticketStatus": "pending_delivery"}),
    )))
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
