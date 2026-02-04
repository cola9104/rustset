pub use sea_orm_migration::prelude::*;

mod m20250101_000001_create_tables;
mod m20250203_000002_make_resource_fields_nullable;
mod m20250203_000001_split_resource_tables;
mod m20250204_000001_add_provider_vendor;
mod m20250204_000002_rename_cloud_platforms_to_cloud_services;
mod m20250204_000003_add_regions;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250101_000001_create_tables::Migration),
            Box::new(m20250203_000002_make_resource_fields_nullable::Migration),
            Box::new(m20250203_000001_split_resource_tables::Migration),
            Box::new(m20250204_000001_add_provider_vendor::Migration),
            Box::new(m20250204_000002_rename_cloud_platforms_to_cloud_services::Migration),
            Box::new(m20250204_000003_add_regions::Migration),
        ]
    }
}
