use crate::{
    api::auth::{Claims, Tokens},
    structs::user::UserModel,
};
use axum::http::StatusCode;
use jsonwebtoken::{encode, EncodingKey, Header};
use once_cell::sync::Lazy;
use sqlx::PgPool;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

static SECRET: Lazy<String> = Lazy::new(|| {
    let secret = std::env::var("SECRET").expect("SECRET env var not found");
    secret
});

static REFRESH_SECRET: Lazy<String> = Lazy::new(|| {
    let secret = std::env::var("REFRESH_SECRET").expect("REFRESH_SECRET env var not found");
    secret
});

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
    Ok(
        sqlx::query_as!(UserModel, r#"SELECT * FROM "user" WHERE id = $1"#, uuid)
            .fetch_one(pool)
            .await
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Failed getting user".to_string()))?,
    )
}
