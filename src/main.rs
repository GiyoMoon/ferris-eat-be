mod api;
mod app;
mod routes;
mod structs;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    app::init().await;
}
