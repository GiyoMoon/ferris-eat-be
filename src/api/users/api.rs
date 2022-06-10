use super::auth::{Claims, RefreshClaims, Tokens};
use super::service::get_tokens;
use axum::{extract, http::StatusCode, Extension, Json};
use entity::structs::user::{UserChangePassword, UserInfo, UserLogin, UserUpdate};
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

#[axum_macros::debug_handler]
pub async fn me(
    claims: Claims,
    Extension(ref connection): Extension<DatabaseConnection>,
) -> Result<(StatusCode, Json<UserInfo>), (StatusCode, String)> {
    let user = user::Entity::find_by_id(claims.get_sub())
        .one(connection)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed getting user".to_string(),
            )
        })?
        .ok_or((StatusCode::UNAUTHORIZED, "Failed getting user".to_string()))?;

    Ok((StatusCode::OK, Json(UserInfo::from(user))))
}

#[axum_macros::debug_handler]
pub async fn update(
    claims: Claims,
    extract::Json(payload): extract::Json<UserUpdate>,
    Extension(ref connection): Extension<DatabaseConnection>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut user: user::ActiveModel = user::Entity::find_by_id(claims.get_sub())
        .one(connection)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed updating user".to_string(),
            )
        })?
        .ok_or((StatusCode::UNAUTHORIZED, "Failed updating user".to_string()))?
        .into();

    user.alias = Set(payload.alias);
    user.email = Set(payload.email);

    user.update(connection).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed updating user".to_string(),
        )
    })?;

    Ok(StatusCode::OK)
}

#[axum_macros::debug_handler]
pub async fn change_password(
    claims: Claims,
    extract::Json(payload): extract::Json<UserChangePassword>,
    Extension(ref connection): Extension<DatabaseConnection>,
) -> Result<(StatusCode, Json<Tokens>), (StatusCode, String)> {
    let user = user::Entity::find_by_id(claims.get_sub())
        .one(connection)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed changing password".to_string(),
            )
        })?
        .ok_or((
            StatusCode::UNAUTHORIZED,
            "Failed changing password".to_string(),
        ))?;

    let valid_password = Password::from_hash(user.password.clone())
        .verify(payload.old_password)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid old password".to_string()))?;

    if !valid_password {
        return Err((StatusCode::UNAUTHORIZED, "Invalid old password".to_string()));
    }

    let mut user: user::ActiveModel = user.into();

    let hashed_password = Password::from_plain(payload.new_password).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error while hashing the password".to_string(),
        )
    })?;

    user.password = Set(hashed_password.get().to_string());

    user.update(connection).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed changing password".to_string(),
        )
    })?;

    let tokens = get_tokens(claims.get_sub(), hashed_password.get().to_string()).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed refreshing tokens".to_string(),
        )
    })?;

    Ok((StatusCode::OK, Json(tokens)))
}
