use sea_orm_migration::prelude::*;

use sea_orm_migration::schema::{pk_auto, string, text, timestamp};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create regions table (地区)
        manager
            .create_table(
                Table::create()
                    .table(Regions::Table)
                    .if_not_exists()
                    .col(pk_auto(Regions::Id))
                    .col(string(Regions::RegionName).not_null()) // 地区名称（如：华北、华南、华东）
                    .col(string(Regions::RegionCode).not_null()) // 地区代码（如：north、south、east）
                    .col(text(Regions::Description)) // 描述
                    .col(timestamp(Regions::CreatedAt).not_null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop regions table
        manager
            .drop_table(Table::drop().table(Regions::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Regions {
    Table,
    Id,
    RegionName,
    RegionCode,
    Description,
    CreatedAt,
}
