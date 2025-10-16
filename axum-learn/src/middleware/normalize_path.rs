use axum::Router;
use tower_http::normalize_path::NormalizePathLayer;

pub fn add_normalize_path<S>(router: Router<S>) -> Router<S>
where
    S: Send + Sync + Clone + 'static,
{
    router.layer(NormalizePathLayer::trim_trailing_slash())
}
