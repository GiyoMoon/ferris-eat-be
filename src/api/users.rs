use axum::{extract, http::StatusCode, response::IntoResponse, Extension, Json};
use chrono::Utc;
use entity::{
    entities::user,
    structs::user::{LoginUser, Password},
};
use sea_orm::{
    prelude::Uuid, ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
};
use validator::Validate;

#[axum_macros::debug_handler]
pub async fn register(
    extract::Json(payload): extract::Json<user::Model>,
    Extension(ref connection): Extension<DatabaseConnection>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    match payload.validate() {
        Ok(_) => (),
        Err(e) => return Err((StatusCode::BAD_REQUEST, e.to_string())),
    };

    let existing_user = user::Entity::find()
        .filter(user::Column::Username.eq(payload.username.to_lowercase()))
        .one(connection)
        .await;
    let existing_user = match existing_user {
        Ok(result) => result,
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed creating user".to_string(),
            ));
        }
    };
    match existing_user {
        None => (),
        Some(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                "Username already exists".to_string(),
            ))
        }
    }
    let existing_user = user::Entity::find()
        .filter(user::Column::Email.eq(payload.email.to_lowercase()))
        .one(connection)
        .await;
    let existing_user = match existing_user {
        Ok(result) => result,
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed creating user".to_string(),
            ));
        }
    };
    match existing_user {
        None => (),
        Some(_) => return Err((StatusCode::BAD_REQUEST, "Email already in use".to_string())),
    }

    let user_id: Uuid = Uuid::new_v4();

    let hashed_password = Password::from_plain(payload.password);

    let hashed_password = match hashed_password {
        Ok(pw) => pw,
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error while hashing the password".to_string(),
            ))
        }
    };

    let now = Utc::now().naive_utc();

    let insert_result = user::ActiveModel {
        id: Set(user_id),
        username: Set(payload.username.to_lowercase()),
        alias: Set(payload.alias),
        email: Set(payload.email.to_lowercase()),
        password: Set(hashed_password.get().to_string()),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(connection)
    .await;

    match insert_result {
        Ok(result) => Ok((StatusCode::CREATED, Json(LoginUser::from(result)))),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed creating user".to_string(),
        )),
    }
}
