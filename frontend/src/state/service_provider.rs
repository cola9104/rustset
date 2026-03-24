/// 服务商（运营商）配置数据模型
#[derive(Clone, Debug, PartialEq)]
pub struct ServiceProviderConfig {
    pub id: i32,
    pub provider_name: String,    // 服务商名称（电信/联通/移动/广电）
    pub provider_code: String,    // 服务商编码
    pub short_name: String,       // 简称
    pub logo_url: Option<String>, // Logo URL
    pub contact_person: String,   // 负责人
    pub contact_phone: String,    // 联系电话
    pub contact_email: String,    // 联系邮箱
    pub headquarters: String,     // 总部地址
    pub service_area: String,     // 服务区域
    pub business_license: String, // 营业执照号
    pub remarks: Option<String>,  // 备注
    pub status: String,           // 状态（active/inactive）
    pub created_at: String,
    pub updated_at: Option<String>,
}

impl ServiceProviderConfig {
    pub fn display_name(&self) -> String {
        format!("{} ({})", self.provider_name, self.short_name)
    }
}

/// 初始化默认服务商数据
pub fn init_service_providers() -> Vec<ServiceProviderConfig> {
    vec![
        ServiceProviderConfig {
            id: 1,
            provider_name: "中国电信".to_string(),
            provider_code: "CHINA_TELECOM".to_string(),
            short_name: "电信".to_string(),
            logo_url: Some("/images/telecom-logo.png".to_string()),
            contact_person: "张经理".to_string(),
            contact_phone: "10000".to_string(),
            contact_email: "telecom@example.com".to_string(),
            headquarters: "北京市西城区金融大街35号".to_string(),
            service_area: "全国".to_string(),
            business_license: "91110000100000001X".to_string(),
            remarks: Some("基础电信业务运营商".to_string()),
            status: "active".to_string(),
            created_at: "2024-01-01 09:00".to_string(),
            updated_at: Some("2024-01-15 10:30".to_string()),
        },
        ServiceProviderConfig {
            id: 2,
            provider_name: "中国联通".to_string(),
            provider_code: "CHINA_UNICOM".to_string(),
            short_name: "联通".to_string(),
            logo_url: Some("/images/unicom-logo.png".to_string()),
            contact_person: "王经理".to_string(),
            contact_phone: "10010".to_string(),
            contact_email: "unicom@example.com".to_string(),
            headquarters: "北京市西城区金融大街21号".to_string(),
            service_area: "全国".to_string(),
            business_license: "91110000100000002X".to_string(),
            remarks: Some("基础电信业务运营商".to_string()),
            status: "active".to_string(),
            created_at: "2024-01-02 14:00".to_string(),
            updated_at: Some("2024-01-14 16:20".to_string()),
        },
        ServiceProviderConfig {
            id: 3,
            provider_name: "中国移动".to_string(),
            provider_code: "CHINA_MOBILE".to_string(),
            short_name: "移动".to_string(),
            logo_url: Some("/images/mobile-logo.png".to_string()),
            contact_person: "李经理".to_string(),
            contact_phone: "10086".to_string(),
            contact_email: "mobile@example.com".to_string(),
            headquarters: "北京市西城区金融大街29号".to_string(),
            service_area: "全国".to_string(),
            business_license: "91110000100000003X".to_string(),
            remarks: Some("基础电信业务运营商".to_string()),
            status: "active".to_string(),
            created_at: "2024-01-03 11:00".to_string(),
            updated_at: Some("2024-01-12 09:15".to_string()),
        },
        ServiceProviderConfig {
            id: 4,
            provider_name: "中国广电".to_string(),
            provider_code: "CHINA_BROADCASTING".to_string(),
            short_name: "广电".to_string(),
            logo_url: Some("/images/broadcasting-logo.png".to_string()),
            contact_person: "赵经理".to_string(),
            contact_phone: "10099".to_string(),
            contact_email: "broadcasting@example.com".to_string(),
            headquarters: "北京市朝阳区光华路甲1号".to_string(),
            service_area: "全国".to_string(),
            business_license: "91110000100000004X".to_string(),
            remarks: Some("新晋基础电信业务运营商".to_string()),
            status: "active".to_string(),
            created_at: "2024-01-04 10:00".to_string(),
            updated_at: Some("2024-01-11 14:30".to_string()),
        },
    ]
}
