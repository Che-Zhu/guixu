use axum::{extract::State, Json};

use crate::{app::AppState, error::ApiError};

use super::types::KimiUsageResponse;

pub async fn respond_with_kimi_usage(
    State(state): State<AppState>,
) -> Result<Json<KimiUsageResponse>, ApiError> {
    let usage = state.kimi_source.fetch_usage().await?;
    Ok(Json(usage))
}
