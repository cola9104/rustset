use sea_orm_migration::prelude::*;

pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ResourceTicket::Table)
                    .add_column(
                        ColumnDef::new(ResourceTicket::SecurityProducts)
                            .text()
                            .null()
                    )
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ResourceTicket::Table)
                    .drop_column(ResourceTicket::SecurityProducts)
            )
            .await
    }
}

#[derive(DeriveIden)]
enum ResourceTicket {
    Table,
    SecurityProducts,
}
