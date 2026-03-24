#![allow(dead_code)]

use sea_orm_migration::prelude::*;

// Import schema helper functions
use sea_orm_migration::schema::{boolean, float, integer, pk_auto, string, text};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create cloud_zones table
        manager
            .create_table(
                Table::create()
                    .table(CloudZones::Table)
                    .if_not_exists()
                    .col(pk_auto(CloudZones::Id))
                    .col(string(CloudZones::ZoneName).not_null())
                    .col(string(CloudZones::ZoneCode).not_null().unique_key())
                    .col(string(CloudZones::Description))
                    .col(string(CloudZones::CreatedAt).not_null())
                    .to_owned(),
            )
            .await?;

        // Create cloud_platforms table
        manager
            .create_table(
                Table::create()
                    .table(CloudPlatforms::Table)
                    .if_not_exists()
                    .col(pk_auto(CloudPlatforms::Id))
                    .col(integer(CloudPlatforms::ZoneId).not_null())
                    .col(string(CloudPlatforms::PlatformName).not_null())
                    .col(string(CloudPlatforms::PlatformCode).not_null())
                    .col(string(CloudPlatforms::Description))
                    .col(string(CloudPlatforms::CreatedAt).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_cloud_platforms_zone_id")
                            .from(CloudPlatforms::Table, CloudPlatforms::ZoneId)
                            .to(CloudZones::Table, CloudZones::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create cloud_provider_configs table
        manager
            .create_table(
                Table::create()
                    .table(CloudProviderConfigs::Table)
                    .if_not_exists()
                    .col(pk_auto(CloudProviderConfigs::Id))
                    .col(integer(CloudProviderConfigs::ZoneId).not_null())
                    .col(integer(CloudProviderConfigs::PlatformId).not_null())
                    .col(string(CloudProviderConfigs::Provider).not_null())
                    .col(string(CloudProviderConfigs::RegionId).not_null())
                    .col(string(CloudProviderConfigs::RegionName).not_null())
                    .col(string(CloudProviderConfigs::AccountName).not_null())
                    .col(string(CloudProviderConfigs::AccessKeyId).not_null())
                    .col(string(CloudProviderConfigs::AccessKeySecret).not_null())
                    .col(string(CloudProviderConfigs::Remarks))
                    .col(string(CloudProviderConfigs::Status).default("active"))
                    .col(string(CloudProviderConfigs::LastTestTime))
                    .col(string(CloudProviderConfigs::LastTestResult))
                    .col(string(CloudProviderConfigs::CreatedAt).not_null())
                    .col(string(CloudProviderConfigs::UpdatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_provider_configs_zone_id")
                            .from(CloudProviderConfigs::Table, CloudProviderConfigs::ZoneId)
                            .to(CloudZones::Table, CloudZones::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_provider_configs_platform_id")
                            .from(
                                CloudProviderConfigs::Table,
                                CloudProviderConfigs::PlatformId,
                            )
                            .to(CloudPlatforms::Table, CloudPlatforms::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create business_resources table
        manager
            .create_table(
                Table::create()
                    .table(BusinessResources::Table)
                    .if_not_exists()
                    .col(pk_auto(BusinessResources::Id))
                    .col(string(BusinessResources::ResourceType).not_null())
                    .col(string(BusinessResources::EcsName).not_null())
                    .col(string(BusinessResources::EcsStatus).not_null())
                    .col(string(BusinessResources::ResourceId).not_null())
                    .col(string(BusinessResources::CloudRegion).not_null())
                    .col(string(BusinessResources::CloudCategory).not_null())
                    .col(integer(BusinessResources::CloudProviderConfigId))
                    .col(string(BusinessResources::ZoneName))
                    .col(string(BusinessResources::PlatformName))
                    .col(string(BusinessResources::CountyCity))
                    .col(string(BusinessResources::VdcName))
                    .col(string(BusinessResources::CustomerName).not_null())
                    .col(string(BusinessResources::ApplicationName))
                    .col(string(BusinessResources::ContractName))
                    .col(string(BusinessResources::InstanceId).not_null())
                    .col(string(BusinessResources::EcsType).not_null())
                    .col(string(BusinessResources::EcsOs).not_null())
                    .col(integer(BusinessResources::CpuCores).not_null())
                    .col(integer(BusinessResources::MemoryGb).not_null())
                    .col(string(BusinessResources::SystemDisk).not_null())
                    .col(integer(BusinessResources::SystemDiskSizeGb).not_null())
                    .col(string(BusinessResources::DataDisk))
                    .col(string(BusinessResources::CompletionTime))
                    .col(string(BusinessResources::ReleaseTime))
                    .col(integer(BusinessResources::HasSecurityProduct).not_null())
                    .col(string(BusinessResources::IpAddress).not_null())
                    .col(string(BusinessResources::EcsLoginMethod))
                    .col(string(BusinessResources::EcsLoginUsername))
                    .col(string(BusinessResources::EcsInitialPassword))
                    .col(string(BusinessResources::BastionAddress))
                    .col(string(BusinessResources::BastionAdminAccount))
                    .col(string(BusinessResources::BastionInitialPassword))
                    .col(string(BusinessResources::SerialNumber))
                    .col(string(BusinessResources::RackLocation))
                    .col(string(BusinessResources::HardwareModel))
                    .col(string(BusinessResources::WarrantyExpiry))
                    .col(string(BusinessResources::AgentStatus))
                    .col(string(BusinessResources::IpmiAddress))
                    .col(string(BusinessResources::Remarks))
                    .col(string(BusinessResources::CreatedAt).not_null())
                    .col(string(BusinessResources::UpdatedAt))
                    .col(string(BusinessResources::CreatedBy))
                    .col(string(BusinessResources::UpdatedBy))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_business_resources_provider_config_id")
                            .from(
                                BusinessResources::Table,
                                BusinessResources::CloudProviderConfigId,
                            )
                            .to(CloudProviderConfigs::Table, CloudProviderConfigs::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        // Create users table
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(string(Users::Id).primary_key())
                    .col(string(Users::Username).not_null().unique_key())
                    .col(string(Users::Password).not_null())
                    .col(string(Users::Role).not_null())
                    .col(string(Users::Permissions))
                    .col(string(Users::CreatedAt).not_null())
                    .col(string(Users::PasswordChangedAt))
                    .col(string(Users::PasswordStrength))
                    .col(integer(Users::ForcePasswordChange).default(0))
                    .col(string(Users::LastLoginAt))
                    .col(string(Users::Email))
                    .col(string(Users::Phone))
                    .col(string(Users::Status))
                    .col(integer(Users::FailedLoginAttempts).default(0))
                    .col(string(Users::LockedUntil))
                    .to_owned(),
            )
            .await?;

        // Create audit_logs table
        manager
            .create_table(
                Table::create()
                    .table(AuditLogs::Table)
                    .if_not_exists()
                    .col(string(AuditLogs::Id).primary_key())
                    .col(string(AuditLogs::UserId).not_null())
                    .col(string(AuditLogs::Username).not_null())
                    .col(string(AuditLogs::Action).not_null())
                    .col(string(AuditLogs::Target).not_null())
                    .col(string(AuditLogs::Details).not_null())
                    .col(string(AuditLogs::Timestamp).not_null())
                    .to_owned(),
            )
            .await?;

        // Create assets table
        manager
            .create_table(
                Table::create()
                    .table(Assets::Table)
                    .if_not_exists()
                    .col(pk_auto(Assets::Id))
                    .col(string(Assets::Name).not_null())
                    .col(string(Assets::Ip).not_null())
                    .col(string(Assets::Zone).not_null())
                    .col(string(Assets::Ports).not_null()) // JSON string
                    .col(string(Assets::LastScanned))
                    .col(string(Assets::ContactPerson))
                    .col(string(Assets::ContactPhone))
                    .col(string(Assets::CreatedBy))
                    .col(string(Assets::UpdatedBy))
                    .col(string(Assets::Owner))
                    .col(integer(Assets::Weight).default(50))
                    .col(string(Assets::Labels)) // JSON string
                    .col(string(Assets::Os))
                    .col(string(Assets::DeviceType))
                    .to_owned(),
            )
            .await?;

        // Create tasks table
        manager
            .create_table(
                Table::create()
                    .table(Tasks::Table)
                    .if_not_exists()
                    .col(string(Tasks::Id).primary_key())
                    .col(string(Tasks::Name).not_null())
                    .col(string(Tasks::Target).not_null())
                    .col(string(Tasks::Status).not_null())
                    .col(string(Tasks::StartTime))
                    .col(string(Tasks::EndTime))
                    .col(integer(Tasks::FoundAssets).default(0))
                    .col(integer(Tasks::FoundRisks).default(0))
                    .col(string(Tasks::PortPolicy).not_null())
                    .col(integer(Tasks::DomainBrute).default(0))
                    .col(integer(Tasks::ServiceDetection).default(0))
                    .col(integer(Tasks::OsDetection).default(0))
                    .col(integer(Tasks::SiteIdentify).default(0))
                    .col(string(Tasks::CreatedBy))
                    .to_owned(),
            )
            .await?;

        // Create risks table
        manager
            .create_table(
                Table::create()
                    .table(Risks::Table)
                    .if_not_exists()
                    .col(string(Risks::Id).primary_key())
                    .col(string(Risks::AssetIp).not_null())
                    .col(integer(Risks::Port).not_null())
                    .col(string(Risks::Severity).not_null())
                    .col(string(Risks::Description).not_null())
                    .col(string(Risks::Solution))
                    .col(string(Risks::Status).not_null())
                    .col(string(Risks::CreatedAt))
                    .col(string(Risks::UpdatedAt))
                    .col(string(Risks::AssignedTo))
                    .to_owned(),
            )
            .await?;

        // Create network_zones table
        manager
            .create_table(
                Table::create()
                    .table(NetworkZones::Table)
                    .if_not_exists()
                    .col(string(NetworkZones::Id).primary_key())
                    .col(string(NetworkZones::Name).not_null())
                    .col(string(NetworkZones::Cidr).not_null())
                    .col(integer(NetworkZones::Priority).not_null())
                    .col(integer(NetworkZones::CloudPlatformId))
                    .col(string(NetworkZones::CloudPlatformName))
                    .col(integer(NetworkZones::MachineRoomId))
                    .col(string(NetworkZones::MachineRoomName))
                    .to_owned(),
            )
            .await?;

        // Create custom_roles table
        manager
            .create_table(
                Table::create()
                    .table(CustomRoles::Table)
                    .if_not_exists()
                    .col(pk_auto(CustomRoles::Id))
                    .col(string(CustomRoles::Name).not_null().unique_key())
                    .col(string(CustomRoles::Description))
                    .col(text(CustomRoles::Permissions).not_null()) // JSON string
                    .col(string(CustomRoles::CreatedAt))
                    .col(string(CustomRoles::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        // Create advanced_scan_tasks table
        manager
            .create_table(
                Table::create()
                    .table(AdvancedScanTasks::Table)
                    .if_not_exists()
                    .col(string(AdvancedScanTasks::Id).primary_key())
                    .col(string(AdvancedScanTasks::Name).not_null())
                    .col(text(AdvancedScanTasks::Targets).not_null()) // JSON array
                    .col(text(AdvancedScanTasks::Config).not_null()) // JSON
                    .col(string(AdvancedScanTasks::Status).not_null())
                    .col(float(AdvancedScanTasks::Progress).default(0.0))
                    .col(string(AdvancedScanTasks::CurrentTarget))
                    .col(integer(AdvancedScanTasks::ScannedCount).default(0))
                    .col(integer(AdvancedScanTasks::TotalCount).default(0))
                    .col(string(AdvancedScanTasks::StartTime))
                    .col(string(AdvancedScanTasks::EndTime))
                    .col(string(AdvancedScanTasks::CreatedBy))
                    .col(text(AdvancedScanTasks::ErrorMessage))
                    .col(string(AdvancedScanTasks::CreatedAt))
                    .col(string(AdvancedScanTasks::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        // Create quick_scan_results table
        manager
            .create_table(
                Table::create()
                    .table(QuickScanResults::Table)
                    .if_not_exists()
                    .col(string(QuickScanResults::Id).primary_key())
                    .col(string(QuickScanResults::TaskId).not_null())
                    .col(string(QuickScanResults::Ip).not_null())
                    .col(boolean(QuickScanResults::IsAlive).not_null())
                    .col(text(QuickScanResults::OpenPorts)) // JSON array
                    .col(string(QuickScanResults::Fingerprint))
                    .col(string(QuickScanResults::ScannedAt).not_null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(QuickScanResults::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(AdvancedScanTasks::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(CustomRoles::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(NetworkZones::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Risks::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Tasks::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Assets::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(AuditLogs::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(BusinessResources::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(CloudProviderConfigs::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(CloudPlatforms::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(CloudZones::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum CloudZones {
    Table,
    Id,
    ZoneName,
    ZoneCode,
    Description,
    CreatedAt,
}

#[derive(DeriveIden)]
enum CloudPlatforms {
    Table,
    Id,
    ZoneId,
    PlatformName,
    PlatformCode,
    Description,
    CreatedAt,
}

#[derive(DeriveIden)]
enum CloudProviderConfigs {
    Table,
    Id,
    ZoneId,
    PlatformId,
    Provider,
    RegionId,
    RegionName,
    AccountName,
    AccessKeyId,
    AccessKeySecret,
    Remarks,
    Status,
    LastTestTime,
    LastTestResult,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub enum BusinessResources {
    Table,
    Id,
    ResourceType,
    EcsName,
    EcsStatus,
    ResourceId,
    CloudRegion,
    CloudCategory,
    CloudProviderConfigId,
    ZoneName,
    PlatformName,
    CountyCity,
    VdcName,
    CustomerName,
    ApplicationName,
    ContractName,
    InstanceId,
    EcsType,
    EcsOs,
    CpuCores,
    MemoryGb,
    SystemDisk,
    SystemDiskSizeGb,
    DataDisk,
    CompletionTime,
    ReleaseTime,
    HasSecurityProduct,
    IpAddress,
    EcsLoginMethod,
    EcsLoginUsername,
    EcsInitialPassword,
    BastionAddress,
    BastionAdminAccount,
    BastionInitialPassword,
    SerialNumber,
    RackLocation,
    HardwareModel,
    WarrantyExpiry,
    AgentStatus,
    IpmiAddress,
    Remarks,
    CreatedAt,
    UpdatedAt,
    CreatedBy,
    UpdatedBy,
    // 申请与交付状态管理
    ApplicationStatus,
    DeliveryStatus,
    DeliveryConfirmedAt,
    DeliveryConfirmedBy,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    Username,
    Password,
    Role,
    Permissions,
    CreatedAt,
    PasswordChangedAt,
    PasswordStrength,
    ForcePasswordChange,
    LastLoginAt,
    Email,
    Phone,
    Status,
    FailedLoginAttempts,
    LockedUntil,
}

#[derive(DeriveIden)]
enum AuditLogs {
    Table,
    Id,
    UserId,
    Username,
    Action,
    Target,
    Details,
    Timestamp,
}

#[derive(DeriveIden)]
enum Assets {
    Table,
    Id,
    Name,
    Ip,
    Zone,
    Ports,
    LastScanned,
    ContactPerson,
    ContactPhone,
    CreatedBy,
    UpdatedBy,
    Owner,
    Weight,
    Labels,
    Os,
    DeviceType,
}

#[derive(DeriveIden)]
enum Tasks {
    Table,
    Id,
    Name,
    Target,
    Status,
    StartTime,
    EndTime,
    FoundAssets,
    FoundRisks,
    PortPolicy,
    DomainBrute,
    ServiceDetection,
    OsDetection,
    SiteIdentify,
    CreatedBy,
}

#[derive(DeriveIden)]
enum Risks {
    Table,
    Id,
    AssetIp,
    Port,
    Severity,
    Description,
    Solution,
    Status,
    CreatedAt,
    UpdatedAt,
    AssignedTo,
}

#[derive(DeriveIden)]
enum NetworkZones {
    Table,
    Id,
    Name,
    Cidr,
    Priority,
    CloudPlatformId,
    CloudPlatformName,
    MachineRoomId,
    MachineRoomName,
}

#[derive(DeriveIden)]
enum CustomRoles {
    Table,
    Id,
    Name,
    Description,
    Permissions,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum AdvancedScanTasks {
    Table,
    Id,
    Name,
    Targets,
    Config,
    Status,
    Progress,
    CurrentTarget,
    ScannedCount,
    TotalCount,
    StartTime,
    EndTime,
    CreatedBy,
    ErrorMessage,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum QuickScanResults {
    Table,
    Id,
    TaskId,
    Ip,
    IsAlive,
    OpenPorts,
    Fingerprint,
    ScannedAt,
}
