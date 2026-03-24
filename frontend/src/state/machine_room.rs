/// 机房配置数据模型
#[derive(Clone, Debug, PartialEq)]
pub struct MachineRoomConfig {
    pub id: i32,
    pub room_name: String,
    pub room_code: String,
    pub facility_type: String,
    pub address: String,
    pub provider_id: i32,
    pub room_type: String,
    pub contact_person: String,
    pub contact_phone: String,
    pub floor: Option<String>,
    pub cabinet_count: Option<i32>,
    pub area_size: Option<String>,
    pub remarks: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: Option<String>,
}

impl MachineRoomConfig {
    pub fn display_name(&self) -> String {
        format!("{} - {}", self.room_name, self.facility_type)
    }
}
