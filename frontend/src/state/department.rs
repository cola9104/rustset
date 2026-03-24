#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DepartmentRecord {
    pub id: i32,
    pub organization_id: i32,
    pub name: String,
    pub code: String,
    pub parent_id: Option<i32>,
    pub level: u32,
    pub status: String,
    pub remarks: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
}
