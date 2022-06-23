use crate::routes::routes;
use axum::Server;
use sqlx::PgPool;
use std::{env, net::SocketAddr};

pub async fn init() {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL env var not found");
    // make sure all environment variables exist at startup
    env::var("SECRET").expect("SECRET env var not found");
    env::var("REFRESH_SECRET").expect("REFRESH_SECRET env var not found");

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Database connection failed");

    let bind_address: SocketAddr = env::var("BIND_ADDRESS")
        .unwrap_or_else(|_| "0.0.0.0:8080".to_string())
        .parse()
        .expect("BIND_ADDRESS is invalid");

    println!("Server started on {}", bind_address);

    Server::bind(&bind_address)
        .serve(routes(pool).into_make_service())
        .await
        .unwrap();
}
