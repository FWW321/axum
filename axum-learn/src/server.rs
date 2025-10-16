use std::net::SocketAddr;

use anyhow::Result;
use axum::routing::Router;
use tokio::net::TcpListener;

use crate::config::ServerConfig;

pub struct Server {
    config: &'static ServerConfig,
}

impl Server {
    pub fn new(config: &'static ServerConfig) -> Self {
        Self { config }
    }

    pub async fn start(&self, router: Router) -> Result<()> {
        let port = self.config.port();
        let listener = TcpListener::bind(format!("0.0.0.0:{port}")).await?;
        tracing::info!("Listening on {port}");
        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await?;
        Ok(())
    }
}
