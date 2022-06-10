use axum::{
    routing::{get, post},
    Extension, Router,
};
use sea_orm::DatabaseConnection;

use crate::api;

pub fn routes(connection: DatabaseConnection) -> Router {
    let users_api = Router::new()
        .route("/register", post(api::users::register))
        .route("/refresh", post(api::users::refresh))
        .route("/login", post(api::users::login))
        .route("/me", get(api::users::me))
        .route("/update", post(api::users::update))
        .route("/change_password", post(api::users::change_password))
        .layer(Extension(connection));

    Router::new().nest("/api/users", users_api)
}
