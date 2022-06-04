mod app;
mod routes;
mod api;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    app::init().await;
}
