use axum::Json;
use serde_json::{Value, json};

pub async fn document() -> Json<Value> {
    Json(json!({
        "openapi": "3.1.0",
        "info": {
            "title": "RustSet Gateway API",
            "description": "RustSet 资产、CMDB、云资源、运维与 AI 管理接口",
            "version": "0.1.0"
        },
        "servers": [{ "url": "/api" }],
        "components": {
            "securitySchemes": {
                "bearerAuth": {
                    "type": "http",
                    "scheme": "bearer",
                    "bearerFormat": "JWT"
                }
            }
        },
        "paths": {
            "/health": {
                "get": { "tags": ["运维"], "summary": "健康检查", "responses": { "200": { "description": "服务正常" } } }
            },
            "/system/auth/login": {
                "post": { "tags": ["认证"], "summary": "登录", "responses": { "200": { "description": "访问令牌与刷新令牌" }, "401": { "description": "用户名或密码错误" } } }
            },
            "/system/auth/refresh-token": {
                "get": { "tags": ["认证"], "summary": "刷新访问令牌", "responses": { "200": { "description": "新令牌" }, "401": { "description": "刷新令牌无效" } } }
            },
            "/system/auth/logout": {
                "post": { "tags": ["认证"], "summary": "退出并撤销刷新令牌", "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "退出成功" } } }
            },
            "/system/auth/me": {
                "get": { "tags": ["认证"], "summary": "当前用户、角色与权限", "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "当前用户信息" } } }
            },
            "/infra/asset/page": {
                "get": { "tags": ["资产中心"], "summary": "分页查询资产台账", "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "资产分页" } } }
            },
            "/infra/network-policy/page": {
                "get": { "tags": ["资产中心"], "summary": "分页查询网络策略", "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "网络策略分页" } } }
            },
            "/cmdb/model/page": {
                "get": { "tags": ["CMDB"], "summary": "分页查询 CMDB 模型", "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "CMDB 模型分页" } } }
            },
            "/cmdb/instance/page": {
                "get": { "tags": ["CMDB"], "summary": "分页查询 CMDB 实例", "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "CMDB 实例分页" } } }
            },
            "/ai/model/page": {
                "get": { "tags": ["AI 大模型"], "summary": "分页查询统一 AI 模型配置", "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "AI 模型配置分页" } } }
            },
            "/ai/model/test": {
                "post": { "tags": ["AI 大模型"], "summary": "测试 AI 模型连接", "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "模型调用结果" } } }
            }
        }
    }))
}
