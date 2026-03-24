use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "security_products")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub category: String,
    pub vendor: String,
    pub model: String,
    pub version: String,
    pub serial_number: Option<String>,
    pub license_type: String,
    pub license_expiry: Option<String>,
    pub management_ip: Option<String>,
    pub deployment_mode: String,
    pub cloud_platform_id: Option<i32>,
    pub machine_room_id: Option<i32>,
    pub provider_id: Option<i32>,
    pub status: String,
    pub features: Option<String>, // JSON array
    pub throughput: Option<String>,
    pub contact_person: String,
    pub contact_phone: String,
    pub remarks: Option<String>,
    pub created_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::cloud_platform_config::Entity",
        from = "Column::CloudPlatformId",
        to = "super::cloud_platform_config::Column::Id"
    )]
    CloudPlatformConfig,
    #[sea_orm(
        belongs_to = "super::machine_room::Entity",
        from = "Column::MachineRoomId",
        to = "super::machine_room::Column::Id"
    )]
    MachineRoom,
    #[sea_orm(
        belongs_to = "super::service_provider::Entity",
        from = "Column::ProviderId",
        to = "super::service_provider::Column::Id"
    )]
    ServiceProvider,
}

impl Related<super::cloud_platform_config::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CloudPlatformConfig.def()
    }
}

impl Related<super::machine_room::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MachineRoom.def()
    }
}

impl Related<super::service_provider::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ServiceProvider.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
