use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    #[sea_orm(unique)]
    pub username: String,
    pub real_name: Option<String>,
    pub password: String,
    pub role: String,
    pub permissions: Option<String>,
    pub created_at: String,
    pub password_changed_at: Option<String>,
    pub password_strength: Option<String>,
    pub force_password_change: i32,
    pub last_login_at: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub status: Option<String>,
    pub organization_id: Option<i32>,
    pub department_id: Option<i32>,
    pub failed_login_attempts: Option<i32>,
    pub locked_until: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
