use sea_orm_migration::prelude::*;

use sea_orm_migration::schema::{integer, pk_auto, string, text, timestamp};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Step 1: Create cloud_services table
        manager
            .create_table(
                Table::create()
                    .table(CloudServices::Table)
                    .if_not_exists()
                    .col(pk_auto(CloudServices::Id))
                    .col(integer(CloudServices::ZoneId).not_null())
                    .col(string(CloudServices::ServiceName).not_null())
                    .col(string(CloudServices::ServiceCode).not_null())
                    .col(text(CloudServices::Description))
                    .col(timestamp(CloudServices::CreatedAt).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_cloud_services_zone_id")
                            .from(CloudServices::Table, CloudServices::ZoneId)
                            .to(CloudZones::Table, CloudZones::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Note: Data migration will be handled by the application
        // This migration only creates the new table structure

        // Step 2: Drop old cloud_platforms table
        // Uncomment after data migration is complete
        // manager
        //     .drop_table(Table::drop().table(CloudPlatforms::Table).to_owned())
        //     .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop cloud_services table
        manager
            .drop_table(Table::drop().table(CloudServices::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum CloudServices {
    Table,
    Id,
    ZoneId,
    ServiceName,
    ServiceCode,
    Description,
    CreatedAt,
}

#[derive(DeriveIden)]
enum CloudPlatforms {
    Table,
}

#[derive(DeriveIden)]
enum CloudZones {
    Table,
    Id,
}
