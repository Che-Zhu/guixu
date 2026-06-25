use axum::{routing::get, Router};

use crate::{
    health::respond_to_health_check::respond_to_health_check,
    youzhiyouxing::{
        fetch_youzhiyouxing_pages::YouzhiyouxingPageSource,
        respond_with_youzhiyouxing::respond_with_youzhiyouxing,
    },
};

#[derive(Clone)]
pub struct AppState {
    pub youzhiyouxing_source: YouzhiyouxingPageSource,
}

pub fn build_app(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(respond_to_health_check))
        .route("/youzhiyouxing", get(respond_with_youzhiyouxing))
        .with_state(state)
}
