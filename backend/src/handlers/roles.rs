use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;

use crate::{
    state::AppState,
    shared::{CustomRole, CreateRoleRequest, UpdateRoleRequest},
};

/// 获取所有角色(包括系统预定义角色和自定义角色)
pub async fn get_roles(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<serde_json::Value>>, StatusCode> {
    // 获取自定义角色
    let custom_roles = state.custom_roles.lock().await;

    // 构建角色列表,包括系统角色和自定义角色
    let mut roles = vec![
        serde_json::json!({
            "id": "sys_admin",
            "name": "系统管理员",
            "description": "系统内置角色,拥有所有权限",
            "is_system": true,
            "role": "SysAdmin"
        }),
        serde_json::json!({
            "id": "sec_admin",
            "name": "安全管理员",
            "description": "系统内置角色,负责资产、扫描和风险管理",
            "is_system": true,
            "role": "SecAdmin"
        }),
        serde_json::json!({
            "id": "auditor",
            "name": "审计员",
            "description": "系统内置角色,只能查看日志",
            "is_system": true,
            "role": "Auditor"
        }),
    ];

    // 添加自定义角色
    for role in custom_roles.iter() {
        roles.push(serde_json::json!({
            "id": role.id.map(|id| id.to_string()),
            "name": role.name,
            "description": role.description,
            "is_system": false,
            "permissions": role.permissions,
            "created_at": role.created_at,
            "updated_at": role.updated_at
        }));
    }

    Ok(Json(roles))
}

/// 获取单个角色
pub async fn get_role(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // 检查系统角色
    match id.as_str() {
        "sys_admin" => {
            return Ok(Json(serde_json::json!({
                "id": "sys_admin",
                "name": "系统管理员",
                "description": "系统内置角色,拥有所有权限",
                "is_system": true,
                "role": "SysAdmin"
            })));
        }
        "sec_admin" => {
            return Ok(Json(serde_json::json!({
                "id": "sec_admin",
                "name": "安全管理员",
                "description": "系统内置角色,负责资产、扫描和风险管理",
                "is_system": true,
                "role": "SecAdmin"
            })));
        }
        "auditor" => {
            return Ok(Json(serde_json::json!({
                "id": "auditor",
                "name": "审计员",
                "description": "系统内置角色,只能查看日志",
                "is_system": true,
                "role": "Auditor"
            })));
        }
        _ => {}
    }

    // 查找自定义角色
    let custom_roles = state.custom_roles.lock().await;
    if let Ok(role_id) = id.parse::<i32>() {
        if let Some(role) = custom_roles.iter().find(|r| r.id == Some(role_id)) {
            return Ok(Json(serde_json::json!({
                "id": role.id.map(|id| id.to_string()),
                "name": role.name,
                "description": role.description,
                "is_system": false,
                "permissions": role.permissions,
                "created_at": role.created_at,
                "updated_at": role.updated_at
            })));
        }
    }

    Err(StatusCode::NOT_FOUND)
}

/// 创建自定义角色
pub async fn create_role(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateRoleRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut custom_roles = state.custom_roles.lock().await;

    // 检查角色名称是否已存在
    if custom_roles.iter().any(|r| r.name == req.name) {
        return Ok(Json(serde_json::json!({
            "error": "角色名称已存在"
        })));
    }

    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let new_id = custom_roles.len() as i32 + 1; // 简单的ID生成

    let new_role = CustomRole {
        id: Some(new_id),
        name: req.name.clone(),
        description: req.description,
        permissions: req.permissions,
        created_at: Some(now.clone()),
        updated_at: Some(now),
    };

    custom_roles.push(new_role);

    Ok(Json(serde_json::json!({
        "id": new_id,
        "name": req.name,
        "message": "角色创建成功"
    })))
}

/// 更新自定义角色
pub async fn update_role(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateRoleRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // 不允许修改系统角色
    if matches!(id.as_str(), "sys_admin" | "sec_admin" | "auditor") {
        return Ok(Json(serde_json::json!({
            "error": "不能修改系统内置角色"
        })));
    }

    let role_id = id.parse::<i32>();
    if role_id.is_err() {
        return Err(StatusCode::NOT_FOUND);
    }

    let mut custom_roles = state.custom_roles.lock().await;
    let role = custom_roles.iter_mut().find(|r| r.id == role_id.ok());

    if let Some(role) = role {
        if let Some(name) = req.name {
            role.name = name;
        }
        if let Some(description) = req.description {
            role.description = Some(description);
        }
        if let Some(permissions) = req.permissions {
            role.permissions = permissions;
        }
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        role.updated_at = Some(now);

        Ok(Json(serde_json::json!({
            "message": "角色更新成功"
        })))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// 删除自定义角色
pub async fn delete_role(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // 不允许删除系统角色
    if matches!(id.as_str(), "sys_admin" | "sec_admin" | "auditor") {
        return Ok(Json(serde_json::json!({
            "error": "不能删除系统内置角色"
        })));
    }

    let role_id = id.parse::<i32>();
    if role_id.is_err() {
        return Err(StatusCode::NOT_FOUND);
    }

    let mut custom_roles = state.custom_roles.lock().await;
    if let Some(pos) = custom_roles.iter().position(|r| r.id == role_id.ok()) {
        custom_roles.remove(pos);

        Ok(Json(serde_json::json!({
            "message": "角色删除成功"
        })))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
