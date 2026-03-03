/// 机房配置数据模型
#[derive(Clone, Debug, PartialEq)]
pub struct MachineRoomConfig {
    pub id: i32,
    pub room_name: String,                // 机房名称
    pub room_code: String,                // 机房编码
    pub facility_type: String,            // 设施类型（政务云中心、数据中心、服务商机房）
    pub address: String,                  // 详细地址
    pub provider_id: i32,                 // 所属服务商ID
    pub room_type: String,                // 机房类型（核心机房、DMZ机房（公有云）、DMZ机房（政务云））
    pub contact_person: String,           // 负责人
    pub contact_phone: String,            // 联系电话
    pub floor: Option<String>,            // 楼层
    pub cabinet_count: Option<i32>,       // 机柜数量
    pub area_size: Option<String>,        // 面积（平方米）
    pub remarks: Option<String>,          // 备注
    pub status: String,                   // 状态（active/inactive）
    pub created_at: String,
    pub updated_at: Option<String>,
}

impl MachineRoomConfig {
    pub fn display_name(&self) -> String {
        format!("{} - {}", self.room_name, self.facility_type)
    }

    pub fn display_short(&self) -> String {
        self.room_name.clone()
    }
}

/// 初始化默认机房数据
/// provider_id 对应: 1=电信, 2=联通, 3=移动, 4=广电
/// 机房类型规则: 数据中心→核心机房; 服务商机房→DMZ机房（公有云）/DMZ机房（政务云）
pub fn init_machine_rooms() -> Vec<MachineRoomConfig> {
    vec![
        // 电信机房 (provider_id = 1)
        MachineRoomConfig {
            id: 1,
            room_name: "市政务云机房A".to_string(),
            room_code: "DX-ZZY-A".to_string(),
            facility_type: "服务商机房".to_string(),
            address: "市政大厦1号楼地下室".to_string(),
            provider_id: 1,
            room_type: "DMZ机房（政务云）".to_string(),
            contact_person: "张三".to_string(),
            contact_phone: "13800138001".to_string(),
            floor: Some("B1".to_string()),
            cabinet_count: Some(120),
            area_size: Some("2000".to_string()),
            remarks: Some("政务云专用，面向政务内网服务".to_string()),
            status: "active".to_string(),
            created_at: "2024-01-01 09:00".to_string(),
            updated_at: Some("2024-01-15 10:30".to_string()),
        },
        MachineRoomConfig {
            id: 2,
            room_name: "核心机房".to_string(),
            room_code: "DX-CORE".to_string(),
            facility_type: "数据中心".to_string(),
            address: "电信大楼3楼".to_string(),
            provider_id: 1,
            room_type: "核心机房".to_string(),
            contact_person: "李四".to_string(),
            contact_phone: "13800138002".to_string(),
            floor: Some("3F".to_string()),
            cabinet_count: Some(200),
            area_size: Some("3500".to_string()),
            remarks: Some("核心区，敏感业务系统".to_string()),
            status: "active".to_string(),
            created_at: "2024-01-02 14:00".to_string(),
            updated_at: Some("2024-01-14 16:20".to_string()),
        },
        // 联通机房 (provider_id = 2)
        MachineRoomConfig {
            id: 3,
            room_name: "市政务云机房B".to_string(),
            room_code: "LT-ZZY-B".to_string(),
            facility_type: "服务商机房".to_string(),
            address: "市民中心5号楼".to_string(),
            provider_id: 2,
            room_type: "DMZ机房（政务云）".to_string(),
            contact_person: "王五".to_string(),
            contact_phone: "13800138003".to_string(),
            floor: Some("2F".to_string()),
            cabinet_count: Some(100),
            area_size: Some("1800".to_string()),
            remarks: Some("政务云专用，对外业务服务".to_string()),
            status: "active".to_string(),
            created_at: "2024-01-03 11:00".to_string(),
            updated_at: Some("2024-01-12 09:15".to_string()),
        },
        MachineRoomConfig {
            id: 4,
            room_name: "联通核心机房".to_string(),
            room_code: "LT-CORE".to_string(),
            facility_type: "数据中心".to_string(),
            address: "联通产业园A栋".to_string(),
            provider_id: 2,
            room_type: "核心机房".to_string(),
            contact_person: "赵六".to_string(),
            contact_phone: "13800138004".to_string(),
            floor: Some("1F".to_string()),
            cabinet_count: Some(150),
            area_size: Some("2500".to_string()),
            remarks: Some("核心区，核心数据存储".to_string()),
            status: "active".to_string(),
            created_at: "2024-01-04 10:00".to_string(),
            updated_at: Some("2024-01-11 14:30".to_string()),
        },
        // 移动机房 (provider_id = 3)
        MachineRoomConfig {
            id: 5,
            room_name: "市政务云机房C".to_string(),
            room_code: "YD-ZZY-C".to_string(),
            facility_type: "服务商机房".to_string(),
            address: "移动大厦裙楼".to_string(),
            provider_id: 3,
            room_type: "DMZ机房（政务云）".to_string(),
            contact_person: "孙七".to_string(),
            contact_phone: "13800138005".to_string(),
            floor: Some("1F".to_string()),
            cabinet_count: Some(80),
            area_size: Some("1500".to_string()),
            remarks: Some("政务云专用，公共服务平台".to_string()),
            status: "active".to_string(),
            created_at: "2024-01-05 15:30".to_string(),
            updated_at: Some("2024-01-10 09:15".to_string()),
        },
        MachineRoomConfig {
            id: 6,
            room_name: "移动核心机房".to_string(),
            room_code: "YD-CORE".to_string(),
            facility_type: "数据中心".to_string(),
            address: "移动数据中心B栋".to_string(),
            provider_id: 3,
            room_type: "核心机房".to_string(),
            contact_person: "周八".to_string(),
            contact_phone: "13800138006".to_string(),
            floor: Some("2F".to_string()),
            cabinet_count: Some(180),
            area_size: Some("3000".to_string()),
            remarks: Some("核心区，关键业务系统".to_string()),
            status: "active".to_string(),
            created_at: "2024-01-06 16:00".to_string(),
            updated_at: Some("2024-01-09 11:20".to_string()),
        },
        // 广电机房 (provider_id = 4)
        MachineRoomConfig {
            id: 7,
            room_name: "互联网机房".to_string(),
            room_code: "GD-INTERNET".to_string(),
            facility_type: "服务商机房".to_string(),
            address: "广电大楼裙楼".to_string(),
            provider_id: 4,
            room_type: "DMZ机房（公有云）".to_string(),
            contact_person: "钱九".to_string(),
            contact_phone: "13800138007".to_string(),
            floor: Some("1F".to_string()),
            cabinet_count: Some(60),
            area_size: Some("1200".to_string()),
            remarks: Some("面向互联网服务，新业务试点".to_string()),
            status: "active".to_string(),
            created_at: "2024-01-07 10:00".to_string(),
            updated_at: Some("2024-01-08 16:00".to_string()),
        },
    ]
}
