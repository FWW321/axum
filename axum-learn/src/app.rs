use anyhow::Result;
use sea_orm::DatabaseConnection;

use crate::config;
use crate::db;
use crate::log;
use crate::router;
use crate::server::Server;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
}

impl AppState {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

pub async fn run() -> Result<()> {
    log::init();
    tracing::info!("Starting application...");
    let db = db::init().await?;
    let state = AppState::new(db);
    let router = router::root(state);
    let server = Server::new(config::get().server());
    server.start(router).await
}
