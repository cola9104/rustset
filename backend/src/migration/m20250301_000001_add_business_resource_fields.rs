use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 申请流程相关字段
        manager
            .alter_table(
                Table::alter()
                    .table(BusinessResources::Table)
                    .add_column(ColumnDef::new(BusinessResources::Applicant).string().null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(BusinessResources::Table)
                    .add_column(
                        ColumnDef::new(BusinessResources::Department)
                            .string()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(BusinessResources::Table)
                    .add_column(ColumnDef::new(BusinessResources::Approver).string().null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(BusinessResources::Table)
                    .add_column(
                        ColumnDef::new(BusinessResources::ApprovalTime)
                            .string()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(BusinessResources::Table)
                    .add_column(
                        ColumnDef::new(BusinessResources::ApprovalRemarks)
                            .string()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(BusinessResources::Table)
                    .add_column(
                        ColumnDef::new(BusinessResources::RejectionReason)
                            .string()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // 资源配置相关字段
        manager
            .alter_table(
                Table::alter()
                    .table(BusinessResources::Table)
                    .add_column(
                        ColumnDef::new(BusinessResources::BandwidthMbps)
                            .integer()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(BusinessResources::Table)
                    .add_column(
                        ColumnDef::new(BusinessResources::BandwidthType)
                            .string()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(BusinessResources::Table)
                    .add_column(
                        ColumnDef::new(BusinessResources::PublicIpCount)
                            .integer()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(BusinessResources::Table)
                    .add_column(
                        ColumnDef::new(BusinessResources::NetworkType)
                            .string()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // 业务关联相关字段
        manager
            .alter_table(
                Table::alter()
                    .table(BusinessResources::Table)
                    .add_column(
                        ColumnDef::new(BusinessResources::ProjectName)
                            .string()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(BusinessResources::Table)
                    .add_column(
                        ColumnDef::new(BusinessResources::ProjectCode)
                            .string()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(BusinessResources::Table)
                    .add_column(
                        ColumnDef::new(BusinessResources::BusinessOwner)
                            .string()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(BusinessResources::Table)
                    .add_column(ColumnDef::new(BusinessResources::TechOwner).string().null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(BusinessResources::Table)
                    .add_column(
                        ColumnDef::new(BusinessResources::ContactPhone)
                            .string()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // 费用相关字段
        manager
            .alter_table(
                Table::alter()
                    .table(BusinessResources::Table)
                    .add_column(
                        ColumnDef::new(BusinessResources::BillingMethod)
                            .string()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(BusinessResources::Table)
                    .add_column(
                        ColumnDef::new(BusinessResources::PurchaseDuration)
                            .integer()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(BusinessResources::Table)
                    .add_column(
                        ColumnDef::new(BusinessResources::CostCenter)
                            .string()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // 合规相关字段
        manager
            .alter_table(
                Table::alter()
                    .table(BusinessResources::Table)
                    .add_column(
                        ColumnDef::new(BusinessResources::SecurityLevel)
                            .string()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(BusinessResources::Table)
                    .add_column(
                        ColumnDef::new(BusinessResources::DataSensitivity)
                            .string()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // 其他字段
        manager
            .alter_table(
                Table::alter()
                    .table(BusinessResources::Table)
                    .add_column(ColumnDef::new(BusinessResources::Purpose).text().null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(BusinessResources::Table)
                    .add_column(
                        ColumnDef::new(BusinessResources::ExpectedDeliveryTime)
                            .string()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 删除添加的字段
        let columns = [
            BusinessResources::Applicant,
            BusinessResources::Department,
            BusinessResources::Approver,
            BusinessResources::ApprovalTime,
            BusinessResources::ApprovalRemarks,
            BusinessResources::RejectionReason,
            BusinessResources::BandwidthMbps,
            BusinessResources::BandwidthType,
            BusinessResources::PublicIpCount,
            BusinessResources::NetworkType,
            BusinessResources::ProjectName,
            BusinessResources::ProjectCode,
            BusinessResources::BusinessOwner,
            BusinessResources::TechOwner,
            BusinessResources::ContactPhone,
            BusinessResources::BillingMethod,
            BusinessResources::PurchaseDuration,
            BusinessResources::CostCenter,
            BusinessResources::SecurityLevel,
            BusinessResources::DataSensitivity,
            BusinessResources::Purpose,
            BusinessResources::ExpectedDeliveryTime,
        ];

        for col in columns {
            manager
                .alter_table(
                    Table::alter()
                        .table(BusinessResources::Table)
                        .drop_column(col)
                        .to_owned(),
                )
                .await?;
        }

        Ok(())
    }
}

#[derive(DeriveIden)]
enum BusinessResources {
    Table,
    // 申请流程相关
    Applicant,
    Department,
    Approver,
    ApprovalTime,
    ApprovalRemarks,
    RejectionReason,
    // 资源配置相关
    BandwidthMbps,
    BandwidthType,
    PublicIpCount,
    NetworkType,
    // 业务关联相关
    ProjectName,
    ProjectCode,
    BusinessOwner,
    TechOwner,
    ContactPhone,
    // 费用相关
    BillingMethod,
    PurchaseDuration,
    CostCenter,
    // 合规相关
    SecurityLevel,
    DataSensitivity,
    // 其他
    Purpose,
    ExpectedDeliveryTime,
}
