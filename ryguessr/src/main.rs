use axum::Router;
use log::info;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let app = Router::new().fallback_service(ServeDir::new("../web/out"));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    info!("Listening on: {}", listener.local_addr()?);

    axum::serve(listener, app).await?;

    Ok(())
}
