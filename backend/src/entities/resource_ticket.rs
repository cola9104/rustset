use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "resource_tickets")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    // 资源类型: cloud, physical, network
    pub resource_type: String,
    // 资源名称
    pub ecs_name: String,
    // 工单状态
    pub ticket_status: String,

    // 关联字段
    pub provider_id: Option<i32>,
    pub provider_name: Option<String>,
    pub cloud_platform_id: Option<i32>,
    pub cloud_platform_name: Option<String>,
    pub machine_room_id: Option<i32>,
    pub machine_room_name: Option<String>,

    // 资源配置
    pub cloud_region: Option<String>,
    pub cloud_category: Option<String>,
    pub zone_name: Option<String>,
    pub zone_cabinet: Option<String>,
    pub rack_units: i32,

    // 基本信息
    pub customer_name: Option<String>,
    pub application_name: Option<String>,
    pub application_endpoint_id: Option<i32>,
    pub application_domain: Option<String>,
    pub contract_name: Option<String>,
    pub ecs_type: Option<String>,
    pub ecs_os: Option<String>,
    pub resource_count: i32,
    pub cpu_cores: i32,
    pub memory_gb: i32,
    pub system_disk: Option<String>,
    pub system_disk_size_gb: i32,
    pub data_disk: Option<String>,
    pub expire_at: Option<String>,
    pub has_security_product: i32,
    pub security_products: Option<String>,
    pub ip_address: Option<String>,
    pub delivery_status: Option<String>,
    pub remarks: Option<String>,

    // 时间信息
    pub created_at: String,
    pub updated_at: Option<String>,
    pub created_by: String,
    pub applicant_name: Option<String>,
    pub organization_id: Option<i32>,
    pub organization_name: Option<String>,
    pub department_id: Option<i32>,
    pub department_name: Option<String>,

    // 审批信息
    pub approver: Option<String>,
    pub approve_time: Option<String>,
    pub approve_comment: Option<String>,

    // 配置信息
    pub provisioner: Option<String>,
    pub provision_time: Option<String>,
    pub provision_details: Option<String>,

    // 交付信息
    pub deliverer: Option<String>,
    pub deliver_time: Option<String>,
    pub deliver_comment: Option<String>,

    // 网络策略专用字段
    pub fw_source_zone: Option<String>,
    pub fw_source_address: Option<String>,
    pub fw_source_port: Option<String>,
    pub fw_dest_zone: Option<String>,
    pub fw_dest_address: Option<String>,
    pub fw_dest_port: Option<String>,
    pub fw_protocol: Option<String>,
    pub fw_port: Option<String>,
    pub fw_direction: Option<String>,
    pub fw_valid_until: Option<String>,
    pub fw_firewall_name: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::cloud_provider_config::Entity",
        from = "Column::ProviderId",
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
