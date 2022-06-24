use crate::api::{
    auth::{Claims, RefreshClaims, Tokens},
    global::{get_default_err, ValidatedJson},
    users::service::{get_tokens, get_user_by_uuid, Password},
};
use axum::{extract, http::StatusCode, Extension, Json};
use fancy_regex::Regex;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;
use validator::{Validate, ValidationError};

use super::service::UserModel;

static PASSWORD_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?=.*[A-Za-z])(?=.*\d)(?=.*[^\dA-Za-z]).{8,}").unwrap());

fn validate_password(value: &str) -> Result<(), ValidationError> {
    let is_match = PASSWORD_REGEX
        .is_match(value)
        .map_err(|_| ValidationError::new("invalid_password"))?;
    if !is_match {
        return Err(ValidationError::new("invalid_password"));
    }
    Ok(())
}

#[derive(Deserialize, Validate)]
pub struct RegisterReq {
    #[validate(length(min = 4, message = "Username has to be 4 or more characters long"))]
    pub username: String,
    pub alias: String,
    #[validate(email(message = "Not a valid email address"))]
    pub email: String,
    #[validate(custom(
        function = "validate_password",
        message = "Password must at least be 8 chars long, contain at least one letter, one number and one special character"
    ))]
    pub password: String,
}

#[axum_macros::debug_handler]
pub async fn register(
    ValidatedJson(payload): ValidatedJson<RegisterReq>,
    Extension(ref pool): Extension<PgPool>,
) -> Result<(StatusCode, Json<Tokens>), (StatusCode, String)> {
    let default_err = get_default_err("Failed creating user");

    let existing_user = sqlx::query!(
        r#"SELECT * FROM "user" WHERE username = $1 OR email = $2"#,
        payload.username.to_lowercase(),
        payload.email.to_lowercase()
    )
    .fetch_optional(pool)
    .await
    .map_err(|_| default_err.clone())?;

    match existing_user {
        Some(user) => {
            if user.username == payload.username.to_lowercase() {
                return Err((
                    StatusCode::BAD_REQUEST,
                    "Username already exists".to_string(),
                ));
            } else {
                return Err((StatusCode::BAD_REQUEST, "Email already in use".to_string()));
            }
        }
        None => (),
    }

    let user_id: Uuid = Uuid::new_v4();

    let hashed_password = Password::from_plain(payload.password)?;

    let insert_result = sqlx::query!(
        r#"
            INSERT INTO "user" ( id, username, alias, email, password )
            VALUES ( $1, $2, $3, $4, $5 )
            RETURNING id, password
        "#,
        user_id,
        payload.username.to_lowercase(),
        payload.alias,
        payload.email.to_lowercase(),
        hashed_password.get()
    )
    .fetch_one(pool)
    .await
    .map_err(|_| default_err)?;

    let tokens = get_tokens(insert_result.id, insert_result.password)?;

    Ok((StatusCode::CREATED, Json(tokens)))
}

#[axum_macros::debug_handler]
pub async fn refresh(
    refresh_claims: RefreshClaims,
) -> Result<(StatusCode, Json<Tokens>), (StatusCode, String)> {
    let tokens = get_tokens(refresh_claims.claims.get_sub(), refresh_claims.password)?;

    Ok((StatusCode::OK, Json(tokens)))
}

#[derive(Deserialize)]
pub struct LoginReq {
    pub username: String,
    pub password: String,
}

#[axum_macros::debug_handler]
pub async fn login(
    extract::Json(payload): extract::Json<LoginReq>,
    Extension(ref pool): Extension<PgPool>,
) -> Result<(StatusCode, Json<Tokens>), (StatusCode, String)> {
    let invalid_err = (
        StatusCode::UNAUTHORIZED,
        "Invalid username or password".to_string(),
    );

    let user = sqlx::query!(
        r#"SELECT * FROM "user" WHERE username = $1"#,
        payload.username.to_lowercase()
    )
    .fetch_one(pool)
    .await
    .map_err(|_| invalid_err.clone())?;

    let valid_password = Password::from_hash(user.password.clone())
        .verify(payload.password)
        .map_err(|_| invalid_err.clone())?;

    if !valid_password {
        return Err(invalid_err);
    }

    let tokens = get_tokens(user.id, user.password)?;

    Ok((StatusCode::OK, Json(tokens)))
}

#[derive(Serialize)]
pub struct MeRes {
    pub id: Uuid,
    pub username: String,
    pub alias: String,
    pub email: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

impl From<UserModel> for MeRes {
    fn from(a: UserModel) -> MeRes {
        MeRes {
            id: a.id,
            username: a.username,
            alias: a.alias,
            email: a.email,
            created_at: a.created_at.assume_utc(),
            updated_at: a.updated_at.assume_utc(),
        }
    }
}

#[axum_macros::debug_handler]
pub async fn me(
    claims: Claims,
    Extension(ref pool): Extension<PgPool>,
) -> Result<(StatusCode, Json<MeRes>), (StatusCode, String)> {
    let user = get_user_by_uuid(claims.get_sub(), pool).await?;

    Ok((StatusCode::OK, Json(MeRes::from(user))))
}

#[derive(Deserialize, Validate)]
pub struct UpdateReq {
    pub alias: String,
    #[validate(email(message = "Not a valid email address"))]
    pub email: String,
}

#[axum_macros::debug_handler]
pub async fn update(
    claims: Claims,
    ValidatedJson(payload): ValidatedJson<UpdateReq>,
    Extension(ref pool): Extension<PgPool>,
) -> Result<StatusCode, (StatusCode, String)> {
    let default_err = get_default_err("Failed updating user");

    let existing_user = sqlx::query!(
        r#"SELECT * FROM "user" WHERE email = $1 AND NOT id = $2"#,
        payload.email.to_lowercase(),
        claims.get_sub(),
    )
    .fetch_optional(pool)
    .await
    .map_err(|_| default_err.clone())?;

    match existing_user {
        Some(_) => {
            return Err((StatusCode::BAD_REQUEST, "Email already in use".to_string()));
        }
        None => (),
    }

    sqlx::query!(
        r#"UPDATE "user" SET alias = $1, email = $2 WHERE id = $3"#,
        payload.alias,
        payload.email.to_lowercase(),
        claims.get_sub(),
    )
    .execute(pool)
    .await
    .map_err(|_| default_err)?;

    Ok(StatusCode::OK)
}

#[derive(Deserialize, Validate)]
pub struct ChangePasswordReq {
    pub old_password: String,
    #[validate(custom(
        function = "validate_password",
        message = "Password must at least be 8 chars long, contain at least one letter, one number and one special character"
    ))]
    pub new_password: String,
}

#[axum_macros::debug_handler]
pub async fn change_password(
    claims: Claims,
    ValidatedJson(payload): ValidatedJson<ChangePasswordReq>,
    Extension(ref pool): Extension<PgPool>,
) -> Result<(StatusCode, Json<Tokens>), (StatusCode, String)> {
    let user = get_user_by_uuid(claims.get_sub(), pool).await?;

    let valid_password = Password::from_hash(user.password.clone())
        .verify(payload.old_password)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid old password".to_string()))?;

    if !valid_password {
        return Err((StatusCode::UNAUTHORIZED, "Invalid old password".to_string()));
    }

    let hashed_password = Password::from_plain(payload.new_password)?;

    sqlx::query!(
        r#"UPDATE "user" SET password = $1 WHERE id = $2"#,
        hashed_password.get(),
        claims.get_sub(),
    )
    .execute(pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed changing the password".to_string(),
        )
    })?;

    let tokens = get_tokens(claims.get_sub(), hashed_password.get().to_string())?;

    Ok((StatusCode::OK, Json(tokens)))
}
