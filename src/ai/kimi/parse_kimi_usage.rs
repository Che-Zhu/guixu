use serde_json::Value;

use super::types::{
    KimiMeta, KimiParseError, KimiQuotas, KimiUsageResponse, PurchasedQuota, QuotaBucket,
};

pub fn parse_kimi_usage(
    body: &Value,
    fetched_at: String,
) -> Result<KimiUsageResponse, KimiParseError> {
    let region = body
        .get("user")
        .and_then(|u| u.get("region"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let five_hour = parse_five_hour_bucket(body)?;
    let weekly = parse_weekly_bucket(body)?;
    let purchased = parse_purchased_quota(body)?;

    Ok(KimiUsageResponse {
        quotas: KimiQuotas {
            five_hour,
            weekly,
            purchased,
        },
        meta: KimiMeta { region, fetched_at },
    })
}

fn parse_five_hour_bucket(body: &Value) -> Result<QuotaBucket, KimiParseError> {
    let limits = body
        .get("limits")
        .and_then(|v| v.as_array())
        .ok_or(KimiParseError::MissingRequiredField("limits"))?;

    let first = limits
        .first()
        .and_then(|v| v.get("detail"))
        .ok_or(KimiParseError::MissingRequiredField("limits[0].detail"))?;

    QuotaBucket::from_value_with_optional_used(first)
}

fn parse_weekly_bucket(body: &Value) -> Result<QuotaBucket, KimiParseError> {
    let usage = body
        .get("usage")
        .ok_or(KimiParseError::MissingRequiredField("usage"))?;

    QuotaBucket::from_value(usage)
}

fn parse_purchased_quota(body: &Value) -> Result<PurchasedQuota, KimiParseError> {
    let total = body
        .get("totalQuota")
        .ok_or(KimiParseError::MissingRequiredField("totalQuota"))?;

    Ok(PurchasedQuota {
        limit: parse_u64_field(total, "limit")?,
        remaining: parse_u64_field(total, "remaining")?,
    })
}

impl QuotaBucket {
    fn from_value(value: &Value) -> Result<Self, KimiParseError> {
        Ok(QuotaBucket {
            limit: parse_u64_field(value, "limit")?,
            used: parse_u64_field(value, "used")?,
            remaining: parse_u64_field(value, "remaining")?,
            resets_at: parse_string_field(value, "resetTime")?,
        })
    }

    fn from_value_with_optional_used(value: &Value) -> Result<Self, KimiParseError> {
        let limit = parse_u64_field(value, "limit")?;
        let remaining = parse_u64_field(value, "remaining")?;
        let used =
            optional_u64_field(value, "used").unwrap_or_else(|| limit.saturating_sub(remaining));

        Ok(QuotaBucket {
            limit,
            used,
            remaining,
            resets_at: parse_string_field(value, "resetTime")?,
        })
    }
}

fn parse_u64_field(value: &Value, field: &'static str) -> Result<u64, KimiParseError> {
    value
        .get(field)
        .and_then(|v| {
            if let Some(n) = v.as_u64() {
                Some(n)
            } else if let Some(s) = v.as_str() {
                s.parse().ok()
            } else {
                None
            }
        })
        .ok_or(KimiParseError::UnexpectedFieldType(field))
}

fn parse_string_field(value: &Value, field: &'static str) -> Result<String, KimiParseError> {
    value
        .get(field)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or(KimiParseError::UnexpectedFieldType(field))
}

fn optional_u64_field(value: &Value, field: &'static str) -> Option<u64> {
    value.get(field).and_then(|v| {
        if let Some(n) = v.as_u64() {
            Some(n)
        } else if let Some(s) = v.as_str() {
            s.parse().ok()
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn parses_full_kimi_response() {
        let body = json!({
            "user": { "userId": "u1", "region": "REGION_CN" },
            "usage": { "limit": "100", "used": "5", "remaining": "95", "resetTime": "2026-07-01T17:58:12Z" },
            "limits": [
                {
                    "window": { "duration": 300, "timeUnit": "TIME_UNIT_MINUTE" },
                    "detail": { "limit": "100", "used": "9", "remaining": "91", "resetTime": "2026-06-26T06:58:12Z" }
                }
            ],
            "totalQuota": { "limit": "100", "remaining": "99" }
        });

        let result =
            parse_kimi_usage(&body, "2026-06-26T10:00:00Z".to_string()).expect("should parse");

        assert_eq!(result.quotas.five_hour.limit, 100);
        assert_eq!(result.quotas.five_hour.used, 9);
        assert_eq!(result.quotas.weekly.remaining, 95);
        assert_eq!(result.quotas.purchased.remaining, 99);
        assert_eq!(result.meta.region, "REGION_CN");
    }

    #[test]
    fn parses_numeric_fields_directly() {
        let body = json!({
            "user": { "region": "REGION_CN" },
            "usage": { "limit": 100, "used": 5, "remaining": 95, "resetTime": "2026-07-01T17:58:12Z" },
            "limits": [
                { "detail": { "limit": 100, "used": 9, "remaining": 91, "resetTime": "2026-06-26T06:58:12Z" } }
            ],
            "totalQuota": { "limit": 100, "remaining": 99 }
        });

        let result =
            parse_kimi_usage(&body, "2026-06-26T10:00:00Z".to_string()).expect("should parse");
        assert_eq!(result.quotas.five_hour.remaining, 91);
    }

    #[test]
    fn derives_five_hour_used_when_upstream_omits_it() {
        let body = json!({
            "user": { "region": "REGION_CN" },
            "usage": { "limit": "100", "used": "11", "remaining": "89", "resetTime": "2026-07-01T17:58:12.318501Z" },
            "limits": [
                {
                    "window": { "duration": 300, "timeUnit": "TIME_UNIT_MINUTE" },
                    "detail": { "limit": "100", "remaining": "100", "resetTime": "2026-06-26T11:58:12.318501Z" }
                }
            ],
            "totalQuota": { "limit": "100", "remaining": "99" }
        });

        let result =
            parse_kimi_usage(&body, "2026-06-26T10:00:00Z".to_string()).expect("should parse");

        assert_eq!(result.quotas.five_hour.limit, 100);
        assert_eq!(result.quotas.five_hour.used, 0);
        assert_eq!(result.quotas.five_hour.remaining, 100);
    }
}
