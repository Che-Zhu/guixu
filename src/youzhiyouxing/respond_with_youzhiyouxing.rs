use axum::{extract::State, Json};

use crate::{app::AppState, error::ApiError};

use super::{parse_youzhiyouxing_pages::parse_youzhiyouxing_pages, types::YouzhiyouxingResponse};

pub async fn respond_with_youzhiyouxing(
    State(state): State<AppState>,
) -> Result<Json<YouzhiyouxingResponse>, ApiError> {
    let pages = state.youzhiyouxing_source.fetch_pages().await?;
    let response = parse_youzhiyouxing_pages(&pages)?;

    Ok(Json(response))
}
