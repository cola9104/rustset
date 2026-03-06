use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "cloud_platform_configs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub platform_name: String,
    pub provider_id: i32,
    pub cloud_type: String,
    pub foundation: String,
    pub region_id: String,
    pub machine_room_id: i32,
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
        belongs_to = "super::service_provider::Entity",
        from = "Column::ProviderId",
        to = "super::service_provider::Column::Id"
    )]
    ServiceProvider,
    #[sea_orm(
        belongs_to = "super::machine_room::Entity",
        from = "Column::MachineRoomId",
        to = "super::machine_room::Column::Id"
    )]
    MachineRoom,
    #[sea_orm(has_many = "super::security_product::Entity")]
    SecurityProduct,
}

impl Related<super::service_provider::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ServiceProvider.def()
    }
}

impl Related<super::machine_room::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MachineRoom.def()
    }
}

impl Related<super::security_product::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SecurityProduct.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
