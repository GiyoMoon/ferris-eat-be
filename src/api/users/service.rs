use crate::api::auth::{Claims, Tokens};
use axum::http::StatusCode;
use bcrypt::BcryptResult;
use jsonwebtoken::{encode, EncodingKey, Header};
use once_cell::sync::Lazy;
use sqlx::PgPool;
use std::time::{SystemTime, UNIX_EPOCH};
use time::PrimitiveDateTime;
use uuid::Uuid;

static SECRET: Lazy<String> =
    Lazy::new(|| std::env::var("SECRET").expect("SECRET env var not found"));

static REFRESH_SECRET: Lazy<String> =
    Lazy::new(|| std::env::var("REFRESH_SECRET").expect("REFRESH_SECRET env var not found"));

pub struct UserModel {
    pub id: Uuid,
    pub username: String,
    pub alias: String,
    pub email: String,
    pub password: String,
    pub created_at: PrimitiveDateTime,
    pub updated_at: PrimitiveDateTime,
}

pub struct Password(String);

impl Password {
    pub fn from_plain(clear_text_password: String) -> Result<Password, (StatusCode, String)> {
        Ok(Password(bcrypt::hash(clear_text_password, 10).map_err(
            |_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed hashing the password".to_string(),
                )
            },
        )?))
    }

    pub fn from_hash(hash: String) -> Password {
        Password(hash)
    }

    pub fn get(&self) -> &str {
        &self.0
    }

    pub fn verify(&self, clear_text_password: String) -> BcryptResult<bool> {
        bcrypt::verify(clear_text_password, &self.0)
    }
}

pub fn get_tokens(uuid: Uuid, password: String) -> Result<Tokens, (StatusCode, String)> {
    let token = encode(
        &Header::default(),
        &Claims::new(
            uuid,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|_| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed creating tokens".to_string(),
                    )
                })?
                .as_secs()
                + 5 * 60,
        ),
        &EncodingKey::from_secret(SECRET.as_ref()),
    )
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed creating tokens".to_string(),
        )
    })?;

    let refresh_token = encode(
        &Header::default(),
        &Claims::new(
            uuid,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|_| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed creating tokens".to_string(),
                    )
                })?
                .as_secs()
                + 7 * 24 * 60 * 60,
        ),
        &EncodingKey::from_secret((REFRESH_SECRET.clone() + password.as_str()).as_ref()),
    )
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed creating tokens".to_string(),
        )
    })?;

    Ok(Tokens {
        token,
        refresh_token,
    })
}

pub async fn get_user_by_uuid(
    uuid: Uuid,
    pool: &PgPool,
) -> Result<UserModel, (StatusCode, String)> {
    sqlx::query_as!(UserModel, r#"SELECT * FROM "user" WHERE id = $1"#, uuid)
        .fetch_one(pool)
        .await
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Failed getting user".to_string()))
}
