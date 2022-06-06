use axum::{routing::post, Extension, Router};
use sea_orm::DatabaseConnection;

use crate::{api, app::EnvVars};

pub fn routes(connection: DatabaseConnection, env_vars: EnvVars) -> Router {
    let users_api = Router::new()
        .route("/register", post(api::users::register))
        .layer(Extension(connection))
        .layer(Extension(env_vars));

    Router::new().nest("/api/users", users_api)
}
