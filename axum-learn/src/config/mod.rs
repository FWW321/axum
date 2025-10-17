mod db;
mod jwt;
mod log;
mod server;

use std::sync::LazyLock;

use anyhow::{Context, Result};
use config::{Config, FileFormat};
use serde::Deserialize;

use db::DbConfig;
use jwt::JwtConfig;
use log::LogConfig;
pub use server::ServerConfig;

pub static CONFIG: LazyLock<AppConfig> =
    LazyLock::new(|| AppConfig::load().expect("Failed to initialize configuration"));

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    #[serde(default = "ServerConfig::default")]
    pub server: ServerConfig,
    #[serde(default = "LogConfig::default")]
    pub log: LogConfig,
    #[serde(default = "DbConfig::default")]
    pub db: DbConfig,
    #[serde(default = "JwtConfig::default")]
    pub jwt: JwtConfig,
}

impl AppConfig {
    fn load() -> Result<Self> {
        Config::builder()
            .add_source(
                config::File::with_name("config")
                    .format(FileFormat::Toml)
                    .required(true),
            )
            .add_source(
                config::Environment::with_prefix("APP")
                    .try_parsing(true)
                    .separator("_")
                    .list_separator(","),
            )
            .build()
            .with_context(|| anyhow::anyhow!("Failed to load config"))?
            .try_deserialize()
            .with_context(|| anyhow::anyhow!("Failed to deserialize config"))
    }

    pub fn server(&self) -> &ServerConfig {
        &self.server
    }

    pub fn log(&self) -> &LogConfig {
        &self.log
    }

    pub fn db(&self) -> &DbConfig {
        &self.db
    }

    pub fn jwt(&self) -> &JwtConfig {
        &self.jwt
    }
}

pub fn get() -> &'static AppConfig {
    &CONFIG
}
