use axum::{routing::get, Router};

use crate::{
    ai::kimi::{client::KimiUsageSource, respond_with_kimi_usage::respond_with_kimi_usage},
    health::respond_to_health_check::respond_to_health_check,
    youzhiyouxing::{
        fetch_youzhiyouxing_pages::YouzhiyouxingPageSource,
        respond_with_youzhiyouxing::respond_with_youzhiyouxing,
    },
};

#[derive(Clone)]
pub struct AppState {
    pub youzhiyouxing_source: YouzhiyouxingPageSource,
    pub kimi_source: KimiUsageSource,
}

pub fn build_app(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(respond_to_health_check))
        .route("/youzhiyouxing", get(respond_with_youzhiyouxing))
        .route("/ai/kimi", get(respond_with_kimi_usage))
        .with_state(state)
}