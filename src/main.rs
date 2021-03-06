mod api;
mod app;
mod routes;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    app::init().await;
}
