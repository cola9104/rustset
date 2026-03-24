/// 云平台配置数据模型
#[derive(Clone, Debug, PartialEq)]
pub struct CloudPlatformConfig {
    pub id: i32,
    pub platform_name: String,
    pub provider_id: i32,   // 服务商ID
    pub cloud_type: String, // 云类型（公有云、政务云）
    pub foundation: String, // 底座（阿里云、华为云等）
    pub region_id: String,
    pub machine_room_id: i32, // 机房ID
    pub access_key_id: String,
    pub access_key_secret: String,
    pub remarks: Option<String>,
    pub status: String,
    pub last_test_time: Option<String>,
    pub last_test_result: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
}

impl CloudPlatformConfig {
    pub fn display_name(&self) -> String {
        format!("{} ({})", self.cloud_type, self.foundation)
    }
}

/// 初始化默认云平台数据
/// provider_id: 1=电信, 2=联通, 3=移动, 4=广电
/// machine_room_id: 1=市政务云机房A, 2=核心机房(电信), 3=市政务云机房B, 4=联通核心机房,
///                 5=市政务云机房C, 6=移动核心机房, 7=市政务云机房D
pub fn init_cloud_platforms() -> Vec<CloudPlatformConfig> {
    vec![
        // 电信 (provider_id = 1) - 阿里云底座
        CloudPlatformConfig {
            id: 1,
            platform_name: "电信-公有云".to_string(),
            provider_id: 1,
            cloud_type: "公有云".to_string(),
            foundation: "阿里云".to_string(),
            region_id: "cn-hangzhou".to_string(),
            machine_room_id: 1, // 市政务云机房A
            access_key_id: "LTAI5tdxxxxxxxxxxxxx".to_string(),
            access_key_secret: "****************************".to_string(),
            remarks: Some("面向互联网服务".to_string()),
            status: "active".to_string(),
            last_test_time: Some("2024-01-15 10:30".to_string()),
            last_test_result: Some("连接成功".to_string()),
            created_at: "2024-01-01 09:00".to_string(),
            updated_at: Some("2024-01-15 10:30".to_string()),
        },
        CloudPlatformConfig {
            id: 2,
            platform_name: "电信-政务云".to_string(),
            provider_id: 1,
            cloud_type: "政务云".to_string(),
            foundation: "阿里云".to_string(),
            region_id: "cn-hangzhou".to_string(),
            machine_room_id: 2, // 核心机房(电信)
            access_key_id: "LTAI5texxxxxxxxxxxxx".to_string(),
            access_key_secret: "****************************".to_string(),
            remarks: Some("政务专用云环境".to_string()),
            status: "active".to_string(),
            last_test_time: Some("2024-01-14 16:20".to_string()),
            last_test_result: Some("连接成功".to_string()),
            created_at: "2024-01-02 14:00".to_string(),
            updated_at: Some("2024-01-14 16:20".to_string()),
        },
        // 联通 (provider_id = 2) - 华为云底座
        CloudPlatformConfig {
            id: 3,
            platform_name: "联通-公有云".to_string(),
            provider_id: 2,
            cloud_type: "公有云".to_string(),
            foundation: "华为云".to_string(),
            region_id: "cn-north-1".to_string(),
            machine_room_id: 3, // 市政务云机房B
            access_key_id: "HWSKTlxxxxxxxxxxxxxx".to_string(),
            access_key_secret: "****************************".to_string(),
            remarks: Some("对外业务服务".to_string()),
            status: "active".to_string(),
            last_test_time: Some("2024-01-12 09:15".to_string()),
            last_test_result: Some("连接成功".to_string()),
            created_at: "2024-01-03 11:00".to_string(),
            updated_at: Some("2024-01-12 09:15".to_string()),
        },
        CloudPlatformConfig {
            id: 4,
            platform_name: "联通-政务云".to_string(),
            provider_id: 2,
            cloud_type: "政务云".to_string(),
            foundation: "华为云".to_string(),
            region_id: "cn-north-1".to_string(),
            machine_room_id: 4, // 联通核心机房
            access_key_id: "HWSKTuxxxxxxxxxxxxxx".to_string(),
            access_key_secret: "****************************".to_string(),
            remarks: Some("政务专用云环境".to_string()),
            status: "active".to_string(),
            last_test_time: Some("2024-01-11 14:30".to_string()),
            last_test_result: Some("连接成功".to_string()),
            created_at: "2024-01-04 10:00".to_string(),
            updated_at: Some("2024-01-11 14:30".to_string()),
        },
        // 移动 (provider_id = 3) - 阿里云底座
        CloudPlatformConfig {
            id: 5,
            platform_name: "移动-公有云".to_string(),
            provider_id: 3,
            cloud_type: "公有云".to_string(),
            foundation: "阿里云".to_string(),
            region_id: "cn-shenzhen".to_string(),
            machine_room_id: 5, // 市政务云机房C
            access_key_id: "LTAI5tmxxxxxxxxxxxxx".to_string(),
            access_key_secret: "****************************".to_string(),
            remarks: Some("公共服务平台".to_string()),
            status: "inactive".to_string(),
            last_test_time: Some("2024-01-10 09:15".to_string()),
            last_test_result: Some("认证失败".to_string()),
            created_at: "2024-01-05 15:30".to_string(),
            updated_at: Some("2024-01-10 09:15".to_string()),
        },
        CloudPlatformConfig {
            id: 6,
            platform_name: "移动-政务云".to_string(),
            provider_id: 3,
            cloud_type: "政务云".to_string(),
            foundation: "阿里云".to_string(),
            region_id: "cn-shenzhen".to_string(),
            machine_room_id: 6, // 移动核心机房
            access_key_id: "LTAI5tnxxxxxxxxxxxxx".to_string(),
            access_key_secret: "****************************".to_string(),
            remarks: Some("政务专用云环境".to_string()),
            status: "active".to_string(),
            last_test_time: Some("2024-01-09 11:20".to_string()),
            last_test_result: Some("连接成功".to_string()),
            created_at: "2024-01-06 16:00".to_string(),
            updated_at: Some("2024-01-09 11:20".to_string()),
        },
    ]
}
