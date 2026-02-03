use sea_orm_migration::prelude::*;
use sea_orm::{ConnectionTrait, Statement, DbBackend};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Step 1: Create new table with nullable resource_id and instance_id
        db.execute(Statement::from_string(
            DbBackend::Sqlite,
            r#"
            CREATE TABLE IF NOT EXISTS business_resources_new (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                resource_type VARCHAR NOT NULL,
                ecs_name VARCHAR NOT NULL,
                ecs_status VARCHAR NOT NULL,
                resource_id VARCHAR,
                cloud_region VARCHAR NOT NULL,
                cloud_category VARCHAR NOT NULL,
                cloud_provider_config_id INTEGER,
                zone_name VARCHAR,
                platform_name VARCHAR,
                county_city VARCHAR,
                vdc_name VARCHAR,
                customer_name VARCHAR NOT NULL,
                application_name VARCHAR,
                contract_name VARCHAR,
                instance_id VARCHAR,
                ecs_type VARCHAR NOT NULL,
                ecs_os VARCHAR NOT NULL,
                cpu_cores INTEGER NOT NULL,
                memory_gb INTEGER NOT NULL,
                system_disk VARCHAR NOT NULL,
                system_disk_size_gb INTEGER NOT NULL,
                data_disk VARCHAR,
                completion_time VARCHAR,
                release_time VARCHAR,
                has_security_product INTEGER NOT NULL,
                ip_address VARCHAR NOT NULL,
                ecs_login_method VARCHAR,
                ecs_login_username VARCHAR,
                ecs_initial_password VARCHAR,
                bastion_address VARCHAR,
                bastion_admin_account VARCHAR,
                bastion_initial_password VARCHAR,
                serial_number VARCHAR,
                rack_location VARCHAR,
                hardware_model VARCHAR,
                warranty_expiry VARCHAR,
                agent_status VARCHAR,
                ipmi_address VARCHAR,
                remarks VARCHAR,
                created_at VARCHAR NOT NULL,
                updated_at VARCHAR,
                created_by VARCHAR,
                updated_by VARCHAR,
                FOREIGN KEY (cloud_provider_config_id) REFERENCES cloud_provider_configs(id) ON DELETE SET NULL
            )
            "#.to_string(),
        )).await?;

        // Step 2: Copy data from old table to new table
        db.execute(Statement::from_string(
            DbBackend::Sqlite,
            r#"
            INSERT INTO business_resources_new 
            SELECT * FROM business_resources
            "#.to_string(),
        )).await?;

        // Step 3: Drop old table
        db.execute(Statement::from_string(
            DbBackend::Sqlite,
            r#"
            DROP TABLE IF EXISTS business_resources
            "#.to_string(),
        )).await?;

        // Step 4: Rename new table
        db.execute(Statement::from_string(
            DbBackend::Sqlite,
            r#"
            ALTER TABLE business_resources_new RENAME TO business_resources
            "#.to_string(),
        )).await?;

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
