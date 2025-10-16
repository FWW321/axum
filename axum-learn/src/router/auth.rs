use axum::{
    debug_handler,
    extract::Extension,
    routing::{get, post},
    Router,
};
use sea_orm::{prelude::*, Condition};
use serde::{Deserialize, Serialize};
use validator::Validate;
use macros::handler;

use crate::{
    app::AppState,
    entity::{prelude::*, user},
    error::ApiError,
    middleware::{
        auth::{get_jwt, Principal},
        get_auth_layer,
    },
    router::{
        extract::ValidJson,
        ApiResponse,
    },
    util,
};


pub fn build_router() -> Router<AppState> {
    let router = Router::new()
        .route("/info", get(info))
        .route_layer(get_auth_layer())
        .route("/login", post(login));

    Router::new().nest("/auth", router)
}

#[derive(Debug, Deserialize, Validate)]
struct LoginInData {
    #[validate(length(min = 3, max = 30, message = "account must be between 3 and 30 characters"))]
    account: String,
    #[validate(length(min = 6, max = 100, message = "password must be between 6 and 100 characters"))]
    password: String,
}

#[derive(Debug, Serialize)]
struct LoginResult {
    access_token: String,
}

#[handler(connect_info = true)]
#[debug_handler]
#[tracing::instrument(skip_all, fields(account = %idata.0.account, ip = %addr.ip()))]
async fn login(idata: ValidJson<LoginInData>) -> LoginResult {
    let LoginInData { account, password } = idata.0;
    tracing::info!("logging in...");
    let user = User::find()
        .filter(
            Condition::any()
                .add(user::Column::Username.eq(&account))
                .add(user::Column::Email.eq(&account))
        )
        .one(&db)
        .await?
        // 密码需要hash验证，耗时比较久，可以通过耗时来判断是账号还是密码错误
        // 安全性要求比较高可以模拟密码验证，使两者耗时相同
        .ok_or_else(|| ApiError::Biz("invalid account or password".to_owned()))?;

    match util::verify_password(&password, &user.password) {
        Ok(_) => {},
        _ => return Err(ApiError::Biz("invalid account or password".to_owned())),
    }

    let access_token = get_jwt().encode(Principal {
        id: user.id,
        name: user.username,
    })?;
    Ok(ApiResponse::ok("Login successful".to_owned(), Some(LoginResult { access_token })))
}

#[handler]
#[debug_handler]
async fn info(Extension(principal): Extension<Principal>) -> user::Model {
    let user = User::find_by_id(principal.id)
        .one(&db)
        .await?
        .ok_or_else(|| ApiError::NotFound)?;
    Ok(ApiResponse::ok("User info retrieved successfully".to_owned(), Some(user)))
}