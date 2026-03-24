#![allow(dead_code)]

use sea_orm_migration::prelude::*;

use sea_orm_migration::schema::string;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add provider_vendor column to business_resources table
        manager
            .alter_table(
                Table::alter()
                    .table(BusinessResources::Table)
                    .add_column(string(BusinessResources::ProviderVendor).null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop provider_vendor column from business_resources table
        manager
            .alter_table(
                Table::alter()
                    .table(BusinessResources::Table)
                    .drop_column(BusinessResources::ProviderVendor)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum BusinessResources {
    Table,
    ProviderVendor,
}
