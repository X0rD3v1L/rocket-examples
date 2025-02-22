use sea_orm::{Database, DatabaseConnection, ConnectOptions, DbErr, Statement, DbBackend, ConnectionTrait};
use std::time::Duration;

use crate::AppConfig;

const DEFAULT_DB: &str = "postgres"; // PostgreSQL system database

pub async fn connect(config: &AppConfig) -> Result<DatabaseConnection, DbErr> {
    // 1️⃣ Connect to the default "postgres" database for admin operations
    let admin_db_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        config.db_username, config.db_password, config.db_host, config.db_port, DEFAULT_DB
    );

    println!("🔗 Connecting to PostgreSQL admin DB: {}", admin_db_url);
    let admin_db = Database::connect(admin_db_url.clone()).await?;

    // 2️⃣ Drop and recreate the target database (`config.db_database`)
    let db_name = &config.db_database;

    admin_db
        .execute(Statement::from_string(
            DbBackend::Postgres,
            format!(r#"DROP DATABASE IF EXISTS "{}";"#, db_name),
        ))
        .await
        .map_err(|e| {
            eprintln!("❌ Failed to drop database '{}': {}", db_name, e);
            e
        })?;

    admin_db
        .execute(Statement::from_string(
            DbBackend::Postgres,
            format!(r#"CREATE DATABASE "{}";"#, db_name),
        ))
        .await
        .map_err(|e| {
            eprintln!("❌ Failed to create database '{}': {}", db_name, e);
            e
        })?;

    println!("✅ Database '{}' has been recreated.", db_name);

    // 3️⃣ Connect to the newly created database
    let user_db_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        config.db_username, config.db_password, config.db_host, config.db_port, db_name
    );

    println!("🔗 Connecting to newly created database: {}", user_db_url);

    let mut opts = ConnectOptions::new(user_db_url);
    opts.sqlx_logging(false) // Disable excessive logging
        .max_connections(10) // Limit max connections
        .min_connections(2) // Maintain min connections
        .connect_timeout(Duration::from_secs(10)) // Connection timeout
        .idle_timeout(Duration::from_secs(300)); // Idle timeout

    match Database::connect(opts).await {
        Ok(db) => {
            println!("✅ Successfully connected to database '{}'.", db_name);
            Ok(db)
        }
        Err(err) => {
            eprintln!("❌ Failed to connect to database '{}': {}", db_name, err);
            Err(err)
        }
    }
}
