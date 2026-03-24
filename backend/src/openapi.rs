use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "RustSet API",
        version = "0.1.0",
        description = "Asset and Risk Management System API",
        contact(
            name = "RustSet Team"
        )
    ),
    paths(
        // Auth
        crate::handlers::auth::login,
        crate::handlers::auth::logout,
        crate::handlers::auth::refresh_token,
        // Users
        crate::handlers::users::get_users,
        crate::handlers::users::create_user,
        crate::handlers::users::delete_user,
        crate::handlers::users::update_user_permissions,
        crate::handlers::users::change_password,
        crate::handlers::users::get_current_user_info,
        // Roles
        crate::handlers::roles::get_roles,
        crate::handlers::roles::get_role,
        crate::handlers::roles::create_role,
        crate::handlers::roles::update_role,
        crate::handlers::roles::delete_role,
        // Business Resources
        crate::handlers::business_resources::get_business_resources,
        crate::handlers::business_resources::create_business_resource,
        crate::handlers::business_resources::update_business_resource,
        crate::handlers::business_resources::delete_business_resource,
        // Scanners
        crate::handlers::scanners::get_scanners,
        crate::handlers::scanners::create_scanner,
        crate::handlers::scanners::update_scanner,
        crate::handlers::scanners::delete_scanner,
    ),
    components(
        schemas(
            shared::LoginRequest,
            shared::LoginResponse,
            shared::User,
            shared::CreateUserRequest,
            shared::UpdateUserRequest,
            shared::Role,
            shared::CustomRole,
            shared::Permissions,
            shared::CreateRoleRequest,
            shared::UpdateRoleRequest,
            shared::BusinessResource,
            shared::CreateBusinessResourceRequest,
            shared::UpdateBusinessResourceRequest,
            shared::AuditLog,
            shared::ScannerConfig,
            shared::CreateScannerRequest,
            shared::UpdateScannerRequest,
        )
    ),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "users", description = "User management"),
        (name = "roles", description = "Role and permission management"),
        (name = "business_resources", description = "Business resource management"),
        (name = "scanners", description = "Scanner configuration management"),
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            )
        }
    }
}
