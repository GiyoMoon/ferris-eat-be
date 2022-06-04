use axum::{extract, http::StatusCode, response::IntoResponse, Extension, Json};
use entity::{entities::user, structs::user::LoginUser};
use sea_orm::{
    prelude::Uuid, ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
};

#[axum_macros::debug_handler]
pub async fn register(
    extract::Json(payload): extract::Json<user::Model>,
    Extension(ref connection): Extension<DatabaseConnection>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    if payload.username.len() < 4 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Username has to be 4 or more chars long",
        ));
    }

    let existing_user = user::Entity::find()
        .filter(user::Column::Username.eq(payload.username.to_lowercase()))
        .one(connection)
        .await;
    let existing_user = match existing_user {
        Ok(result) => result,
        Err(_) => return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed creating user")),
    };
    match existing_user {
        None => {}
        Some(_) => return Err((StatusCode::BAD_REQUEST, "Username already exists")),
    }

    let user_id: Uuid = Uuid::new_v4();

    let insert_result = user::ActiveModel {
        id: Set(user_id),
        username: Set(payload.username),
    }
    .insert(connection)
    .await;
    match insert_result {
        Ok(result) => Ok((StatusCode::CREATED, Json(LoginUser::from(result)))),
        Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed creating user")),
    }
}
