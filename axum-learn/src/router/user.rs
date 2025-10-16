use axum::debug_handler;
use axum::Router;
use axum::extract::State;
use axum::routing::{post, get, put, delete};
use sea_orm::prelude::*;
use sea_orm::Condition;
use sea_orm::QueryOrder;
use sea_orm::QueryTrait;
use serde::Deserialize;
use chrono::NaiveDate;
use validator::Validate;
use sea_orm::{ActiveValue, ActiveModelTrait};
use validator::ValidationError;

use super::ApiResponse;
use super::extract::{ValidJson, Path};
use crate::app::AppState;
use crate::error::ApiResult;
use crate::entity::user;
use crate::entity::prelude::*;
use crate::middleware::get_auth_layer;
use crate::router::extract::ValidQuery;
use crate::router::{Page, PaginationParams};
use crate::util;
use crate::error::ApiError;
use macros::handler;

pub fn build_router() -> Router<AppState> {
    let router = Router::new()
        .route("/", get(list))
        .route("/{id}", put(update))
        .route_layer(get_auth_layer())
        .route("/", post(create));

    Router::new().nest("/users", router)
}

#[derive(Debug, Clone, Deserialize, Validate)]
struct CreateInData {
    #[validate(length(min = 3, max = 30, message = "username must be between 3 and 30 characters"))]
    pub name: String,
    #[validate(email(message = "invalid email address"))]
    pub email: String,
    #[validate(length(min = 6, message = "password must be at least 6 characters"))]
    pub password: String,
}

// 宏的扩展是从外到内的
// handler宏会看到原始定义
// debug_handler宏会看到handler宏生成的代码
// 如果顺序相反，则会遇到错误
// axum中，请求体只能消费一次
// 因此只能有一个消费请求体的提取器
// axum要求消费请求体的提取器是参数列表中的最后一个来强制执行该规则
#[handler]
#[debug_handler]
async fn create(idata: ValidJson<CreateInData>) {
    let CreateInData { name, email, mut password } = idata.0;
    password = util::hash_password(&password)?;
    let active_model = user::ActiveModel {
        id: ActiveValue::NotSet,
        username: ActiveValue::Set(name),
        email: ActiveValue::Set(email),
        password: ActiveValue::Set(password),
        birthday: ActiveValue::Set(None),
        created_at: ActiveValue::NotSet,
        updated_at: ActiveValue::NotSet,
    };
    active_model.insert(&db).await?;
    Ok(ApiResponse::ok("User created successfully".to_owned(), None))
}

#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
struct UpdateInData {
    #[validate(length(min = 3, max = 30, message = "username must be between 3 and 30 characters"))]
    name: Option<String>,
    #[validate(email(message = "invalid email address"))]
    email: Option<String>,
    // 直接使用 NaiveDate 类型，Serde 会自动处理 "YYYY-MM-DD" 格式的字符串
    #[validate(custom(function = "validate_birthday"))]
    birthday: Option<NaiveDate>,
    // #[serde(flatten)]
    #[validate(nested)]
    password: Option<UpdateInPassword>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
struct UpdateInPassword {
    #[validate(length(min = 6, message = "password must be at least 6 characters"))]
    pub current: String,
    #[validate(length(min = 6, message = "password must be at least 6 characters"))]
    pub new: String,
}

#[handler]
#[debug_handler]
async fn update(Path(id): Path<Uuid>, idata: ValidJson<UpdateInData>) {
    let UpdateInData { name, email, birthday, password } = idata.0;
    // 查找现有用户
    let user = user::Entity::find_by_id(id)
        .one(&db)
        .await?
        .ok_or_else(|| ApiError::NotFound)?;
    
    // 检查是否尝试更改已设置的生日
    if birthday.is_some() && user.birthday.is_some() {
        return Err(ApiError::BadRequest("Birthday cannot be changed once set".to_string()));
    }
    
    // 转换为ActiveModel以便更新
    let mut user_model: user::ActiveModel = user.into();
    
    // 处理密码更新逻辑（如果提供了新密码）
    if let Some(UpdateInPassword { current, new }) = password {
        if current == new {
            return Err(ApiError::BadRequest("New password must be different from current password".to_string()));
        }
        // 验证当前密码是否正确
        match util::verify_password(&current, &user_model.password.try_as_ref().unwrap()) {
            Ok(_) => {}, // 密码正确，继续
            Err(_) => return Err(ApiError::BadRequest("Current password is incorrect".to_string())),
        }

        let hashed_password = util::hash_password(&new)?;
        user_model.password = ActiveValue::Set(hashed_password);
    }
    
    // 处理普通信息更新逻辑
    if let Some(name) = name {
        user_model.username = ActiveValue::Set(name);
    }

    if let Some(email) = email {
        user_model.email = ActiveValue::Set(email);
    }
    
    if let Some(birthday_date) = birthday {
        user_model.birthday = ActiveValue::Set(Some(birthday_date));
    }
    
    // 执行更新
    user_model.update(&db).await?;

    Ok(ApiResponse::ok("User updated successfully".to_owned(), None))
}

fn validate_birthday(birthday: &NaiveDate) -> Result<(), ValidationError> {
    // 验证日期合理性（不能是未来日期）
        let today = chrono::Local::now().naive_local().date();
        if birthday > &today {
            return Err(ValidationError::new("birthday cannot be in the future"));
        }
    Ok(())
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct ListQuery {
    keyword: Option<String>,
    #[validate(nested)]
    #[serde(flatten)]
    pagination: PaginationParams,
}

#[handler]
#[debug_handler]
pub async fn list(query: ValidQuery<ListQuery>) -> Page<user::Model> {
    let ListQuery { keyword, pagination } = query.0;
    let paginator = User::find().apply_if(keyword.as_ref(), |query, keyword| {
        query.filter(
            Condition::any()
                .add(user::Column::Username.contains(keyword))
                .add(user::Column::Email.contains(keyword))
        )
    })
    .order_by_desc(user::Column::CreatedAt)
    .paginate(&db, pagination.size);
    let total = paginator.num_items().await?;
    let items = paginator.fetch_page(pagination.page - 1).await?;
    
    Ok(ApiResponse::ok("User list fetched successfully".to_owned(), Some(Page::from_params(&pagination, total, items))))
}
