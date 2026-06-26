use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct KimiUsageResponse {
    pub quotas: KimiQuotas,
    pub meta: KimiMeta,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct KimiQuotas {
    pub five_hour: QuotaBucket,
    pub weekly: QuotaBucket,
    pub purchased: PurchasedQuota,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct QuotaBucket {
    pub limit: u64,
    pub used: u64,
    pub remaining: u64,
    pub resets_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PurchasedQuota {
    pub limit: u64,
    pub remaining: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct KimiMeta {
    pub region: String,
    pub fetched_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum KimiParseError {
    #[error("kimi response missing required field: {0}")]
    MissingRequiredField(&'static str),
    #[error("kimi response field has unexpected type: {0}")]
    UnexpectedFieldType(&'static str),
}

#[derive(Debug, thiserror::Error)]
pub enum KimiFetchError {
    #[error("kimi request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("kimi authentication failed")]
    AuthenticationFailed,
    #[error("kimi API returned unexpected status: {status}")]
    UnexpectedStatus { status: reqwest::StatusCode },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn response_serializes_to_snake_case_json() {
        let response = KimiUsageResponse {
            quotas: KimiQuotas {
                five_hour: QuotaBucket {
                    limit: 100,
                    used: 9,
                    remaining: 91,
                    resets_at: "2026-06-26T06:58:12Z".to_string(),
                },
                weekly: QuotaBucket {
                    limit: 100,
                    used: 5,
                    remaining: 95,
                    resets_at: "2026-07-01T17:58:12Z".to_string(),
                },
                purchased: PurchasedQuota {
                    limit: 100,
                    remaining: 99,
                },
            },
            meta: KimiMeta {
                region: "REGION_CN".to_string(),
                fetched_at: "2026-06-26T10:00:00Z".to_string(),
            },
        };

        let json = serde_json::to_value(&response).expect("should serialize");
        assert_eq!(json["quotas"]["five_hour"]["limit"], 100);
        assert_eq!(json["quotas"]["purchased"]["remaining"], 99);
        assert_eq!(json["meta"]["region"], "REGION_CN");
    }
}
