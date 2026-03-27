//! SeaORM Entity definitions
//!
//! This module contains all SeaORM entity definitions for the application.
//! Each entity corresponds to a database table.

pub mod advanced_scan_task;
pub mod application_endpoint;
pub mod asset;
pub mod audit_log;
pub mod business_application;
pub mod business_resource;
pub mod cloud_platform;
pub mod cloud_platform_config;
pub mod cloud_provider_config;
pub mod cloud_service;
pub mod cloud_virtual_machine;
pub mod cloud_zone;
pub mod custom_role;
pub mod department;
pub mod machine_room;
pub mod network_zone;
pub mod organization;
pub mod physical_machine;
pub mod quick_scan_result;
pub mod region;
pub mod resource_ticket;
pub mod risk;
pub mod security_product;
pub mod service_provider;
pub mod task;
pub mod user;

pub use advanced_scan_task::Entity as AdvancedScanTask;
pub use application_endpoint::Entity as ApplicationEndpoint;
pub use asset::Entity as Asset;
pub use audit_log::Entity as AuditLog;
pub use business_application::Entity as BusinessApplication;
pub use business_resource::Entity as BusinessResource;
pub use cloud_provider_config::Entity as CloudProviderConfig;
pub use cloud_service::Entity as CloudService;
pub use cloud_virtual_machine::Entity as CloudVirtualMachine;
pub use cloud_zone::Entity as CloudZone;
pub use custom_role::Entity as CustomRole;
pub use network_zone::Entity as NetworkZone;
pub use physical_machine::Entity as PhysicalMachine;
pub use risk::Entity as Risk;
pub use task::Entity as Task;
pub use user::Entity as User;

// Re-export the models for convenience
pub mod prelude {
    pub use sea_orm::entity::prelude::*;
}
