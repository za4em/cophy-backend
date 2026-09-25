use std::error::Error;

use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
    dotenvy::dotenv().ok();

    let config = cophy_backend::Config::from_env()?;
    let bind_address = config.server.bind_address.clone();
    let state = cophy_backend::AppState::connect(config).await?;
    let listener = tokio::net::TcpListener::bind(bind_address).await?;
    tracing::info!("listening on {}", listener.local_addr()?);
    axum::serve(listener, cophy_backend::router(state)).await?;
    Ok(())
}
