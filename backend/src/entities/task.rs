use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "tasks")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub name: String,
    pub target: String,
    pub status: String,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub found_assets: i32,
    pub found_risks: i32,
    pub port_policy: String,
    pub domain_brute: i32,
    pub service_detection: i32,
    pub os_detection: i32,
    pub site_identify: i32,
    pub created_by: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
