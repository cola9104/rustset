use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "application_endpoints")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub business_application_id: i32,
    pub protocol: String,
    pub dest_ip: String,
    pub nat_ip: Option<String>,
    pub dest_port: String,
    pub domain: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub created_by: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
