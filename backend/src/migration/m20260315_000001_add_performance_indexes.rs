use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // User table indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_user_username")
                    .table(User::Table)
                    .col(User::Username)
                    .to_owned(),
            )
            .await?;

        // Cloud service indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_cloud_service_zone_id")
                    .table(CloudService::Table)
                    .col(CloudService::ZoneId)
                    .to_owned(),
            )
            .await?;

        // Cloud provider config indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_cloud_provider_config_status")
                    .table(CloudProviderConfig::Table)
                    .col(CloudProviderConfig::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_cloud_provider_config_zone_id")
                    .table(CloudProviderConfig::Table)
                    .col(CloudProviderConfig::ZoneId)
                    .to_owned(),
            )
            .await?;

        // Business resource indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_business_resource_ecs_status")
                    .table(BusinessResource::Table)
                    .col(BusinessResource::EcsStatus)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_business_resource_customer_name")
                    .table(BusinessResource::Table)
                    .col(BusinessResource::CustomerName)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_business_resource_delivery_status")
                    .table(BusinessResource::Table)
                    .col(BusinessResource::DeliveryStatus)
                    .to_owned(),
            )
            .await?;

        // Physical machine indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_physical_machine_business_resource_id")
                    .table(PhysicalMachine::Table)
                    .col(PhysicalMachine::BusinessResourceId)
                    .to_owned(),
            )
            .await?;

        // Cloud virtual machine indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_cloud_virtual_machine_business_resource_id")
                    .table(CloudVirtualMachine::Table)
                    .col(CloudVirtualMachine::BusinessResourceId)
                    .to_owned(),
            )
            .await?;

        // Quick scan result indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_quick_scan_result_task_id")
                    .table(QuickScanResult::Table)
                    .col(QuickScanResult::TaskId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name("idx_user_username").table(User::Table).to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_cloud_service_zone_id").table(CloudService::Table).to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_cloud_provider_config_status").table(CloudProviderConfig::Table).to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_cloud_provider_config_zone_id").table(CloudProviderConfig::Table).to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_business_resource_ecs_status").table(BusinessResource::Table).to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_business_resource_customer_name").table(BusinessResource::Table).to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_business_resource_delivery_status").table(BusinessResource::Table).to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_physical_machine_business_resource_id").table(PhysicalMachine::Table).to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_cloud_virtual_machine_business_resource_id").table(CloudVirtualMachine::Table).to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_quick_scan_result_task_id").table(QuickScanResult::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum User {
    Table,
    Username,
}

#[derive(DeriveIden)]
enum CloudService {
    Table,
    ZoneId,
}

#[derive(DeriveIden)]
enum CloudProviderConfig {
    Table,
    Status,
    ZoneId,
}

#[derive(DeriveIden)]
enum BusinessResource {
    Table,
    EcsStatus,
    CustomerName,
    DeliveryStatus,
}

#[derive(DeriveIden)]
enum PhysicalMachine {
    Table,
    BusinessResourceId,
}

#[derive(DeriveIden)]
enum CloudVirtualMachine {
    Table,
    BusinessResourceId,
}

#[derive(DeriveIden)]
enum QuickScanResult {
    Table,
    TaskId,
}
