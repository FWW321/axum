use axum::Router;
use axum::extract::DefaultBodyLimit;

pub fn add_body_limit<S>(router: Router<S>) -> Router<S>
where
    S: Send + Sync + Clone + 'static,
{
    // 默认限制为 10MB
    let limit = 10 * 1024 * 1024;
    router.layer(DefaultBodyLimit::max(limit))
}
