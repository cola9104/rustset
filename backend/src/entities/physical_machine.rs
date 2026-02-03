use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "physical_machines")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub business_resource_id: i32,
    pub serial_number: Option<String>,
    pub rack_location: Option<String>,
    pub hardware_model: Option<String>,
    pub warranty_expiry: Option<String>,
    pub agent_status: Option<String>,
    pub ipmi_address: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::business_resource::Entity",
        from = "Column::BusinessResourceId",
        to = "super::business_resource::Column::Id"
    )]
    BusinessResource,
}

impl Related<super::business_resource::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::BusinessResource.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
