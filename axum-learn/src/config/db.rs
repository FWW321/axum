use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbConfig {
    #[serde(flatten)]
    url: Url,
    max_connections: Option<u32>,
    min_connections: Option<u32>,
    connection_timeout: Option<u32>,
    acquire_timeout: Option<u32>,
    idle_timeout: Option<u32>,
    max_lifetime: Option<u32>,
    sqlx_logging: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Url {
    host: Option<String>,
    port: Option<u16>,
    user: Option<String>,
    password: Option<String>,
    database: Option<String>,
    schema: Option<String>,
}

impl Default for Url {
    fn default() -> Self {
        Self {
            host: None,
            port: None,
            user: None,
            password: None,
            database: None,
            schema: None,
        }
    }
}

impl Url {
    fn host(&self) -> &str {
        self.host.as_deref().unwrap_or("localhost")
    }

    fn port(&self) -> u16 {
        self.port.unwrap_or(5432)
    }

    fn user(&self) -> &str {
        self.user.as_deref().unwrap_or("postgres")
    }

    fn password(&self) -> &str {
        self.password.as_deref().unwrap_or("")
    }

    fn database(&self) -> &str {
        self.database.as_deref().unwrap_or("postgres")
    }

    fn schema(&self) -> &str {
        self.schema.as_deref().unwrap_or("public")
    }
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            url: Url::default(),
            max_connections: None,
            min_connections: None,
            connection_timeout: None,
            acquire_timeout: None,
            idle_timeout: None,
            max_lifetime: None,
            sqlx_logging: None,
        }
    }
}

impl DbConfig {
    pub fn url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.url.user(),
            self.url.password(),
            self.url.host(),
            self.url.port(),
            self.url.database()
        )
    }

    pub fn schema(&self) -> &str {
        self.url.schema()
    }

    pub fn max_connections(&self) -> u32 {
        let default = || {
            let cpus = num_cpus::get() as u32;
            20.max(cpus * 8)
        };
        self.max_connections.unwrap_or_else(default)
    }

    pub fn min_connections(&self) -> u32 {
        let default = || {
            let cpus = num_cpus::get() as u32;
            10.max(cpus * 4)
        };
        self.min_connections.unwrap_or_else(default)
    }

    pub fn connection_timeout(&self) -> Duration {
        Duration::from_secs(self.connection_timeout.unwrap_or(10) as u64)
    }

    pub fn acquire_timeout(&self) -> Duration {
        Duration::from_secs(self.acquire_timeout.unwrap_or(30) as u64)
    }

    pub fn idle_timeout(&self) -> Duration {
        Duration::from_secs(self.idle_timeout.unwrap_or(300) as u64)
    }

    pub fn max_lifetime(&self) -> Duration {
        Duration::from_secs(self.max_lifetime.unwrap_or(3600 * 24) as u64)
    }

    pub fn sqlx_logging(&self) -> bool {
        self.sqlx_logging.unwrap_or(false)
    }
}
