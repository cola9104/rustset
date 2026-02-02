use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "assets")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub ip: String,
    pub zone: String,
    pub ports: String, // JSON string
    pub last_scanned: Option<String>,
    pub contact_person: Option<String>,
    pub contact_phone: Option<String>,
    pub created_by: Option<String>,
    pub updated_by: Option<String>,
    pub owner: Option<String>,
    pub weight: i32,
    pub labels: Option<String>, // JSON string
    pub os: Option<String>,
    pub device_type: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
