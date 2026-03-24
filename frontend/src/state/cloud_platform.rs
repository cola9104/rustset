/// 云平台配置数据模型
#[derive(Clone, Debug, PartialEq)]
pub struct CloudPlatformConfig {
    pub id: i32,
    pub platform_name: String,
    pub provider_id: i32,
    pub cloud_type: String,
    pub foundation: String,
    pub region_id: String,
    pub machine_room_id: i32,
    pub access_key_id: String,
    pub access_key_secret: String,
    pub remarks: Option<String>,
    pub status: String,
    pub last_test_time: Option<String>,
    pub last_test_result: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
}
