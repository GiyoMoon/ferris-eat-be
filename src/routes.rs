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
        .layer(Extension(pool.clone()));

    let recipes_api = Router::new()
        .route("/", get(api::recipes::get_all).post(api::recipes::create))
        .route(
            "/:id",
            get(api::recipes::get)
                .put(api::recipes::update)
                .delete(api::recipes::delete),
        )
        .layer(Extension(pool.clone()));

    let ingredients_api = Router::new()
        .route(
            "/",
            get(api::ingredients::get_all)
                .post(api::ingredients::create)
                .patch(api::ingredients::sort),
        )
        .route(
            "/:id",
            put(api::ingredients::update).delete(api::ingredients::delete),
        )
        .layer(Extension(pool.clone()));

    let units_api = Router::new()
        .route("/", get(api::units::get_all))
        .layer(Extension(pool.clone()));

    Router::new()
        .nest("/api/users", users_api)
        .nest("/api/recipes", recipes_api)
        .nest("/api/ingredients", ingredients_api)
        .nest("/api/units", units_api)
}
