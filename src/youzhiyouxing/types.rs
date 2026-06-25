use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct YouzhiyouxingResponse {
    pub dashboard: DashboardSummary,
    pub balance: BalanceSummary,
    pub investment: InvestmentSummary,
    pub cashflow: CashflowSummary,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DashboardSummary {
    pub family_total_assets: f64,
    pub asset_change: Option<f64>,
    pub debt_ratio: f64,
    pub cashflow_configured: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BalanceSummary {
    pub net_assets: f64,
    pub total_assets: f64,
    pub total_liabilities: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct InvestmentSummary {
    pub total_assets: Option<f64>,
    pub accumulated_profit: Option<f64>,
    pub money_weighted_return: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CashflowSummary {
    pub configured: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum YouzhiyouxingParseError {
    #[error("youzhiyouxing session expired")]
    SessionExpired,
    #[error("missing required field: {0}")]
    MissingRequiredField(&'static str),
}

#[derive(Debug, thiserror::Error)]
pub enum YouzhiyouxingFetchError {
    #[error("youzhiyouxing request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("youzhiyouxing session expired")]
    SessionExpired,
    #[error("youzhiyouxing {path} returned unexpected status: {status}")]
    UnexpectedStatus {
        path: String,
        status: reqwest::StatusCode,
    },
}
