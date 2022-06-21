use crate::{
    api::{
        auth::{Claims, RefreshClaims, Tokens},
        global::ValidatedJson,
        users::service::{get_tokens, get_user_by_uuid},
    },
    structs::user::{
        Password, UserChangePasswordReq, UserLoginReq, UserMeRes, UserRegisterReq, UserUpdateReq,
    },
};
use axum::{extract, http::StatusCode, Extension, Json};
use sqlx::PgPool;
use uuid::Uuid;

#[axum_macros::debug_handler]
pub async fn register(
    ValidatedJson(payload): ValidatedJson<UserRegisterReq>,
    Extension(ref pool): Extension<PgPool>,
) -> Result<(StatusCode, Json<Tokens>), (StatusCode, String)> {
    let existing_user = sqlx::query!(
        r#"SELECT * FROM "user" WHERE username = $1 OR email = $2"#,
        payload.username.to_lowercase(),
        payload.email.to_lowercase()
    )
    .fetch_optional(pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed creating user".to_string(),
        )
    })?;

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

    let hashed_password = Password::from_plain(payload.password).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed hashing the password".to_string(),
        )
    })?;

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
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed creating user".to_string(),
        )
    })?;

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

#[axum_macros::debug_handler]
pub async fn login(
    extract::Json(payload): extract::Json<UserLoginReq>,
    Extension(ref pool): Extension<PgPool>,
) -> Result<(StatusCode, Json<Tokens>), (StatusCode, String)> {
    let user = sqlx::query!(
        r#"SELECT * FROM "user" WHERE username = $1"#,
        payload.username.to_lowercase()
    )
    .fetch_one(pool)
    .await
    .map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            "Invalid username or password".to_string(),
        )
    })?;

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

    let tokens = get_tokens(user.id, user.password)?;

    Ok((StatusCode::OK, Json(tokens)))
}

#[axum_macros::debug_handler]
pub async fn me(
    claims: Claims,
    Extension(ref pool): Extension<PgPool>,
) -> Result<(StatusCode, Json<UserMeRes>), (StatusCode, String)> {
    let user = get_user_by_uuid(claims.get_sub(), pool).await?;

    Ok((StatusCode::OK, Json(UserMeRes::from(user))))
}

#[axum_macros::debug_handler]
pub async fn update(
    claims: Claims,
    ValidatedJson(payload): ValidatedJson<UserUpdateReq>,
    Extension(ref pool): Extension<PgPool>,
) -> Result<StatusCode, (StatusCode, String)> {
    let existing_user = sqlx::query!(
        r#"SELECT * FROM "user" WHERE email = $1 AND NOT id = $2"#,
        payload.email.to_lowercase(),
        claims.get_sub(),
    )
    .fetch_optional(pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed updating user".to_string(),
        )
    })?;

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
    .map_err(|_| {
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
    ValidatedJson(payload): ValidatedJson<UserChangePasswordReq>,
    Extension(ref pool): Extension<PgPool>,
) -> Result<(StatusCode, Json<Tokens>), (StatusCode, String)> {
    let user = get_user_by_uuid(claims.get_sub(), pool).await?;

    let valid_password = Password::from_hash(user.password.clone())
        .verify(payload.old_password)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid old password".to_string()))?;

    if !valid_password {
        return Err((StatusCode::UNAUTHORIZED, "Invalid old password".to_string()));
    }

    let hashed_password = Password::from_plain(payload.new_password).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed hashing the password".to_string(),
        )
    })?;

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
