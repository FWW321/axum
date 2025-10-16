use std::time::Duration;

use axum::Router;
use tower_http::timeout::TimeoutLayer;

pub fn add_timeout<S>(router: Router<S>) -> Router<S>
where
    S: Send + Sync + Clone + 'static,
{
    let timeout = Duration::from_secs(120);
    router.layer(TimeoutLayer::new(timeout))
}
