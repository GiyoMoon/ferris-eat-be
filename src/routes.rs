use axum::{routing::post, Extension, Router};
use sea_orm::DatabaseConnection;

use crate::api;

pub fn routes(connection: DatabaseConnection) -> Router {
    let users_api = Router::new()
        .route("/register", post(api::users::register))
        .route("/refresh", post(api::users::refresh))
        .route("/login", post(api::users::login))
        .layer(Extension(connection));

    Router::new().nest("/api/users", users_api)
}
