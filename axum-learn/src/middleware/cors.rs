use std::time::Duration;

use axum::Router;
use tower_http::cors::{Any, CorsLayer};

pub fn add_cors<S>(router: Router<S>) -> Router<S>
where
    S: Send + Sync + Clone + 'static,
{
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_credentials(false)
        .max_age(Duration::from_secs(3600 * 12)); // 24 hours

    router.layer(cors)
}
