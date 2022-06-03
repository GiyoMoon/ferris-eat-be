use std::env;

use migration::{Migrator, MigratorTrait};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL env var not found");

    let connection = sea_orm::Database::connect(&database_url).await.expect("Database connection failed");
    Migrator::up(&connection, None).await.unwrap();
}
