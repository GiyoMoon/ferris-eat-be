use axum::{Extension, http::StatusCode, Json, extract};
use entity::entities::user;
use sea_orm::{DatabaseConnection, ActiveModelTrait, prelude::Uuid, Set, EntityTrait};

#[axum_macros::debug_handler]
pub async fn register(
  extract::Json(payload): extract::Json<user::Model>,
  Extension(ref connection): Extension<DatabaseConnection>,
) -> Result<Json<user::Model>, StatusCode> {
  let user_id: Uuid = Uuid::new_v4();

  user::ActiveModel {
    id: Set(user_id),
    username: Set(payload.username),
  }.insert(connection).await
  .expect("Failt creating new user");

  let new_user = user::Entity::find_by_id(user_id).one(connection).await.expect("Error while getting created user");

  match new_user {
    Some(user) => Ok(Json(user)),
    None => Err(StatusCode::INTERNAL_SERVER_ERROR)
  }
}
