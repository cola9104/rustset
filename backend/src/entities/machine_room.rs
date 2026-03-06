use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "machine_rooms")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub room_name: String,
    pub room_code: String,
    pub facility_type: String,
    pub address: String,
    pub provider_id: i32,
    pub room_type: String,
    pub contact_person: String,
    pub contact_phone: String,
    pub floor: Option<String>,
    pub cabinet_count: Option<i32>,
    pub area_size: Option<String>,
    pub remarks: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::service_provider::Entity",
        from = "Column::ProviderId",
        to = "super::service_provider::Column::Id"
    )]
    ServiceProvider,
    #[sea_orm(has_many = "super::cloud_platform_config::Entity")]
    CloudPlatformConfig,
    #[sea_orm(has_many = "super::security_product::Entity")]
    SecurityProduct,
}

impl Related<super::service_provider::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ServiceProvider.def()
    }
}

impl Related<super::cloud_platform_config::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CloudPlatformConfig.def()
    }
}

impl Related<super::security_product::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SecurityProduct.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
