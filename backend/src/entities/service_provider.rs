use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "service_providers")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub provider_name: String,
    pub provider_code: String,
    pub short_name: String,
    pub logo_url: Option<String>,
    pub contact_person: String,
    pub contact_phone: String,
    pub contact_email: String,
    pub headquarters: String,
    pub service_area: String,
    pub business_license: String,
    pub remarks: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::machine_room::Entity")]
    MachineRoom,
    #[sea_orm(has_many = "super::cloud_platform_config::Entity")]
    CloudPlatformConfig,
}

impl Related<super::machine_room::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MachineRoom.def()
    }
}

impl Related<super::cloud_platform_config::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CloudPlatformConfig.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
