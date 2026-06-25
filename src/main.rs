use guixu::{
    app::{build_app, AppState},
    config::load_config_from_env,
    youzhiyouxing::fetch_youzhiyouxing_pages::{YouzhiyouxingClient, YouzhiyouxingPageSource},
};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "guixu=info,tower_http=info".to_string()),
        )
        .init();

    let config = load_config_from_env().expect("invalid guixu configuration");
    let youzhiyouxing_client = YouzhiyouxingClient::new(config.youzhiyouxing_cookie.clone())
        .expect("failed to build youzhiyouxing client");
    let app_state = AppState {
        youzhiyouxing_source: YouzhiyouxingPageSource::Live(youzhiyouxing_client),
    };
    let listener = tokio::net::TcpListener::bind(&config.bind_addr)
        .await
        .expect("failed to bind GUIXU_BIND_ADDR");

    tracing::info!(bind_addr = %config.bind_addr, "starting guixu");
    axum::serve(listener, build_app(app_state))
        .await
        .expect("server failed");
}
