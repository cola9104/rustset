use sea_orm_migration::prelude::*;

use sea_orm_migration::schema::{pk_auto, string, integer, text, boolean, timestamp, timestamp_with_time_zone, float};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Step 1: Create physical_machines table
        manager
            .create_table(
                Table::create()
                    .table(PhysicalMachines::Table)
                    .if_not_exists()
                    .col(pk_auto(PhysicalMachines::Id))
                    .col(integer(PhysicalMachines::BusinessResourceId).not_null().unique_key())
                    .col(string(PhysicalMachines::SerialNumber)) // 设备序列号
                    .col(string(PhysicalMachines::RackLocation)) // 机架位置
                    .col(string(PhysicalMachines::HardwareModel)) // 硬件型号
                    .col(string(PhysicalMachines::WarrantyExpiry)) // 维保到期时间
                    .col(string(PhysicalMachines::AgentStatus)) // Agent 状态
                    .col(string(PhysicalMachines::IpmiAddress)) // IPMI/iDRAC 地址
                    .col(string(PhysicalMachines::CreatedAt).not_null())
                    .col(string(PhysicalMachines::UpdatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_physical_machines_business_resource_id")
                            .from(PhysicalMachines::Table, PhysicalMachines::BusinessResourceId)
                            .to(BusinessResources::Table, BusinessResources::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .to_owned(),
            )
            .await?;

        // Step 2: Create cloud_virtual_machines table
        manager
            .create_table(
                Table::create()
                    .table(CloudVirtualMachines::Table)
                    .if_not_exists()
                    .col(pk_auto(CloudVirtualMachines::Id))
                    .col(integer(CloudVirtualMachines::BusinessResourceId).not_null().unique_key())
                    .col(string(CloudVirtualMachines::BillingMode)) // 计费模式
                    .col(string(CloudVirtualMachines::ExpireTime)) // 到期时间
                    .col(string(CloudVirtualMachines::ChargeType)) // 付费类型
                    .col(string(CloudVirtualMachines::InstanceChargeType)) // 实例计费类型
                    .col(string(CloudVirtualMachines::InternetChargeType)) // 网络计费类型
                    .col(integer(CloudVirtualMachines::InternetMaxBandwidthOut)) // 公网带宽出带宽最大值
                    .col(string(CloudVirtualMachines::ImageId)) // 镜像ID
                    .col(string(CloudVirtualMachines::VSwitchId)) // 虚拟交换机ID
                    .col(string(CloudVirtualMachines::VpcId)) // VPC ID
                    .col(string(CloudVirtualMachines::SecurityGroupIds)) // 安全组ID列表 (JSON)
                    .col(string(CloudVirtualMachines::CreatedAt).not_null())
                    .col(string(CloudVirtualMachines::UpdatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_cloud_virtual_machines_business_resource_id")
                            .from(CloudVirtualMachines::Table, CloudVirtualMachines::BusinessResourceId)
                            .to(BusinessResources::Table, BusinessResources::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .to_owned(),
            )
            .await?;

        // Step 3: Migrate existing data from business_resources to physical_machines
        // First, migrate records with resource_type = "physical"
        manager
            .execute_stmt(
                sea_query::Query::insert()
                    .into_table(PhysicalMachines::Table)
                    .columns([
                        PhysicalMachines::BusinessResourceId,
                        PhysicalMachines::SerialNumber,
                        PhysicalMachines::RackLocation,
                        PhysicalMachines::HardwareModel,
                        PhysicalMachines::WarrantyExpiry,
                        PhysicalMachines::AgentStatus,
                        PhysicalMachines::IpmiAddress,
                        PhysicalMachines::CreatedAt,
                    ])
                    .query_from(
                        sea_query::Query::select()
                            .column(BusinessResources::Id)
                            .column(BusinessResources::SerialNumber)
                            .column(BusinessResources::RackLocation)
                            .column(BusinessResources::HardwareModel)
                            .column(BusinessResources::WarrantyExpiry)
                            .column(BusinessResources::AgentStatus)
                            .column(BusinessResources::IpmiAddress)
                            .column(BusinessResources::CreatedAt)
                            .from(BusinessResources::Table)
                            .and_where(Expr::col(BusinessResources::ResourceType).eq("physical"))
                            .and_where(Expr::col(BusinessResources::SerialNumber).is_not_null())
                            .to_owned(),
                    )
                    .to_owned(),
            )
            .await?;

        // Step 4: Remove physical machine specific columns from business_resources table
        // Note: SQLite doesn't support DROP COLUMN directly, so we need to recreate the table
        // For now, we'll just leave them as NULL and ignore them in the application layer
        // In a production environment with PostgreSQL/MySQL, we would use ALTER TABLE ... DROP COLUMN

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CloudVirtualMachines::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(PhysicalMachines::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum BusinessResources {
    Table,
    Id,
    ResourceType,
    SerialNumber,
    RackLocation,
    HardwareModel,
    WarrantyExpiry,
    AgentStatus,
    IpmiAddress,
    CreatedAt,
}

#[derive(DeriveIden)]
enum PhysicalMachines {
    Table,
    Id,
    BusinessResourceId,
    SerialNumber,
    RackLocation,
    HardwareModel,
    WarrantyExpiry,
    AgentStatus,
    IpmiAddress,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum CloudVirtualMachines {
    Table,
    Id,
    BusinessResourceId,
    BillingMode,
    ExpireTime,
    ChargeType,
    InstanceChargeType,
    InternetChargeType,
    InternetMaxBandwidthOut,
    ImageId,
    VSwitchId,
    VpcId,
    SecurityGroupIds,
    CreatedAt,
    UpdatedAt,
}
