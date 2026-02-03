//! SeaORM Entity definitions
//!
//! This module contains all SeaORM entity definitions for the application.
//! Each entity corresponds to a database table.

pub mod cloud_zone;
pub mod cloud_platform;
pub mod cloud_provider_config;
pub mod business_resource;
pub mod physical_machine;
pub mod cloud_virtual_machine;
pub mod user;
pub mod audit_log;
pub mod asset;
pub mod task;
pub mod risk;
pub mod network_zone;
pub mod custom_role;
pub mod advanced_scan_task;
pub mod quick_scan_result;

pub use cloud_zone::Entity as CloudZone;
pub use cloud_platform::Entity as CloudPlatform;
pub use cloud_provider_config::Entity as CloudProviderConfig;
pub use business_resource::Entity as BusinessResource;
pub use physical_machine::Entity as PhysicalMachine;
pub use cloud_virtual_machine::Entity as CloudVirtualMachine;
pub use user::Entity as User;
pub use audit_log::Entity as AuditLog;
pub use asset::Entity as Asset;
pub use task::Entity as Task;
pub use risk::Entity as Risk;
pub use network_zone::Entity as NetworkZone;
pub use custom_role::Entity as CustomRole;
pub use advanced_scan_task::Entity as AdvancedScanTask;
pub use quick_scan_result::Entity as QuickScanResult;

// Re-export the models for convenience
pub mod prelude {
    pub use sea_orm::entity::prelude::*;
}
