use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "business_resources")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub resource_type: String,
    pub ecs_name: String,
    pub ecs_status: String,
    pub resource_id: String,
    pub cloud_region: String,
    pub cloud_category: String,
    pub cloud_provider_config_id: Option<i32>,
    pub zone_name: Option<String>,
    pub platform_name: Option<String>,
    pub provider_vendor: Option<String>,
    pub county_city: Option<String>,
    pub vdc_name: Option<String>,
    pub customer_name: String,
    pub application_name: Option<String>,
    pub contract_name: Option<String>,
    pub instance_id: String,
    pub ecs_type: String,
    pub ecs_os: String,
    pub cpu_cores: i32,
    pub memory_gb: i32,
    pub system_disk: String,
    pub system_disk_size_gb: i32,
    pub data_disk: Option<String>,
    pub completion_time: Option<String>,
    pub release_time: Option<String>,
    pub has_security_product: i32,
    pub ip_address: String,
    pub ecs_login_method: Option<String>,
    pub ecs_login_username: Option<String>,
    pub ecs_initial_password: Option<String>,
    pub bastion_address: Option<String>,
    pub bastion_admin_account: Option<String>,
    pub bastion_initial_password: Option<String>,
    pub serial_number: Option<String>,
    pub rack_location: Option<String>,
    pub hardware_model: Option<String>,
    pub warranty_expiry: Option<String>,
    pub agent_status: Option<String>,
    pub ipmi_address: Option<String>,
    pub remarks: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub created_by: Option<String>,
    pub updated_by: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::cloud_provider_config::Entity",
        from = "Column::CloudProviderConfigId",
        to = "super::cloud_provider_config::Column::Id"
    )]
    CloudProviderConfig,
}

impl Related<super::cloud_provider_config::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CloudProviderConfig.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
