use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

use crate::{
    ai::kimi::types::{KimiFetchError, KimiParseError},
    youzhiyouxing::types::{YouzhiyouxingFetchError, YouzhiyouxingParseError},
};

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub error: &'static str,
    pub message: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error(transparent)]
    KimiFetch(#[from] KimiFetchError),
    #[error(transparent)]
    KimiParse(#[from] KimiParseError),
    #[error(transparent)]
    YouzhiyouxingFetch(#[from] YouzhiyouxingFetchError),
    #[error(transparent)]
    YouzhiyouxingParse(#[from] YouzhiyouxingParseError),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error, message) = match self {
            ApiError::KimiFetch(KimiFetchError::AuthenticationFailed) => (
                StatusCode::BAD_GATEWAY,
                "upstream_authentication_failed",
                "Kimi API key is invalid or expired. Refresh KIMI_CODING_PLAN_TOKEN."
                    .to_string(),
            ),
            ApiError::KimiFetch(KimiFetchError::UnexpectedStatus { status }) => (
                StatusCode::BAD_GATEWAY,
                "upstream_request_failed",
                format!("Kimi API returned unexpected status: {status}"),
            ),
            ApiError::KimiFetch(KimiFetchError::Request(e)) => (
                StatusCode::BAD_GATEWAY,
                "upstream_request_failed",
                format!("Kimi request failed: {e}"),
            ),
            ApiError::KimiParse(e) => (
                StatusCode::BAD_GATEWAY,
                "upstream_parse_failed",
                format!("Failed to parse Kimi response: {e}"),
            ),
            ApiError::YouzhiyouxingFetch(YouzhiyouxingFetchError::SessionExpired)
            | ApiError::YouzhiyouxingParse(YouzhiyouxingParseError::SessionExpired) => (
                StatusCode::BAD_GATEWAY,
                "upstream_session_expired",
                "Youzhiyouxing session is expired or invalid. Refresh YOUZHIYOUXING_COOKIE."
                    .to_string(),
            ),
            ApiError::YouzhiyouxingFetch(error) => (
                StatusCode::BAD_GATEWAY,
                "upstream_request_failed",
                error.to_string(),
            ),
            ApiError::YouzhiyouxingParse(error) => (
                StatusCode::BAD_GATEWAY,
                "upstream_parse_failed",
                error.to_string(),
            ),
        };

        (status, Json(ErrorBody { error, message })).into_response()
    }
}