#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OrganizationRecord {
    pub id: i32,
    pub name: String,
    pub code: String,
    pub status: String,
    pub remarks: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
}
