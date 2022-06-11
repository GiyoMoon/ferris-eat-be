use crate::api;
use axum::{
    routing::{get, patch, post, put},
    Extension, Router,
};
use sqlx::PgPool;

pub fn routes(pool: PgPool) -> Router {
    let users_api = Router::new()
        .route("/register", post(api::users::register))
        .route("/refresh", patch(api::users::refresh))
        .route("/login", post(api::users::login))
        .route("/me", get(api::users::me))
        .route("/update", put(api::users::update))
        .route("/change_password", put(api::users::change_password))
        .layer(Extension(pool));

    Router::new().nest("/api/users", users_api)
}
