use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, fmt};

use crate::config;

pub fn init() {
    let log_config = config::get().log();

    let env_filter = EnvFilter::new(log_config.filter_level());

    let fmt_layer = fmt::layer()
        .with_ansi(log_config.with_ansi())
        .with_level(log_config.with_level())
        .with_thread_ids(log_config.with_thread_ids())
        .with_thread_names(log_config.with_thread_names())
        .with_target(log_config.with_target())
        .with_file(log_config.with_file())
        .with_line_number(log_config.with_line());

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();
}
