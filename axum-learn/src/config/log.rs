use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct LogConfig {
    filter_level: Option<String>,
    with_ansi: Option<bool>,
    with_level: Option<bool>,
    with_thread_ids: Option<bool>,
    with_thread_names: Option<bool>,
    with_target: Option<bool>,
    with_file: Option<bool>,
    with_line: Option<bool>,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            filter_level: None,
            with_ansi: None,
            with_level: None,
            with_thread_ids: None,
            with_thread_names: None,
            with_target: None,
            with_file: None,
            with_line: None,
        }
    }
}

impl LogConfig {
    pub fn filter_level(&self) -> &str {
        self.filter_level.as_deref().unwrap_or("info")
    }

    pub fn with_ansi(&self) -> bool {
        self.with_ansi.unwrap_or(true)
    }

    pub fn with_level(&self) -> bool {
        self.with_level.unwrap_or(true)
    }

    pub fn with_thread_ids(&self) -> bool {
        self.with_thread_ids.unwrap_or(true)
    }

    pub fn with_thread_names(&self) -> bool {
        self.with_thread_names.unwrap_or(true)
    }

    pub fn with_target(&self) -> bool {
        self.with_target.unwrap_or(false)
    }

    pub fn with_file(&self) -> bool {
        self.with_file.unwrap_or(true)
    }

    pub fn with_line(&self) -> bool {
        self.with_line.unwrap_or(true)
    }
}
