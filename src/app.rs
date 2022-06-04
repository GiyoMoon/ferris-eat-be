use axum::Server;

use std::{env, net::SocketAddr};

use migration::{Migrator, MigratorTrait};

use crate::routes::routes;

pub async fn init() {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL env var not found");

    let connection = sea_orm::Database::connect(&database_url).await.expect("Database connection failed");
    Migrator::up(&connection, None).await.unwrap();

    let bind_address: SocketAddr = env::var("BIND_ADDRESS")
        .expect("BIND_ADDRESS env var not foundt")
        .parse()
        .expect("BIND_ADDRESS is invalid");

    println!("Server started on {}", bind_address);

    Server::bind(&bind_address)
        .serve(routes(connection).into_make_service())
        .await
        .unwrap();
}
