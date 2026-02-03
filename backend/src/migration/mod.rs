pub use sea_orm_migration::prelude::*;

mod m20250101_000001_create_tables;
mod m20250203_000002_make_resource_fields_nullable;
// TODO: Fix API compatibility issues in m20250203_000001_split_resource_tables
// mod m20250203_000001_split_resource_tables;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250101_000001_create_tables::Migration),
            Box::new(m20250203_000002_make_resource_fields_nullable::Migration),
            // Box::new(m20250203_000001_split_resource_tables::Migration),
        ]
    }
}
