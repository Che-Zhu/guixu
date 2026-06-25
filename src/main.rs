use guixu::{app::build_app, config::load_config_from_env};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "guixu=info,tower_http=info".to_string()),
        )
        .init();

    let config = load_config_from_env().expect("invalid guixu configuration");
    let listener = tokio::net::TcpListener::bind(&config.bind_addr)
        .await
        .expect("failed to bind GUIXU_BIND_ADDR");

    tracing::info!(bind_addr = %config.bind_addr, "starting guixu");
    axum::serve(listener, build_app())
        .await
        .expect("server failed");
}
