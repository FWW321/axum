use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    port: Option<u16>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self { port: None }
    }
}

impl ServerConfig {
    pub fn port(&self) -> u16 {
        self.port.unwrap_or(3000)
    }
}
