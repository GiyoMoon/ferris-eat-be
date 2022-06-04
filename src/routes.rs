use axum::{routing::post, Extension, Router};
use sea_orm::DatabaseConnection;

use crate::api;

pub fn routes(connection: DatabaseConnection) -> Router {
    let users_api = Router::new()
        .route("/register", post(api::users::register))
        .layer(Extension(connection));

    Router::new().nest("/api/users", users_api)
}
