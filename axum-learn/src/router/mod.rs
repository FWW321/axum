mod extract;
mod response;
mod pagination;
mod auth;
mod user;

pub use response::ApiResponse;
pub use pagination::{PaginationParams, Page};

use axum::Router;
use axum::routing::get;

use crate::app::AppState;
use crate::middleware;


pub fn root(state: AppState) -> Router {
    // 洋葱模型，每一层中间件都包裹前面的中间件
    // 对于request，中间件的执行顺序是从外到内
    // 对于response，中间件的执行顺序是从内到外
    // ServiceBuilder可以按中间件的声明顺序来处理request
    // 每一层中间件都可以直接中断请求并返回响应，不会经过后续的中间件处理
    // router匹配完后才会进入中间件
    // 中间件只会影响在它之内的路由，即在它之前挂载的路由
    let mut root = Router::new()
    .route("/", get(|| async { "Hello, World!" }))
    .merge(user::build_router())
    .merge(auth::build_router());

    root = middleware::add_timeout(root);
    root = middleware::add_body_limit(root);
    root = middleware::add_tracing(root);
    root = middleware::add_cors(root);
    root = middleware::add_normalize_path(root);
    root.with_state(state)
}
