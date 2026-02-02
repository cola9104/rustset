use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "quick_scan_results")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub task_id: String, // Foreign key to advanced_scan_tasks
    pub ip: String,
    pub is_alive: bool,
    pub open_ports: String, // JSON array of PortInfo
    pub fingerprint: Option<String>,
    pub scanned_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
