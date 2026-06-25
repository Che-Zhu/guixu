use guixu::app::build_app;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "guixu=info,tower_http=info".to_string()),
        )
        .init();

    let bind_addr =
        std::env::var("GUIXU_BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".to_string());
    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .expect("failed to bind GUIXU_BIND_ADDR");

    tracing::info!(%bind_addr, "starting guixu");
    axum::serve(listener, build_app())
        .await
        .expect("server failed");
}
