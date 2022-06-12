use axum::{
    async_trait,
    extract::{FromRequest, RequestParts},
    headers::{authorization::Bearer, Authorization},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json, TypedHeader,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use std::str::from_utf8;
use uuid::Uuid;

static SECRET: Lazy<String> = Lazy::new(|| {
    let secret = std::env::var("SECRET").expect("SECRET env var not found");
    secret
});

static REFRESH_SECRET: Lazy<String> = Lazy::new(|| {
    let secret = std::env::var("REFRESH_SECRET").expect("REFRESH_SECRET env var not found");
    secret
});

#[derive(Deserialize, Serialize, Clone)]
pub struct Claims {
    sub: Uuid,
    exp: u64,
}

impl Claims {
    pub fn new(user_id: Uuid, exp: u64) -> Self {
        Self { sub: user_id, exp }
    }
    pub fn get_sub(&self) -> Uuid {
        self.sub
    }
}

#[async_trait]
impl<B> FromRequest<B> for Claims
where
    B: Send,
{
    type Rejection = AuthError;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        // Extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request(req)
                .await
                .map_err(|_| AuthError::MissingCredentials)?;

        // Validate and decode the jwt
        let token_data = decode::<Claims>(
            bearer.token(),
            &DecodingKey::from_secret(&SECRET.as_ref()),
            &Validation::default(),
        )
        .map_err(|_| AuthError::InvalidToken)?;

        Ok(token_data.claims)
    }
}

pub struct RefreshClaims {
    pub claims: Claims,
    pub password: String,
}

#[async_trait]
impl<B> FromRequest<B> for RefreshClaims
where
    B: Send,
{
    type Rejection = AuthError;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        // Extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request(req)
                .await
                .map_err(|_| AuthError::MissingCredentials)?;

        // Get the jwt payload and decode it
        let base64_payload = bearer
            .token()
            .split(".")
            .nth(1)
            .ok_or(AuthError::InvalidToken)?;
        let decoded = &base64::decode(base64_payload).map_err(|_| AuthError::InvalidToken)?[..];
        let payload = from_utf8(decoded).map_err(|_| AuthError::InvalidToken)?;
        let claims: Claims = serde_json::from_str(payload).map_err(|_| AuthError::InvalidToken)?;

        // Get the user password from the database
        let Extension(pool) = Extension::<PgPool>::from_request(req)
            .await
            .expect("`DatabaseConnection` extension is missing");
        let user = sqlx::query!(r#"SELECT password FROM "user" WHERE id = $1"#, claims.sub)
            .fetch_one(&pool)
            .await
            .map_err(|_| AuthError::InvalidToken)?;

        // Validate and decode the jwt
        let token_data = decode::<Claims>(
            bearer.token(),
            &DecodingKey::from_secret((REFRESH_SECRET.clone() + user.password.as_str()).as_ref()),
            &Validation::default(),
        )
        .map_err(|_| AuthError::InvalidToken)?;

        Ok(RefreshClaims {
            claims: token_data.claims,
            password: user.password,
        })
    }
}

pub enum AuthError {
    MissingCredentials,
    InvalidToken,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::MissingCredentials => (StatusCode::BAD_REQUEST, "Missing credentials"),
            AuthError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token"),
        };
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}

#[derive(Serialize)]
pub struct Tokens {
    pub token: String,
    pub refresh_token: String,
}
