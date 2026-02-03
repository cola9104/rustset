use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "cloud_virtual_machines")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub business_resource_id: i32,
    pub billing_mode: Option<String>,
    pub expire_time: Option<String>,
    pub charge_type: Option<String>,
    pub instance_charge_type: Option<String>,
    pub internet_charge_type: Option<String>,
    pub internet_max_bandwidth_out: Option<i32>,
    pub image_id: Option<String>,
    pub v_switch_id: Option<String>,
    pub vpc_id: Option<String>,
    pub security_group_ids: Option<String>, // JSON array
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
