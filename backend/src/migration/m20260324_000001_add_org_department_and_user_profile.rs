use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Organizations::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Organizations::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Organizations::Name).string().not_null())
                    .col(
                        ColumnDef::new(Organizations::Code)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Organizations::Status)
                            .string()
                            .not_null()
                            .default("active"),
                    )
                    .col(ColumnDef::new(Organizations::Remarks).string().null())
                    .col(ColumnDef::new(Organizations::CreatedAt).string().not_null())
                    .col(ColumnDef::new(Organizations::UpdatedAt).string().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Departments::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Departments::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Departments::OrganizationId)
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Departments::Name).string().not_null())
                    .col(ColumnDef::new(Departments::Code).string().not_null())
                    .col(ColumnDef::new(Departments::ParentId).integer().null())
                    .col(
                        ColumnDef::new(Departments::Level)
                            .integer()
                            .not_null()
                            .default(1),
                    )
                    .col(
                        ColumnDef::new(Departments::Status)
                            .string()
                            .not_null()
                            .default("active"),
                    )
                    .col(ColumnDef::new(Departments::Remarks).string().null())
                    .col(ColumnDef::new(Departments::CreatedAt).string().not_null())
                    .col(ColumnDef::new(Departments::UpdatedAt).string().null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_departments_organization_id")
                            .from(Departments::Table, Departments::OrganizationId)
                            .to(Organizations::Table, Organizations::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(ColumnDef::new(Users::RealName).string().null())
                    .add_column(ColumnDef::new(Users::OrganizationId).integer().null())
                    .add_column(ColumnDef::new(Users::DepartmentId).integer().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::DepartmentId)
                    .drop_column(Users::OrganizationId)
                    .drop_column(Users::RealName)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(Departments::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Organizations::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Organizations {
    Table,
    Id,
    Name,
    Code,
    Status,
    Remarks,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Departments {
    Table,
    Id,
    OrganizationId,
    Name,
    Code,
    ParentId,
    Level,
    Status,
    Remarks,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    RealName,
    OrganizationId,
    DepartmentId,
}
