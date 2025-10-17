use std::pin::Pin;
use std::sync::LazyLock;
use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, Response, header};
use jsonwebtoken::errors::Result as JwtResult;
use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode, get_current_timestamp,
};
use sea_orm::prelude::Uuid;
use serde::{Deserialize, Serialize};
use tower_http::auth::{AsyncAuthorizeRequest, AsyncRequireAuthorizationLayer};

use crate::error::ApiError;

static JWT: LazyLock<Jwt> = LazyLock::new(|| Jwt::new());

#[derive(Debug, Clone)]
pub struct Principal {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    jti: String,
    sub: String,
    aud: String,
    iss: String,
    iat: u64,
    exp: u64,
}

pub struct Jwt {
    encoding_secret: EncodingKey,
    decoding_secret: DecodingKey,
    header: Header,
    validation: Validation,
    expiration: Duration,
    audience: String,
    issuer: String,
}

impl Jwt {
    pub fn new() -> Self {
        let config = crate::config::get().jwt();
        let encoding_secret = EncodingKey::from_secret(config.secret().as_bytes());
        let decoding_secret = DecodingKey::from_secret(config.secret().as_bytes());
        let header = Header::new(Algorithm::HS256);
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_audience(&[config.audience()]);
        validation.set_issuer(&[config.issuer()]);
        validation.set_required_spec_claims(&["jti", "sub", "iss", "aud", "iat", "exp"]);
        Self {
            encoding_secret,
            decoding_secret,
            header,
            validation,
            expiration: config.expiration(),
            audience: config.audience().to_string(),
            issuer: config.issuer().to_string(),
        }
    }

    pub fn encode(&self, principal: Principal) -> JwtResult<String> {
        let current_timestamp = get_current_timestamp();
        let claims = Claims {
            jti: xid::new().to_string(),
            sub: format!("{}:{}", principal.id, principal.name),
            aud: self.audience.clone(),
            iss: self.issuer.clone(),
            iat: current_timestamp,
            exp: current_timestamp.saturating_add(self.expiration.as_secs()),
        };
        encode(&self.header, &claims, &self.encoding_secret)
    }

    pub fn decode(&self, token: &str) -> JwtResult<Principal> {
        let claims: Claims = decode(token, &self.decoding_secret, &self.validation)?.claims;
        let mut parts = claims.sub.splitn(2, ':');
        let principal = Principal {
            id: parts.next().unwrap().parse().map_err(|_| {
                jsonwebtoken::errors::Error::from(jsonwebtoken::errors::ErrorKind::InvalidToken)
            })?,
            name: parts.next().unwrap().to_string(),
        };
        Ok(principal)
    }
}

pub fn get_jwt() -> &'static Jwt {
    &JWT
}

#[derive(Clone)]
pub struct JwtAuth {
    jwt: &'static Jwt,
}

impl JwtAuth {
    pub fn new(jwt: &'static Jwt) -> Self {
        Self { jwt }
    }
}

impl AsyncAuthorizeRequest<Body> for JwtAuth {
    type RequestBody = Body;
    type ResponseBody = Body;
    type Future = Pin<
        Box<
            dyn Future<Output = Result<Request<Self::RequestBody>, Response<Self::ResponseBody>>>
                + Send,
        >,
    >;

    fn authorize(&mut self, mut req: Request<Self::RequestBody>) -> Self::Future {
        let jwt = self.jwt;
        Box::pin(async move {
            tracing::info!("authorizing request: {}", req.uri().path());
            let token = req
                .headers()
                .get(header::AUTHORIZATION)
                .map(|value| -> Result<_, ApiError> {
                    let token = value
                        .to_str()
                        .map_err(|_| {
                            ApiError::Unauthorized(format!("Invalid authorization header"))
                        })?
                        .strip_prefix("Bearer ")
                        .ok_or_else(|| {
                            ApiError::Unauthorized(
                                "Authorization header must be Bearer <token>".to_string(),
                            )
                        })?;
                    Ok(token)
                })
                .transpose()?
                .ok_or_else(|| {
                    ApiError::Unauthorized("Authorization header is required".to_string())
                })?;

            let principal = jwt.decode(token).map_err(|err| ApiError::from(err))?;
            req.extensions_mut().insert(principal);
            Ok(req)
        })
    }
}

pub fn get_auth_layer() -> AsyncRequireAuthorizationLayer<JwtAuth> {
    AsyncRequireAuthorizationLayer::new(JwtAuth::new(get_jwt()))
}
