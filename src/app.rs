use axum::Server;

use std::{env, net::SocketAddr};

use migration::{Migrator, MigratorTrait};

use crate::routes::routes;

#[derive(Clone)]
pub struct EnvVars {
    pub secret: String,
    pub refresh_secret: String,
}

pub async fn init() {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL env var not found");
    let secret = env::var("SECRET").expect("SECRET env var not found");
    let refresh_secret = env::var("REFRESH_SECRET").expect("REFRESH_SECRET env var not found");

    let connection = sea_orm::Database::connect(&database_url)
        .await
        .expect("Database connection failed");
    Migrator::up(&connection, None).await.unwrap();

    let bind_address: SocketAddr = env::var("BIND_ADDRESS")
        .expect("BIND_ADDRESS env var not foundt")
        .parse()
        .expect("BIND_ADDRESS is invalid");

    let env_vars = EnvVars {
        secret,
        refresh_secret,
    };

    println!("Server started on {}", bind_address);

    Server::bind(&bind_address)
        .serve(routes(connection, env_vars).into_make_service())
        .await
        .unwrap();
}
