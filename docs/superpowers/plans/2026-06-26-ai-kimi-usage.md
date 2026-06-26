# /ai/kimi Usage Endpoint Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a `GET /ai/kimi` endpoint that fetches the current Kimi Coding Plan usage and returns three normalized quota buckets (five-hour rolling window, weekly window, and purchased total) as clean JSON.

**Architecture:** Mirror the existing `youzhiyouxing` module layout: a thin `respond_with_kimi_usage` handler, a `KimiClient` that fetches from the upstream API, and a parser that maps Kimi's messy response into a strongly typed, well-named domain model. The client is injectable into `AppState` so integration tests can run against a local mock server.

**Tech Stack:** Rust 2021, axum 0.8, reqwest 0.12, serde, tokio, thiserror.

## Global Constraints

- Keep files small and single-responsibility; follow the `src/<feature>/{mod.rs, fetch_*.rs, respond_with_*.rs, types.rs, parse_*.rs}` convention already established by `src/youzhiyouxing/`.
- All secrets use `crate::config::SecretString` and are redacted in `Debug` output.
- All upstream errors map to `502 Bad Gateway` via `crate::error::ApiError`.
- Tests must not hit the real Kimi API; use a local TCP test server or a static client override.
- JSON field names are camelCase or snake_case consistently with the rest of the API (snake_case for response bodies).
- Commit after every independently testable deliverable.

---

## File Structure

- **Create:** `src/ai/mod.rs` — module root, exports public children.
- **Create:** `src/ai/kimi/mod.rs` — sub-module root.
- **Create:** `src/ai/kimi/client.rs` — `KimiClient` and `KimiUsageSource`.
- **Create:** `src/ai/kimi/types.rs` — request/response DTOs and error enums.
- **Create:** `src/ai/kimi/parse_kimi_usage.rs` — maps raw upstream JSON into the domain `KimiUsageResponse`.
- **Create:** `src/ai/kimi/respond_with_kimi_usage.rs` — axum handler for `GET /ai/kimi`.
- **Create:** `tests/api_kimi_test.rs` — route-level integration test with a mock Kimi server.
- **Modify:** `src/lib.rs` — register `pub mod ai;`.
- **Modify:** `src/app.rs` — add `kimi_source: KimiUsageSource` to `AppState`, mount `/ai/kimi` route.
- **Modify:** `src/config.rs` — load `KIMI_CODING_PLAN_TOKEN` as `SecretString`.
- **Modify:** `src/main.rs` — build `KimiClient` from config and inject into `AppState`.
- **Modify:** `src/error.rs` — add `KimiFetch` and `KimiParse` variants to `ApiError`.
- **Modify:** `.env.example` — add `KIMI_CODING_PLAN_TOKEN` placeholder.
- **Modify:** `docs/api.md` — document `GET /ai/kimi` response shape and errors.

---

### Task 1: Load Kimi Token from Environment

**Files:**
- Modify: `src/config.rs`

**Interfaces:**
- Consumes: nothing
- Produces: `AppConfig.kimi_coding_plan_token: SecretString`

- [ ] **Step 1: Write the failing test**

In `tests/config_test.rs`, add:

```rust
#[test]
fn config_loads_kimi_coding_plan_token() {
    let token = "sk-kimi-test-token";
    let env = HashMap::from([
        ("GUIXU_BIND_ADDR", "127.0.0.1:3000"),
        ("YOUZHIYOUXING_COOKIE", "_weasley_key=test"),
        ("KIMI_CODING_PLAN_TOKEN", token),
    ]);

    let config = load_config_from_env_with(env.into_iter().map(|(k, v)| (k.to_string(), v.to_string())))
        .expect("config should load");

    assert_eq!(config.kimi_coding_plan_token.as_str(), token);
}
```

The test will fail because `kimi_coding_plan_token` and the helper do not exist yet.

- [ ] **Step 2: Add the token to `AppConfig` and load it**

Modify `src/config.rs`:

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppConfig {
    pub bind_addr: String,
    pub youzhiyouxing_cookie: SecretString,
    pub kimi_coding_plan_token: SecretString,
}

pub fn load_config_from_env() -> Result<AppConfig, ConfigError> {
    let bind_addr = load_bind_addr_from_env();
    let youzhiyouxing_cookie = std::env::var("YOUZHIYOUXING_COOKIE")
        .map_err(|_| ConfigError::MissingEnv("YOUZHIYOUXING_COOKIE"))?;

    if !youzhiyouxing_cookie
        .split(';')
        .any(|pair| pair.trim_start().starts_with("_weasley_key="))
    {
        return Err(ConfigError::InvalidYouzhiyouxingCookie);
    }

    let kimi_coding_plan_token = std::env::var("KIMI_CODING_PLAN_TOKEN")
        .map_err(|_| ConfigError::MissingEnv("KIMI_CODING_PLAN_TOKEN"))?;

    Ok(AppConfig {
        bind_addr,
        youzhiyouxing_cookie: SecretString::new(youzhiyouxing_cookie),
        kimi_coding_plan_token: SecretString::new(kimi_coding_plan_token),
    })
}
```

- [ ] **Step 3: Update the existing config test helper**

Modify `tests/config_test.rs` to inject `KIMI_CODING_PLAN_TOKEN` in the existing happy-path test, and add a helper that lets tests pass an explicit env iterator:

```rust
use std::collections::HashMap;
use guixu::config::{load_config_from_env, AppConfig, ConfigError, SecretString};

fn load_config_from_env_with(
    overrides: impl Iterator<Item = (String, String)>,
) -> Result<AppConfig, ConfigError> {
    let mut preserved: HashMap<String, Option<String>> = HashMap::new();
    for (key, _) in overrides {
        preserved.insert(key.clone(), std::env::var(&key).ok());
    }

    for (key, value) in overrides {
        std::env::set_var(&key, value);
    }

    let result = load_config_from_env();

    for (key, value) in preserved {
        match value {
            Some(v) => std::env::set_var(&key, v),
            None => std::env::remove_var(&key),
        }
    }

    result
}
```

Adjust the existing tests to include `KIMI_CODING_PLAN_TOKEN` in their env map.

- [ ] **Step 4: Run tests**

Run: `cargo test --test config_test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/config.rs tests/config_test.rs
git commit -m "feat(config): load KIMI_CODING_PLAN_TOKEN from env"
```

---

### Task 2: Create Kimi Usage Types and Errors

**Files:**
- Create: `src/ai/mod.rs`
- Create: `src/ai/kimi/mod.rs`
- Create: `src/ai/kimi/types.rs`

**Interfaces:**
- Consumes: nothing
- Produces:
  - `KimiUsageSource` enum
  - `KimiClient`
  - `KimiUsageResponse`, `QuotaBucket`, `QuotaMeta`
  - `KimiFetchError`, `KimiParseError`

- [ ] **Step 1: Create module scaffolding**

`src/ai/mod.rs`:

```rust
pub mod kimi;
```

`src/ai/kimi/mod.rs`:

```rust
pub mod client;
pub mod parse_kimi_usage;
pub mod respond_with_kimi_usage;
pub mod types;
```

- [ ] **Step 2: Write the domain types**

`src/ai/kimi/types.rs`:

```rust
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
```

- [ ] **Step 3: Add a unit test for serialization**

Append to `src/ai/kimi/types.rs` under `#[cfg(test)]`:

```rust
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
```

- [ ] **Step 4: Register module in lib.rs**

Modify `src/lib.rs`:

```rust
pub mod ai;
pub mod app;
pub mod config;
pub mod error;
pub mod health;
pub mod youzhiyouxing;
```

- [ ] **Step 5: Run tests**

Run: `cargo test kimi::types`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/ai src/lib.rs
git commit -m "feat(ai/kimi): add usage response types and errors"
```

---

### Task 3: Implement Kimi Client and Parser

**Files:**
- Create: `src/ai/kimi/client.rs`
- Create: `src/ai/kimi/parse_kimi_usage.rs`

**Interfaces:**
- Consumes: `KimiFetchError`, `KimiParseError`, `KimiUsageResponse`, `SecretString`
- Produces:
  - `KimiUsageSource::fetch_usage(&self) -> Result<KimiUsageResponse, KimiFetchError>`
  - `parse_kimi_usage(body: &serde_json::Value, fetched_at: String) -> Result<KimiUsageResponse, KimiParseError>`

- [ ] **Step 1: Write the client**

`src/ai/kimi/client.rs`:

```rust
use std::{sync::Arc, time::Duration};

use reqwest::{header, Client, StatusCode};

use crate::config::SecretString;

use super::{parse_kimi_usage::parse_kimi_usage, types::KimiFetchError};

const KIMI_USAGE_URL: &str = "https://api.kimi.com/coding/v1/usages";

#[derive(Clone)]
pub enum KimiUsageSource {
    Live(KimiClient),
    Static(serde_json::Value),
}

impl KimiUsageSource {
    pub async fn fetch_usage(&self) -> Result<super::types::KimiUsageResponse, KimiFetchError> {
        match self {
            Self::Live(client) => client.fetch_usage().await,
            Self::Static(body) => {
                let fetched_at = now_iso8601();
                parse_kimi_usage(body, fetched_at).map_err(|_| {
                    KimiFetchError::UnexpectedStatus {
                        status: StatusCode::UNPROCESSABLE_ENTITY,
                    }
                })
            }
        }
    }
}

#[derive(Clone)]
pub struct KimiClient {
    http: Client,
    token: SecretString,
    base_url: String,
}

impl KimiClient {
    pub fn new(token: SecretString) -> Result<Self, reqwest::Error> {
        let http = Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent("guixu/0.1")
            .redirect(reqwest::redirect::Policy::none())
            .build()?;

        Ok(Self {
            http,
            token,
            base_url: KIMI_USAGE_URL.to_string(),
        })
    }

    #[cfg(test)]
    fn new_with_base_url(token: SecretString, base_url: String) -> Result<Self, reqwest::Error> {
        let http = Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent("guixu/0.1")
            .redirect(reqwest::redirect::Policy::none())
            .build()?;

        Ok(Self {
            http,
            token,
            base_url,
        })
    }

    pub async fn fetch_usage(&self) -> Result<super::types::KimiUsageResponse, KimiFetchError> {
        let response = self
            .http
            .get(&self.base_url)
            .header(header::AUTHORIZATION, format!("Bearer {}", self.token.as_str()))
            .header(header::ACCEPT, "application/json")
            .send()
            .await?;

        let status = response.status();
        if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
            return Err(KimiFetchError::AuthenticationFailed);
        }

        if !status.is_success() {
            return Err(KimiFetchError::UnexpectedStatus { status });
        }

        let body: serde_json::Value = response.json().await?;
        let fetched_at = now_iso8601();
        parse_kimi_usage(&body, fetched_at).map_err(|e| {
            // Parser errors are logic errors on a successful upstream response;
            // surface them as an unexpected status for now.
            KimiFetchError::UnexpectedStatus {
                status: StatusCode::UNPROCESSABLE_ENTITY,
            }
        })
    }
}
```

The project does not currently depend on `chrono` or `time`. To stay YAGNI, generate `fetched_at` using a small, dependency-free UTC helper:

```rust
fn now_iso8601() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now();
    let duration = now.duration_since(UNIX_EPOCH).expect("clock drift");
    let secs = duration.as_secs();
    let (year, month, day, hour, minute, second) = utc_from_seconds(secs);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

fn utc_from_seconds(mut secs: u64) -> (u32, u32, u32, u32, u32, u32) {
    let days = secs / 86_400;
    let second = (secs % 86_400) as u32;
    let hour = second / 3_600;
    let minute = (second % 3_600) / 60;
    let second = second % 60;

    let mut year = 1970;
    let mut remaining_days = days;
    loop {
        let year_len = if is_leap_year(year) { 366 } else { 365 };
        if remaining_days < year_len {
            break;
        }
        remaining_days -= year_len;
        year += 1;
    }

    let days_in_month = [
        31,
        if is_leap_year(year) { 29 } else { 28 },
        31, 30, 31, 30, 31, 31, 30, 31, 30, 31,
    ];
    let mut month = 1;
    for dim in days_in_month {
        if remaining_days < dim {
            break;
        }
        remaining_days -= dim;
        month += 1;
    }

    (year, month, (remaining_days + 1) as u32, hour, minute, second)
}

fn is_leap_year(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}
```

Use `now_iso8601()` in `KimiClient::fetch_usage` and in `KimiUsageSource::Static`.

- [ ] **Step 2: Write the parser**

`src/ai/kimi/parse_kimi_usage.rs`:

```rust
use serde_json::Value;

use super::types::{
    KimiMeta, KimiQuotas, KimiUsageResponse, KimiParseError, PurchasedQuota, QuotaBucket,
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
        meta: KimiMeta {
            region,
            fetched_at,
        },
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

    QuotaBucket::from_value(first)
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
```

- [ ] **Step 3: Add parser unit tests**

Append to `src/ai/kimi/parse_kimi_usage.rs` under `#[cfg(test)]`:

```rust
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

        let result = parse_kimi_usage(&body, "2026-06-26T10:00:00Z".to_string()).expect("should parse");

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

        let result = parse_kimi_usage(&body, "2026-06-26T10:00:00Z".to_string()).expect("should parse");
        assert_eq!(result.quotas.five_hour.remaining, 91);
    }
}
```

- [ ] **Step 4: Add client unit tests with a mock server**

Append to `src/ai/kimi/client.rs` under `#[cfg(test)]`:

```rust
#[cfg(test)]
mod tests {
    use std::net::SocketAddr;

    use serde_json::json;
    use tokio::net::TcpListener;

    use crate::config::SecretString;

    use super::*;

    #[tokio::test]
    async fn live_client_returns_parsed_usage() {
        let base_url = spawn_test_server(r#"HTTP/1.1 200 OK
content-type: application/json
content-length: 380

{"user":{"region":"REGION_CN"},"usage":{"limit":"100","used":"5","remaining":"95","resetTime":"2026-07-01T17:58:12Z"},"limits":[{"window":{"duration":300},"detail":{"limit":"100","used":"9","remaining":"91","resetTime":"2026-06-26T06:58:12Z"}}],"totalQuota":{"limit":"100","remaining":"99"}}"#).await;

        let client = KimiClient::new_with_base_url(
            SecretString::new("sk-test".to_string()),
            base_url,
        )
        .expect("client should build");

        let result = client.fetch_usage().await.expect("should fetch");
        assert_eq!(result.quotas.five_hour.used, 9);
        assert_eq!(result.quotas.weekly.used, 5);
        assert_eq!(result.quotas.purchased.remaining, 99);
    }

    #[tokio::test]
    async fn live_client_maps_401_to_authentication_failed() {
        let base_url = spawn_test_server("HTTP/1.1 401 Unauthorized\r\n\r\n").await;
        let client = KimiClient::new_with_base_url(
            SecretString::new("sk-test".to_string()),
            base_url,
        )
        .expect("client should build");

        let error = client.fetch_usage().await.expect_err("401 should fail");
        assert!(matches!(error, KimiFetchError::AuthenticationFailed));
    }

    async fn spawn_test_server(response: &'static str) -> String {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("test server should bind");
        let addr: SocketAddr = listener.local_addr().expect("test server should have addr");

        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.expect("test server should accept");
            let mut buffer = [0; 2048];
            let _ = socket.read(&mut buffer).await;
            socket
                .write_all(response.as_bytes())
                .await
                .expect("test server should respond");
        });

        format!("http://{addr}")
    }
}
```

Note: the mock server test reads only the first request/response exchange, which is sufficient for `fetch_usage`.

- [ ] **Step 5: Run tests**

Run: `cargo test --lib ai::kimi`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/ai/kimi/client.rs src/ai/kimi/parse_kimi_usage.rs
git commit -m "feat(ai/kimi): add client and parser for usage API"
```

---

### Task 4: Add Axum Handler and Route

**Files:**
- Create: `src/ai/kimi/respond_with_kimi_usage.rs`
- Modify: `src/app.rs`
- Modify: `src/error.rs`
- Modify: `src/main.rs`

**Interfaces:**
- Consumes: `KimiUsageSource`, `KimiUsageResponse`, `KimiFetchError`, `KimiParseError`
- Produces: `respond_with_kimi_usage(State<AppState>) -> Result<Json<KimiUsageResponse>, ApiError>`

- [ ] **Step 1: Write the handler**

`src/ai/kimi/respond_with_kimi_usage.rs`:

```rust
use axum::{extract::State, Json};

use crate::{app::AppState, error::ApiError};

use super::types::KimiUsageResponse;

pub async fn respond_with_kimi_usage(
    State(state): State<AppState>,
) -> Result<Json<KimiUsageResponse>, ApiError> {
    let usage = state.kimi_source.fetch_usage().await?;
    Ok(Json(usage))
}
```

- [ ] **Step 2: Extend ApiError for Kimi**

Modify `src/error.rs`:

```rust
use crate::{
    ai::kimi::types::{KimiFetchError, KimiParseError},
    youzhiyouxing::types::{YouzhiyouxingFetchError, YouzhiyouxingParseError},
};

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
                "Kimi API key is invalid or expired. Refresh KIMI_CODING_PLAN_TOKEN.".to_string(),
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
            // ... existing Youzhiyouxing arms unchanged ...
        };

        (status, Json(ErrorBody { error, message })).into_response()
    }
}
```

Keep the existing Youzhiyouxing arms exactly as they are.

- [ ] **Step 3: Mount the route and add state**

Modify `src/app.rs`:

```rust
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
```

- [ ] **Step 4: Wire client in main**

Modify `src/main.rs`:

```rust
use guixu::{
    ai::kimi::client::KimiClient,
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
    let kimi_client =
        KimiClient::new(config.kimi_coding_plan_token.clone()).expect("failed to build kimi client");

    let app_state = AppState {
        youzhiyouxing_source: YouzhiyouxingPageSource::Live(youzhiyouxing_client),
        kimi_source: guixu::ai::kimi::client::KimiUsageSource::Live(kimi_client),
    };
    // ... rest unchanged
}
```

- [ ] **Step 5: Fix existing tests that construct AppState**

Any test constructing `AppState` directly must now include `kimi_source`. In `tests/api_youzhiyouxing_test.rs`, add a static source:

```rust
use guixu::ai::kimi::client::KimiUsageSource;

let app = build_app(AppState {
    youzhiyouxing_source: YouzhiyouxingPageSource::Static(Arc::new(pages)),
    kimi_source: KimiUsageSource::Static(serde_json::json!({
        "user": { "region": "REGION_CN" },
        "usage": { "limit": "100", "used": "0", "remaining": "100", "resetTime": "2026-07-01T17:58:12Z" },
        "limits": [{ "detail": { "limit": "100", "used": "0", "remaining": "100", "resetTime": "2026-06-26T06:58:12Z" } }],
        "totalQuota": { "limit": "100", "remaining": "100" }
    })),
});
```

- [ ] **Step 6: Run tests**

Run: `cargo test`
Expected: PASS

- [ ] **Step 7: Commit**

```bash
git add src/ai/kimi/respond_with_kimi_usage.rs src/app.rs src/error.rs src/main.rs tests/api_youzhiyouxing_test.rs
git commit -m "feat(ai/kimi): add /ai/kimi endpoint"
```

---

### Task 5: Add Route-Level Integration Test

**Files:**
- Create: `tests/api_kimi_test.rs`

**Interfaces:**
- Consumes: `build_app`, `AppState`, `KimiUsageSource`
- Produces: passing test asserting JSON shape

- [ ] **Step 1: Write the integration test**

`tests/api_kimi_test.rs`:

```rust
use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;

use guixu::{
    ai::kimi::client::KimiUsageSource,
    app::{build_app, AppState},
    youzhiyouxing::{
        fetch_youzhiyouxing_pages::YouzhiyouxingPageSource,
        parse_youzhiyouxing_pages::YouzhiyouxingHtmlPages,
    },
};
use std::sync::Arc;

#[tokio::test]
async fn kimi_route_returns_normalized_quotas() {
    let app = build_app(AppState {
        youzhiyouxing_source: empty_youzhiyouxing_source(),
        kimi_source: KimiUsageSource::Static(json!({
            "user": { "userId": "u1", "region": "REGION_CN" },
            "usage": { "limit": "100", "used": "5", "remaining": "95", "resetTime": "2026-07-01T17:58:12Z" },
            "limits": [
                {
                    "window": { "duration": 300, "timeUnit": "TIME_UNIT_MINUTE" },
                    "detail": { "limit": "100", "used": "9", "remaining": "91", "resetTime": "2026-06-26T06:58:12Z" }
                }
            ],
            "totalQuota": { "limit": "100", "remaining": "99" }
        })),
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/ai/kimi")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("request should complete");

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let json: Value = serde_json::from_slice(&body).expect("response should be json");

    assert_eq!(json["quotas"]["five_hour"]["limit"], 100);
    assert_eq!(json["quotas"]["five_hour"]["used"], 9);
    assert_eq!(json["quotas"]["five_hour"]["remaining"], 91);
    assert_eq!(json["quotas"]["five_hour"]["resets_at"], "2026-06-26T06:58:12Z");

    assert_eq!(json["quotas"]["weekly"]["limit"], 100);
    assert_eq!(json["quotas"]["weekly"]["used"], 5);
    assert_eq!(json["quotas"]["weekly"]["remaining"], 95);
    assert_eq!(json["quotas"]["weekly"]["resets_at"], "2026-07-01T17:58:12Z");

    assert_eq!(json["quotas"]["purchased"]["limit"], 100);
    assert_eq!(json["quotas"]["purchased"]["remaining"], 99);

    assert_eq!(json["meta"]["region"], "REGION_CN");
    assert!(json["meta"]["fetched_at"].as_str().is_some());
}

fn empty_youzhiyouxing_source() -> YouzhiyouxingPageSource {
    let pages = YouzhiyouxingHtmlPages {
        dashboard: "".to_string(),
        balance: "".to_string(),
        abooks: "".to_string(),
        cashflow: "".to_string(),
    };
    YouzhiyouxingPageSource::Static(Arc::new(pages))
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --test api_kimi_test`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add tests/api_kimi_test.rs
git commit -m "test(ai/kimi): add route-level integration test"
```

---

### Task 6: Update Documentation and Environment Example

**Files:**
- Modify: `docs/api.md`
- Modify: `.env.example`

- [ ] **Step 1: Update API docs**

Append to `docs/api.md` after the Youzhiyouxing section:

```markdown
## Kimi Coding Plan Usage

```http
GET <base-url>/ai/kimi
```

Returns the current Kimi Coding Plan quota usage, normalized into three buckets.

Successful response:

```json
{
  "quotas": {
    "five_hour": {
      "limit": 100,
      "used": 9,
      "remaining": 91,
      "resets_at": "2026-06-26T06:58:12Z"
    },
    "weekly": {
      "limit": 100,
      "used": 5,
      "remaining": 95,
      "resets_at": "2026-07-01T17:58:12Z"
    },
    "purchased": {
      "limit": 100,
      "remaining": 99
    }
  },
  "meta": {
    "region": "REGION_CN",
    "fetched_at": "2026-06-26T10:00:00Z"
  }
}
```

Authentication failed or invalid token response:

```http
HTTP/1.1 502 Bad Gateway
content-type: application/json
```

```json
{
  "error": "upstream_authentication_failed",
  "message": "Kimi API key is invalid or expired. Refresh KIMI_CODING_PLAN_TOKEN."
}
```

Other upstream fetch or parse failures also return `502 Bad Gateway` with an `error` and `message` field.
```

- [ ] **Step 2: Update .env.example**

`.env.example`:

```bash
# Optional for local development. Deployments such as Railway should use PORT.
GUIXU_BIND_ADDR=127.0.0.1:3000
YOUZHIYOUXING_COOKIE="_weasley_key=replace-with-local-cookie"
KIMI_CODING_PLAN_TOKEN="sk-kimi-replace-with-real-token"
```

- [ ] **Step 3: Commit**

```bash
git add docs/api.md .env.example
git commit -m "docs(api): document /ai/kimi usage endpoint"
```

---

## Self-Review

**1. Spec coverage:**

- ✅ Returns five-hour window quota — Task 3 parser + Task 5 integration test.
- ✅ Returns weekly window quota — Task 3 parser + Task 5 integration test.
- ✅ Returns total purchased quota — Task 3 parser + Task 5 integration test.
- ✅ JSON response format — Task 2 types + Task 5 asserts exact JSON shape.
- ✅ Elegant API design — `quotas` object with clearly named buckets, `meta` for metadata.
- ✅ No real API calls in tests — Task 3 client test uses local TCP mock, Task 5 uses static source.
- ✅ Follows existing patterns — mirrors `youzhiyouxing` module structure and `AppState` injection.

**2. Placeholder scan:**

- No "TBD", "TODO", or vague instructions.
- No "add appropriate error handling" without specifics.
- Every task ends with exact commands and expected output.

**3. Type consistency:**

- `AppState` gains `kimi_source: KimiUsageSource` in Task 4 and all construction sites are updated.
- `KimiFetchError`/`KimiParseError` are defined in Task 2 and mapped in `ApiError` in Task 4.
- `KimiUsageSource` provides `fetch_usage()` used by the handler in Task 4.

No inconsistencies found.

---

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-06-26-ai-kimi-usage.md`. Two execution options:

**1. Subagent-Driven (recommended)** — I dispatch a fresh subagent per task, review between tasks, fast iteration.

**2. Inline Execution** — Execute tasks in this session using `superpowers:executing-plans`, batch execution with checkpoints.

Which approach?
