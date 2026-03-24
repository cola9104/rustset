use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        add_column_if_missing(
            manager,
            ResourceTickets::ApplicantName,
            ColumnDef::new(ResourceTickets::ApplicantName)
                .string()
                .null(),
        )
        .await?;
        add_column_if_missing(
            manager,
            ResourceTickets::OrganizationId,
            ColumnDef::new(ResourceTickets::OrganizationId)
                .integer()
                .null(),
        )
        .await?;
        add_column_if_missing(
            manager,
            ResourceTickets::OrganizationName,
            ColumnDef::new(ResourceTickets::OrganizationName)
                .string()
                .null(),
        )
        .await?;
        add_column_if_missing(
            manager,
            ResourceTickets::DepartmentId,
            ColumnDef::new(ResourceTickets::DepartmentId)
                .integer()
                .null(),
        )
        .await?;
        add_column_if_missing(
            manager,
            ResourceTickets::DepartmentName,
            ColumnDef::new(ResourceTickets::DepartmentName)
                .string()
                .null(),
        )
        .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ResourceTickets::Table)
                    .drop_column(ResourceTickets::DepartmentName)
                    .drop_column(ResourceTickets::DepartmentId)
                    .drop_column(ResourceTickets::OrganizationName)
                    .drop_column(ResourceTickets::OrganizationId)
                    .drop_column(ResourceTickets::ApplicantName)
                    .to_owned(),
            )
            .await
    }
}

async fn add_column_if_missing(
    manager: &SchemaManager<'_>,
    column_name: ResourceTickets,
    column_def: &mut ColumnDef,
) -> Result<(), DbErr> {
    let table_name = ResourceTickets::Table.to_string();
    let column_name_str = column_name.to_string();

    if manager.has_column(&table_name, &column_name_str).await? {
        return Ok(());
    }

    manager
        .alter_table(
            Table::alter()
                .table(ResourceTickets::Table)
                .add_column(column_def)
                .to_owned(),
        )
        .await
}

#[derive(DeriveIden)]
enum ResourceTickets {
    Table,
    ApplicantName,
    OrganizationId,
    OrganizationName,
    DepartmentId,
    DepartmentName,
}
