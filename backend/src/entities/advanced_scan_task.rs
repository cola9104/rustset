use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "advanced_scan_tasks")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub name: String,
    pub targets: String, // JSON array of strings
    pub config: String, // JSON of AdvancedScanConfig
    pub status: String, // TaskStatus as string
    pub progress: f32,
    pub current_target: Option<String>,
    pub scanned_count: i32,
    pub total_count: i32,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub created_by: Option<String>,
    pub error_message: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
