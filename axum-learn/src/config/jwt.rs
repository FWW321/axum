use std::time::Duration;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct JwtConfig {
    secret: Option<String>,
    expiration: Option<u64>,
    audience: Option<String>,
    issuer: Option<String>,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: None,
            expiration: None,
            audience: None,
            issuer: None,
        }
    }
}

impl JwtConfig {
    pub fn secret(&self) -> &str {
        self.secret.as_deref().unwrap_or("secret")
    }

    pub fn expiration(&self) -> Duration {
        Duration::from_secs(self.expiration.unwrap_or(3600))
    }

    pub fn audience(&self) -> &str {
        self.audience.as_deref().unwrap_or("myapp")
    }

    pub fn issuer(&self) -> &str {
        self.issuer.as_deref().unwrap_or("myapp")
    }
}
