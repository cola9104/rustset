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
                    .name("idx_users_username")
                    .table(Users::Table)
                    .col(Users::Username)
                    .to_owned(),
            )
            .await?;

        // Cloud service indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_cloud_services_zone_id")
                    .table(CloudServices::Table)
                    .col(CloudServices::ZoneId)
                    .to_owned(),
            )
            .await?;

        // Cloud provider config indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_cloud_provider_configs_status")
                    .table(CloudProviderConfigs::Table)
                    .col(CloudProviderConfigs::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_cloud_provider_configs_zone_id")
                    .table(CloudProviderConfigs::Table)
                    .col(CloudProviderConfigs::ZoneId)
                    .to_owned(),
            )
            .await?;

        // Business resource indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_business_resources_ecs_status")
                    .table(BusinessResources::Table)
                    .col(BusinessResources::EcsStatus)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_business_resources_customer_name")
                    .table(BusinessResources::Table)
                    .col(BusinessResources::CustomerName)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_business_resources_delivery_status")
                    .table(BusinessResources::Table)
                    .col(BusinessResources::DeliveryStatus)
                    .to_owned(),
            )
            .await?;

        // Physical machine indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_physical_machines_business_resource_id")
                    .table(PhysicalMachines::Table)
                    .col(PhysicalMachines::BusinessResourceId)
                    .to_owned(),
            )
            .await?;

        // Cloud virtual machine indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_cloud_virtual_machines_business_resource_id")
                    .table(CloudVirtualMachines::Table)
                    .col(CloudVirtualMachines::BusinessResourceId)
                    .to_owned(),
            )
            .await?;

        // Quick scan result indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_quick_scan_results_task_id")
                    .table(QuickScanResults::Table)
                    .col(QuickScanResults::TaskId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name("idx_users_username").table(Users::Table).to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_cloud_services_zone_id").table(CloudServices::Table).to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_cloud_provider_configs_status").table(CloudProviderConfigs::Table).to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_cloud_provider_configs_zone_id").table(CloudProviderConfigs::Table).to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_business_resources_ecs_status").table(BusinessResources::Table).to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_business_resources_customer_name").table(BusinessResources::Table).to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_business_resources_delivery_status").table(BusinessResources::Table).to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_physical_machines_business_resource_id").table(PhysicalMachines::Table).to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_cloud_virtual_machines_business_resource_id").table(CloudVirtualMachines::Table).to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_quick_scan_results_task_id").table(QuickScanResults::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Username,
}

#[derive(DeriveIden)]
enum CloudServices {
    Table,
    ZoneId,
}

#[derive(DeriveIden)]
enum CloudProviderConfigs {
    Table,
    Status,
    ZoneId,
}

#[derive(DeriveIden)]
enum BusinessResources {
    Table,
    EcsStatus,
    CustomerName,
    DeliveryStatus,
}

#[derive(DeriveIden)]
enum PhysicalMachines {
    Table,
    BusinessResourceId,
}

#[derive(DeriveIden)]
enum CloudVirtualMachines {
    Table,
    BusinessResourceId,
}

#[derive(DeriveIden)]
enum QuickScanResults {
    Table,
    TaskId,
}
