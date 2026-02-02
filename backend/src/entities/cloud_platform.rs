use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "cloud_platforms")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub zone_id: i32,
    pub platform_name: String,
    pub platform_code: String,
    pub description: Option<String>,
    pub created_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::cloud_zone::Entity",
        from = "Column::ZoneId",
        to = "super::cloud_zone::Column::Id"
    )]
    CloudZone,
    #[sea_orm(has_many = "super::cloud_provider_config::Entity")]
    CloudProviderConfig,
}

impl Related<super::cloud_zone::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CloudZone.def()
    }
}

impl Related<super::cloud_provider_config::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CloudProviderConfig.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
