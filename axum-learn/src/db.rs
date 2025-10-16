use anyhow::Result;
use sea_orm::Statement;
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbBackend};

use crate::config;

pub async fn init() -> Result<DatabaseConnection> {
    let db_config = config::get().db();
    let mut opt = ConnectOptions::new(db_config.url());
    opt.max_connections(db_config.max_connections())
        .min_connections(db_config.min_connections())
        .connect_timeout(db_config.connection_timeout())
        .acquire_timeout(db_config.acquire_timeout())
        .idle_timeout(db_config.idle_timeout())
        .max_lifetime(db_config.max_lifetime())
        .sqlx_logging(db_config.sqlx_logging())
        .set_schema_search_path(db_config.schema());
    tracing::info!("Connecting to database...");
    let db = Database::connect(opt).await?;
    db.ping().await?;
    tracing::info!("Database connected successfully");
    log_version(&db).await?;
    Ok(db)
}

async fn log_version(db: &DatabaseConnection) -> Result<()> {
    let version = db
        .query_one(Statement::from_string(
            DbBackend::Postgres,
            "SELECT version()".to_owned(),
        ))
        .await?
        .ok_or_else(|| anyhow::anyhow!("Failed to get database version"))?;
    tracing::info!(
        "Database version: {}",
        version.try_get_by_index::<String>(0)?
    );
    Ok(())
}
