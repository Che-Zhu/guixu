use axum::{routing::get, Router};

use crate::health::respond_to_health_check::respond_to_health_check;

pub fn build_app() -> Router {
    Router::new().route("/healthz", get(respond_to_health_check))
}
