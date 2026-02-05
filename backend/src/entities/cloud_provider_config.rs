use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "cloud_provider_configs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub zone_id: i32,
    pub platform_id: i32,
    pub provider: String,
    pub region_id: String,
    pub region_name: String,
    pub account_name: String,
    pub access_key_id: String,
    pub access_key_secret: String,
    pub remarks: Option<String>,
    pub status: String,
    pub last_test_time: Option<String>,
    pub last_test_result: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::cloud_zone::Entity",
        from = "Column::ZoneId",
        to = "super::cloud_zone::Column::Id"
    )]
    CloudZone,
    #[sea_orm(
        belongs_to = "super::cloud_service::Entity",
        from = "Column::PlatformId",
        to = "super::cloud_service::Column::Id"
    )]
    CloudService,
    #[sea_orm(has_many = "super::business_resource::Entity")]
    BusinessResource,
}

impl Related<super::cloud_zone::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CloudZone.def()
    }
}

impl Related<super::cloud_service::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CloudService.def()
    }
}

impl Related<super::business_resource::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::BusinessResource.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
