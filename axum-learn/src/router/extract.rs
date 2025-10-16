use axum::extract::Request;
use axum::extract::{FromRequest, FromRequestParts};
use axum::http::request::Parts;
use axum_valid::HasValidate;
use validator::Validate;

use crate::error::ApiError;

// 桥接axum::extract::Query的from_request
// 通过from将QueryRejection转换为ApiError
// 调用ApiError的IntoResponse实现
#[derive(FromRequestParts)]
#[from_request(via(axum::extract::Query), rejection(ApiError))]
pub struct Query<T>(pub T);

#[derive(FromRequestParts)]
#[from_request(via(axum::extract::Path), rejection(ApiError))]
pub struct Path<T>(pub T);

#[derive(FromRequest)]
#[from_request(via(axum::extract::Json), rejection(ApiError))]
pub struct Json<T>(pub T);

// ValidRejection内包装的错误是内部抽取器的错误
// 对于自定义的抽取器，也就是ApiError
// 需要实现VaildRejection<ApiError>到ApiError的转换
#[derive(FromRequest, FromRequestParts)]
#[from_request(via(axum_valid::Valid), rejection(ApiError))]
pub struct Valid<T>(pub T);

// 将需求委派给Valid<Query<T>>
pub struct ValidQuery<T>(pub T);

pub struct ValidPath<T>(pub T);

pub struct ValidJson<T>(pub T);

macro_rules! impl_has_validate {
    ($($ty:ident),+) => {
        $(
            impl<T> HasValidate for $ty<T>
            where
                T: Validate,
            {
                type Validate = T;

                fn get_validate(&self) -> &Self::Validate {
                    &self.0
                }
            }
        )+
    };
}

macro_rules! impl_from_request_parts {
    ($name: ident, $wrapper: ident) => {
        impl<S, T> FromRequestParts<S> for $name<T>
        where
            S: Send + Sync,
            // Valid<Extractor>的FromRequestParts<S>实现中要求
            // Extractor实现FromRequestParts<S>和HasValidate
            // 并要求Extractor的Validate关联类型实现Validate
            // 这样写太啰嗦了，直接要求Valid<Extractor>实现了FromRequestParts<S>
            // 则Extractor必然满足了前面的要求
            Valid<$wrapper<T>>: FromRequestParts<S, Rejection = ApiError>,
        {
            type Rejection = ApiError;

            async fn from_request_parts(
                parts: &mut Parts,
                state: &S,
            ) -> Result<Self, Self::Rejection> {
                Ok($name(Valid::from_request_parts(parts, state).await?.0.0))
            }
        }
    };
}

macro_rules! impl_from_request {
    ($name: ident, $wrapper: ident) => {
        impl<S, T> FromRequest<S> for $name<T>
        where
            S: Send + Sync,
            Valid<$wrapper<T>>: FromRequest<S, Rejection = ApiError>,
        {
            type Rejection = ApiError;

            async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
                Ok($name(Valid::from_request(request, state).await?.0.0))
            }
        }
    };
}

impl_from_request_parts!(ValidQuery, Query);
impl_from_request_parts!(ValidPath, Path);
impl_from_request!(ValidJson, Json);
impl_has_validate!(Query, Path, Json);