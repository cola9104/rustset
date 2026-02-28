use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "business_resources")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub resource_type: String,
    pub ecs_name: String,
    pub ecs_status: String,
    pub resource_id: String,
    pub cloud_region: String,
    pub cloud_category: String,
    pub cloud_provider_config_id: Option<i32>,
    pub zone_name: Option<String>,
    pub platform_name: Option<String>,
    pub county_city: Option<String>,
    pub vdc_name: Option<String>,
    pub customer_name: String,
    pub application_name: Option<String>,
    pub contract_name: Option<String>,
    pub instance_id: String,
    pub ecs_type: String,
    pub ecs_os: String,
    pub cpu_cores: i32,
    pub memory_gb: i32,
    pub system_disk: String,
    pub system_disk_size_gb: i32,
    pub data_disk: Option<String>,
    pub completion_time: Option<String>,
    pub release_time: Option<String>,
    pub has_security_product: i32,
    pub ip_address: String,
    pub ecs_login_method: Option<String>,
    pub ecs_login_username: Option<String>,
    pub ecs_initial_password: Option<String>,
    pub bastion_address: Option<String>,
    pub bastion_admin_account: Option<String>,
    pub bastion_initial_password: Option<String>,
    pub serial_number: Option<String>,
    pub rack_location: Option<String>,
    pub hardware_model: Option<String>,
    pub warranty_expiry: Option<String>,
    pub agent_status: Option<String>,
    pub ipmi_address: Option<String>,
    pub remarks: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub created_by: Option<String>,
    pub updated_by: Option<String>,
    // 申请状态: 待审核、已批准、已拒绝
    pub application_status: Option<String>,
    // 交付状态: 待交付、交付中、已交付
    pub delivery_status: Option<String>,
    // 交付确认时间
    pub delivery_confirmed_at: Option<String>,
    // 交付确认人
    pub delivery_confirmed_by: Option<String>,

    // 申请流程相关
    pub applicant: Option<String>,              // 申请人
    pub department: Option<String>,             // 申请部门
    pub approver: Option<String>,               // 审批人
    pub approval_time: Option<String>,          // 审批时间
    pub approval_remarks: Option<String>,       // 审批备注
    pub rejection_reason: Option<String>,       // 拒绝原因

    // 资源配置相关
    pub bandwidth_mbps: Option<i32>,            // 带宽大小(Mbps)
    pub bandwidth_type: Option<String>,         // 带宽类型(按量/包月)
    pub public_ip_count: Option<i32>,           // 公网IP数量
    pub network_type: Option<String>,           // 网络类型(VPC/经典网络)

    // 业务关联相关
    pub project_name: Option<String>,           // 项目名称
    pub project_code: Option<String>,           // 项目编号
    pub business_owner: Option<String>,         // 业务负责人
    pub tech_owner: Option<String>,             // 技术负责人
    pub contact_phone: Option<String>,          // 联系电话

    // 费用相关
    pub billing_method: Option<String>,         // 计费方式(包年包月/按量付费)
    pub purchase_duration: Option<i32>,         // 购买时长(月)
    pub cost_center: Option<String>,            // 成本中心

    // 合规相关
    pub security_level: Option<String>,         // 等保级别(二级/三级)
    pub data_sensitivity: Option<String>,       // 数据敏感级别(公开/内部/机密/绝密)

    // 其他
    pub purpose: Option<String>,                // 用途说明
    pub expected_delivery_time: Option<String>, // 期望交付时间
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::cloud_provider_config::Entity",
        from = "Column::CloudProviderConfigId",
        to = "super::cloud_provider_config::Column::Id"
    )]
    CloudProviderConfig,
}

impl Related<super::cloud_provider_config::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CloudProviderConfig.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
