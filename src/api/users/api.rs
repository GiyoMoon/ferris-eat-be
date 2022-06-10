use super::auth::RefreshClaims;
use super::auth::Tokens;
use axum::{extract, http::StatusCode, Extension, Json};
use entity::{entities::user, structs::user::Password};
use sea_orm::{
    prelude::Uuid, ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
};
use validator::Validate;

use crate::{
    api::users::service::{login, refresh_token},
    app::EnvVars,
};

#[axum_macros::debug_handler]
pub async fn register(
    extract::Json(payload): extract::Json<user::Model>,
    Extension(ref connection): Extension<DatabaseConnection>,
    Extension(ref env_vars): Extension<EnvVars>,
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

    let tokens = login(&insert_result, env_vars).map_err(|_| {
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
    Extension(ref env_vars): Extension<EnvVars>,
) -> Result<(StatusCode, Json<Tokens>), (StatusCode, String)> {
    let refresh_result = refresh_token(refresh_claims.claims, refresh_claims.password, env_vars)
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed refreshing tokens".to_string(),
            )
        })?;

    Ok((StatusCode::OK, Json(refresh_result)))
}
