use crate::config::audit_logs_url;
use crate::utils::storage::authorization_header;
use gloo_net::http::{Request, RequestBuilder};
use serde::Deserialize;
use web_sys::RequestCredentials;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct AuditLogRecord {
    pub id: String,
    pub user_id: String,
    pub username: String,
    pub action: String,
    pub target: String,
    pub details: String,
    pub timestamp: String,
}

fn with_auth(mut request: RequestBuilder) -> RequestBuilder {
    if let Some(header) = authorization_header() {
        request = request.header("Authorization", &header);
    }
    request
}

pub async fn fetch_audit_logs() -> Result<Vec<AuditLogRecord>, String> {
    let response =
        with_auth(Request::get(&audit_logs_url()).credentials(RequestCredentials::Include))
            .send()
            .await
            .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        return Err(format!("服务器错误: {}", response.status()));
    }

    response
        .json::<Vec<AuditLogRecord>>()
        .await
        .map_err(|e| format!("解析失败: {}", e))
}
