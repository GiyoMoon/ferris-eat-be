use super::auth::{RefreshClaims, Tokens};
use super::service::get_tokens;
use axum::{extract, http::StatusCode, Extension, Json};
use entity::structs::user::UserLogin;
use entity::{entities::user, structs::user::Password};
use sea_orm::{
    prelude::Uuid, ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
};
use validator::Validate;

#[axum_macros::debug_handler]
pub async fn register(
    extract::Json(payload): extract::Json<user::Model>,
    Extension(ref connection): Extension<DatabaseConnection>,
) -> Result<(StatusCode, Json<Tokens>), (StatusCode, String)> {
    payload
        .validate()
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let existing_user = user::Entity::find()
        .filter(
            user::Column::Username
                .eq(payload.username.to_lowercase())
                .or(user::Column::Email.eq(payload.email.to_lowercase())),
        )
        .one(connection)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed creating user".to_string(),
            )
        })?;

    match existing_user {
        None => (),
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
    }

    let user_id: Uuid = Uuid::new_v4();

    let hashed_password = Password::from_plain(payload.password).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error while hashing the password".to_string(),
        )
    })?;

    let insert_result = user::ActiveModel {
        id: Set(user_id),
        username: Set(payload.username.to_lowercase()),
        alias: Set(payload.alias),
        email: Set(payload.email.to_lowercase()),
        password: Set(hashed_password.get().to_string()),
        ..Default::default()
    }
    .insert(connection)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed creating user".to_string(),
        )
    })?;

    let tokens = get_tokens(insert_result.id, insert_result.password).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed creating user".to_string(),
        )
    })?;

    Ok((StatusCode::CREATED, Json(tokens)))
}

#[axum_macros::debug_handler]
pub async fn refresh(
    refresh_claims: RefreshClaims,
) -> Result<(StatusCode, Json<Tokens>), (StatusCode, String)> {
    let tokens =
        get_tokens(refresh_claims.claims.get_sub(), refresh_claims.password).map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed refreshing tokens".to_string(),
            )
        })?;

    Ok((StatusCode::OK, Json(tokens)))
}

#[axum_macros::debug_handler]
pub async fn login(
    extract::Json(payload): extract::Json<UserLogin>,
    Extension(ref connection): Extension<DatabaseConnection>,
) -> Result<(StatusCode, Json<Tokens>), (StatusCode, String)> {
    let user = user::Entity::find()
        .filter(user::Column::Username.eq(payload.username))
        .one(connection)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed logging in".to_string(),
            )
        })?
        .ok_or((
            StatusCode::UNAUTHORIZED,
            "Invalid username or password".to_string(),
        ))?;

    let valid_password = Password::from_hash(user.password.clone())
        .verify(payload.password)
        .map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                "Invalid username or password".to_string(),
            )
        })?;

    if !valid_password {
        return Err((
            StatusCode::UNAUTHORIZED,
            "Invalid username or password".to_string(),
        ));
    }

    let tokens = get_tokens(user.id, user.password).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed refreshing tokens".to_string(),
        )
    })?;

    Ok((StatusCode::OK, Json(tokens)))
}
