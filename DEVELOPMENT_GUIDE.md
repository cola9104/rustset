# RustSet开发建议

**适用工具栈**：VS Code + Dioxus + Claude Code
**生成时间**：2026-03-12

---

## 🎯 核心建议：使用Claude Code作为主要编码助手

**为什么选择Claude Code？**
1. ✅ **Rust专家** - Claude对Rust的掌握远超其他AI
2. ✅ **Dioxus熟悉** - 对现代Rust前端框架有深度理解
3. ✅ **异步编程** - 对Tokio和async/await有深刻理解
4. ✅ **类型系统** - 能有效利用Rust的强类型系统
5. ✅ **最佳实践** - 熟悉Rust生态的最佳实践和常用库

---

## 📝 具体开发工作流

### 前端开发（Dioxus）

#### 1. Claude Code最佳实践
```rust
// ❌ 不要这样问Claude
"帮我写一个用户登录页面"

// ✅ 这样问
"我需要为RustSet项目写一个用户登录组件。要求：
- 使用Dioxus框架
- 支持用户名/密码登录
- 包含表单验证
- 错误提示清晰
- 与后端API /api/login 对接
- 使用Tailwind CSS进行样式
- 前端状态管理使用signals

这是我的当前代码结构：
[code粘贴]

请帮我实现完整的登录组件。"
```

#### 2. Dioxus开发建议

**组件结构：**
```rust
// src/components/login.rs
use dioxus::prelude::*;

#[component]
pub fn Login(cx: Scope) -> Element {
    // 使用signals管理状态
    let username = use_signal(cx, || String::new());
    let password = use_signal(cx, || String::new());
    let error_msg = use_signal(cx, || String::new());
    
    // 表单验证
    let is_valid = move || {
        !username.get().is_empty() 
            && !password.get().is_empty()
            && password.get().len() >= 8
    };

    rsx! {
        div { class: "login-container min-h-screen flex items-center justify-center bg-gray-100" {
            div { class: "max-w-md w-full bg-white rounded-lg shadow-lg p-8" {
                h2 { "用户登录" }
                
                if error_msg.get().is_empty() {
                    div { class: "alert alert-error" {
                        {error_msg.get()}
                    }
                }
                
                form { 
                    onsubmit: move |event| {
                        event.prevent_default();
                        // 登录逻辑
                    }
                    class: "space-y-4"
                {
                    div { class: "form-group" {
                        label { "用户名" }
                        input {
                            r#type: "text",
                            value: "{username}",
                            oninput: move |evt| username.set(evt.value())
                        }
                    }
                    }
                    
                    div { class: "form-group" {
                        label { "密码" }
                        input {
                            r#type: "password",
                            value: "{password}",
                            oninput: move |evt| password.set(evt.value())
                        }
                    }
                    }
                    
                    button {
                        r#type: "submit",
                        disabled: !is_valid(),
                        class: "w-full bg-blue-600 text-white py-2 px-4 rounded hover:bg-blue-700 disabled:opacity-50"
                    } {
                        "登录"
                    }
                }
            }
        }
    }
}
```

**状态管理建议：**
```rust
// src/state/user.rs
use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct UserState {
    pub username: String,
    pub is_logged_in: bool,
    pub token: Option<String>,
}

impl UserState {
    pub fn new() -> Self {
        Self {
            username: String::new(),
            is_logged_in: false,
            token: None,
        }
    }
}

// 使用signals
pub fn use_user_state(cx: Scope) -> UserState {
    let state = use_shared_state::<UserState>(cx);
    state
}
```

#### 3. API调用最佳实践
```rust
use gloo_net::httpclient::WebSocket;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Deserialize, Serialize)]
struct LoginResponse {
    success: bool,
    token: Option<String>,
    user: Option<UserInfo>,
}

pub async fn login(
    username: String,
    password: String,
) -> Result<LoginResponse, String> {
    let client = WebSocket::new("/api/login").unwrap();
    
    let request = LoginRequest {
        username,
        password,
    };
    
    match client.post(&request).await {
        Ok(response) => Ok(response),
        Err(e) => Err(format!("登录失败: {}", e)),
    }
}
```

### 后端开发（Axum + SeaORM）

#### 1. Axum开发建议

**错误处理：**
```rust
// src/error.rs
use axum::{
    response::{IntoResponse, Response},
    Json,
    http::StatusCode,
};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    Unauthorized(String),
    BadRequest(String),
    InternalError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        (status, Json(json!({"error": message}))).into_response()
    }
}

// 使用
pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<User>, AppError> {
    state.users.lock()
        .iter()
        .find(|u| u.id == id)
        .cloned()
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))
        .map(Json)
}
```

**路由组织：**
```rust
// src/routes.rs
use axum::Router;

pub fn create_routes() -> Router {
    Router::new()
        // 认证路由
        .nest("/api/auth", auth_routes())
        
        // 用户路由
        .nest("/api/users", user_routes())
        
        // 资产路由
        .nest("/api/assets", asset_routes())
        
        // 扫描路由
        .nest("/api/scan", scan_routes())
}
```

#### 2. SeaORM最佳实践

**查询优化：**
```rust
// ❌ 避免：获取所有数据后在内存中过滤
let all_assets = assets::Entity::find()
    .all(db)
    .await
    .into_iter()
    .filter(|a| a.zone == NetworkZone::Intranet)
    .collect();

// ✅ 推荐：在数据库层面过滤
let filtered_assets = assets::Entity::find()
    .filter(assets::Column::Zone.eq(NetworkZone::Intranet))
    .all(db)
    .await?;
```

**事务处理：**
```rust
use sea_orm::TransactionTrait;

pub async fn create_asset_with_ports(
    db: &DatabaseConnection,
    asset_data: AssetData,
    ports: Vec<PortData>,
) -> Result<Asset, AppError> {
    let txn = db.begin().await?;
    
    match txn {
        async {
            // 创建资产
            let asset = assets::ActiveModel {
                id: Uuid::new_v4(),
                name: asset_data.name.clone(),
                zone: asset_data.zone.clone(),
                ip: asset_data.ip.clone(),
                ..Default::default()
            }
            .insert(&txn).await?;
            
            // 创建端口
            for port in ports {
                let port_model = ports::ActiveModel {
                    id: Uuid::new_v4(),
                    asset_id: asset.id,
                    port: port.port,
                    service: port.service,
                    ..Default::default()
                };
                port_model.insert(&txn).await?;
            }
            
            txn.commit().await?;
            
            Ok(asset)
        }
        .await
    }
}
```

---

## 🚀 VS Code配置建议

### 推荐扩展

```json
{
  "recommendations": {
    "extensions": [
      "rust-lang.rust-analyzer",
      "vadimcn.vscode-lldb",
      "matklad.rust-syntax",
      "serayuzg.rust-parquet",
      "usernamehw.errorlens",
      "eamodio.gitlens",
      "dbaeumer.vscode-gitlens",
      "streetsidesoftware.code-spell-checker"
    ]
  }
}
```

### VS Code设置

```json
{
  "[rust]": {
    "rust-analyzer.cargo.loadOutDirsFromCheck": true,
    "rust-analyzer.checkOnSave.command": "clippy"
  },
  "[rust-analyzer]": {
    "checkOnSave": {
      "command": "clippy"
    },
    "cargo.loadOutDirsFromCheck": true,
    "procMacro.enable": true
  }
}
```

### Tasks配置（编译和检查）

```json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "cargo check",
      "type": "shell",
      "command": "cargo check",
      "problemMatcher": [
        "$rustc"
      ],
      "group": {
        "kind": "build",
        "isDefault": true
      }
    },
    {
      "label": "cargo clippy",
      "type": "shell",
      "command": "cargo clippy",
      "problemMatcher": [
        "$rustc"
      ],
      "group": "build"
    },
    {
      "label": "cargo test",
      "type": "shell",
      "command": "cargo test",
      "problemMatcher": [
        "$rustc"
      ],
      "group": "test"
    }
  ]
}
```

---

## 💡 Claude Code使用技巧

### 1. 代码审查
**场景：** 想改进现有代码

**提示词：**
```markdown
请帮我审查以下RustSet代码，重点关注：
1. 错误处理是否完整
2. 是否有潜在的并发问题（如Arc、Mutex的使用）
3. 性能是否有优化空间
4. 代码是否符合Rust惯用语
5. 是否有安全隐患

请提供：
- 具体问题列表
- 优化建议
- 修改后的代码示例

[代码粘贴]
```

### 2. 架构设计
**场景：** 需要设计新功能

**提示词：**
```markdown
我需要在RustSet中添加"批量导入资产"功能。请帮我设计：

1. 数据库schema设计（需要的表和字段）
2. API路由设计（Axum路由结构）
3. 前端组件设计（Dioxus组件）
4. 错误处理策略
5. 并发安全考虑（使用Tokio和锁）

技术栈：
- 前端：Dioxus + Gloo-net
- 后端：Axum + SeaORM
- 数据库：SQLite
- 异步：Tokio

请提供完整的设计方案和代码框架。
```

### 3. 性能优化
**场景：** 性能问题

**提示词：**
```markdown
RustSet应用的响应时间很慢，特别是资产列表API（/api/assets）。请帮我分析可能的原因并提供优化方案：

当前情况：
- 后端：Axum + SeaORM + SQLite
- 前端：Dioxus
- 数据量：约1000个资产
- 响应时间：3-5秒

可能的原因（请帮我验证和优化）：
1. N+1查询问题（循环查询数据库）
2. 没有使用数据库索引
3. 返回了不必要的数据
4. 内存中大量对象创建

请提供：
1. 问题诊断
2. 具体的优化代码
3. 数据库索引建议
4. 前端渲染优化建议
```

### 4. 错误调试
**场景：** 出现bug

**提示词：**
```markdown
我在RustSet中遇到了以下错误，请帮我定位和修复：

错误信息：
[错误信息粘贴]

调用栈：
[调用栈粘贴]

相关代码：
[相关代码粘贴]

请帮我：
1. 分析错误的根本原因
2. 提供修复方案
3. 提供预防措施
4. 如果是Rust生命周期或所有权问题，请详细解释
```

---

## 📚 学习资源

### Rust学习
- **The Rust Book** - 官方教程
- **Rust by Example** - 实例丰富
- **Rustlings** - 小练习题

### Dioxus学习
- **Dioxus官方文档** - https://dioxuslabs.com/docs/main/guide/
- **Dioxus最佳实践** - https://dioxuslabs.com/docs/main/best_practices/
- **Dioxus Awesome** - https://github.com/DioxusLabs/awesome-dioxus

### Axum学习
- **Axum官方文档** - https://docs.rs/axum/axum/index.html
- **Axum Examples** - https://github.com/tokio-rs/axum/tree/main/examples

### SeaORM学习
- **SeaORM官方文档** - https://www.sea-ql.org/SeaORM/docs/index.html
- **SeaORM Examples** - https://github.com/SeaQL/sea-orm/tree/master/examples

---

## 🎯 具体行动建议

### 前端优化行动
1. [ ] 使用`use_memo`缓存计算密集型操作
2. [ ] 将大型组件拆分为更小的子组件
3. [ ] 使用`use_resource_loader`异步加载资源
4. [ ] 实现虚拟滚动（针对资产列表）

### 后端优化行动
1. [ ] 修复`AppState`的并发问题（使用RwLock）
2. [ ] 添加数据库连接池（SeaORM支持）
3. [ ] 实现统一的错误处理中间件
4. [ ] 修复密码明文存储（使用Argon2）

### 架构改进行动
1. [ ] 考虑引入API版本控制
2. [ ] 实现请求限流（防止滥用）
3. [ ] 添加API文档（使用utoipa）
4. [ ] 实现WebSocket（用于实时扫描进度推送）

---

## 💬 与AI Agent的协作

### 如何让Main Agent帮助你

**前端开发：**
```
@moss 帮我优化资产列表页面的渲染性能

当前问题：1000个资产，渲染很慢
技术栈：Dioxus + Rust
```

**后端开发：**
```
@moss 帮我重构AppState，解决并发性能问题

当前使用Arc<StdMutex<>>，有锁竞争
希望使用更高效的并发方案
```

**架构设计：**
```
@moss 帮我设计资产批量导入的完整方案

需要：数据库设计、API设计、前端组件、错误处理
技术栈：Axum + SeaORM + Dioxus
```

---

_最后更新：2026-03-12_
