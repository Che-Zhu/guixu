# Youzhiyouxing API Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first Rust-based Guixu integration endpoint, `GET /youzhiyouxing`, which fetches logged-in Youzhiyouxing pages using `YOUZHIYOUXING_COOKIE`, parses the initial LiveView HTML, and returns stable JSON for the first dashboard data slice.

**Architecture:** This is a single Rust web service using Axum. The API handler calls a server-only Youzhiyouxing integration client, which performs authenticated HTML GET requests with Reqwest, then parses text anchors from the returned HTML into typed structs. The first version deliberately avoids automatic login, websocket LiveView protocol support, persistent cache, and database storage.

**Tech Stack:** Rust 2021, Tokio, Axum 0.8, Reqwest, Scraper, Serde, Thiserror, Tracing, Dotenvy, Cargo tests.

---

## Scope

Build:

- A Rust crate for the Guixu HTTP API.
- `GET /healthz` for smoke checks.
- `GET /youzhiyouxing` returning JSON from `/dashboard`, `/balance`, `/abooks`, and `/cashflow`.
- Env-based configuration using `YOUZHIYOUXING_COOKIE`.
- Explicit upstream session-expired errors when the cookie no longer authenticates.
- Unit tests for parsing and config behavior.

Do not build:

- Automatic Youzhiyouxing login.
- LiveView websocket client.
- Cookie refresh persistence.
- Multi-user auth.
- Database/cache layer.
- Public frontend UI.

## File Structure

- `Cargo.toml`: Rust crate metadata and dependencies.
- `Cargo.lock`: Locked dependency graph for this Rust application.
- `.env.example`: Safe env names only, no real secrets.
- `src/main.rs`: Binary entrypoint; loads env, builds app, binds listener.
- `src/lib.rs`: Library module exports for tests.
- `src/app.rs`: Axum router construction and shared app state.
- `src/config.rs`: Env loading and validation.
- `src/error.rs`: API and integration error types plus Axum response mapping.
- `src/health/respond_to_health_check.rs`: `GET /healthz` handler.
- `src/youzhiyouxing/respond_with_youzhiyouxing.rs`: `GET /youzhiyouxing` handler.
- `src/youzhiyouxing/fetch_youzhiyouxing_pages.rs`: Authenticated upstream HTML fetch action.
- `src/youzhiyouxing/parse_youzhiyouxing_pages.rs`: HTML/text parsing action.
- `src/youzhiyouxing/types.rs`: Request-independent domain and response structs.
- `tests/fixtures/youzhiyouxing/dashboard.html`: Sanitized fixture created from the verified page shape.
- `tests/fixtures/youzhiyouxing/balance.html`: Sanitized fixture.
- `tests/fixtures/youzhiyouxing/abooks.html`: Sanitized fixture.
- `tests/fixtures/youzhiyouxing/cashflow.html`: Sanitized fixture.
- `tests/parse_youzhiyouxing_pages_test.rs`: Parser tests against fixtures.
- `tests/config_test.rs`: Env/config tests.
- `tests/api_youzhiyouxing_test.rs`: Handler behavior tests using a static in-memory HTML source.
- `reference/youzhiyouxing/yx-dashboard-data-source-summary.md`: Existing research note; update only if implementation proves a documented assumption wrong.

## Public API Contract

Request:

```http
GET /youzhiyouxing
```

Successful response:

```json
{
  "dashboard": {
    "family_total_assets": 123456.78,
    "asset_change": -1234.56,
    "debt_ratio": 12.34,
    "cashflow_configured": false
  },
  "balance": {
    "net_assets": 100000.00,
    "total_assets": 123456.78,
    "total_liabilities": 23456.78
  },
  "investment": {
    "total_assets": 80000.00,
    "accumulated_profit": -789.01,
    "money_weighted_return": null
  },
  "cashflow": {
    "configured": false
  }
}
```

If a field cannot be parsed from an authenticated page, use `null` for optional fields. Required first-version fields are `dashboard.family_total_assets`, `dashboard.debt_ratio`, `balance.net_assets`, `balance.total_assets`, and `balance.total_liabilities`.

Error response for expired or invalid cookie:

```json
{
  "error": "upstream_session_expired",
  "message": "Youzhiyouxing session is expired or invalid. Refresh YOUZHIYOUXING_COOKIE."
}
```

Status code: `502 Bad Gateway`.

---

### Task 1: Create Rust Crate Skeleton

**Files:**
- Create: `Cargo.toml`
- Create: `Cargo.lock`
- Create: `src/main.rs`
- Create: `src/lib.rs`
- Create: `src/app.rs`
- Create: `src/health/respond_to_health_check.rs`
- Create: `src/health/mod.rs`
- Modify: `.gitignore`
- Create: `.env.example`

- [ ] **Step 1: Create the crate manifest**

Create `Cargo.toml`:

```toml
[package]
name = "guixu"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = "0.8"
dotenvy = "0.15"
reqwest = { version = "0.12", features = ["rustls-tls", "charset"], default-features = false }
scraper = "0.23"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tokio = { version = "1", features = ["macros", "rt-multi-thread", "net"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

[dev-dependencies]
serial_test = "3"
tower = { version = "0.5", features = ["util"] }
```

- [ ] **Step 2: Create the health handler module**

Create `src/health/mod.rs`:

```rust
pub mod respond_to_health_check;
```

Create `src/health/respond_to_health_check.rs`:

```rust
pub async fn respond_to_health_check() -> &'static str {
    "ok"
}
```

- [ ] **Step 3: Create the app router**

Create `src/app.rs`:

```rust
use axum::{routing::get, Router};

use crate::health::respond_to_health_check::respond_to_health_check;

pub fn build_app() -> Router {
    Router::new().route("/healthz", get(respond_to_health_check))
}
```

- [ ] **Step 4: Create library exports**

Create `src/lib.rs`:

```rust
pub mod app;
pub mod health;
```

- [ ] **Step 5: Create the binary entrypoint**

Create `src/main.rs`:

```rust
use guixu::app::build_app;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "guixu=info,tower_http=info".to_string()),
        )
        .init();

    let bind_addr = std::env::var("GUIXU_BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".to_string());
    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .expect("failed to bind GUIXU_BIND_ADDR");

    tracing::info!(%bind_addr, "starting guixu");
    axum::serve(listener, build_app())
        .await
        .expect("server failed");
}
```

- [ ] **Step 6: Add safe env example**

Create `.env.example`:

```env
GUIXU_BIND_ADDR=127.0.0.1:3000
YOUZHIYOUXING_COOKIE="_weasley_key=replace-with-local-cookie"
```

- [ ] **Step 7: Keep local env ignored**

Ensure `.gitignore` contains exactly these env rules:

```gitignore
.env
.env.*
!.env.example
```

- [ ] **Step 8: Format and verify skeleton**

Run:

```bash
cargo fmt
cargo test
```

Expected: command succeeds with zero tests or only compile checks.

- [ ] **Step 9: Commit skeleton**

Run:

```bash
git add Cargo.toml Cargo.lock .env.example .gitignore src
git commit -m "feat: scaffold rust api service"
```

### Task 2: Add Configuration Loading

**Files:**
- Create: `src/config.rs`
- Modify: `src/lib.rs`
- Modify: `src/main.rs`
- Test: `tests/config_test.rs`

- [ ] **Step 1: Write failing config tests**

Create `tests/config_test.rs`:

```rust
use serial_test::serial;

use guixu::config::{load_config_from_env, ConfigError};

fn clear_env() {
    std::env::remove_var("GUIXU_BIND_ADDR");
    std::env::remove_var("YOUZHIYOUXING_COOKIE");
}

#[test]
#[serial]
fn loads_required_youzhiyouxing_cookie() {
    clear_env();
    std::env::set_var("YOUZHIYOUXING_COOKIE", "_weasley_key=abc123");

    let config = load_config_from_env().expect("config should load");

    assert_eq!(config.bind_addr, "127.0.0.1:3000");
    assert_eq!(config.youzhiyouxing_cookie.expose_for_test(), "_weasley_key=abc123");
}

#[test]
#[serial]
fn rejects_missing_youzhiyouxing_cookie() {
    clear_env();

    let error = load_config_from_env().expect_err("missing cookie should fail");

    assert_eq!(error, ConfigError::MissingEnv("YOUZHIYOUXING_COOKIE"));
}

#[test]
#[serial]
fn rejects_cookie_without_weasley_key_name() {
    clear_env();
    std::env::set_var("YOUZHIYOUXING_COOKIE", "abc123");

    let error = load_config_from_env().expect_err("raw cookie value should fail");

    assert_eq!(error, ConfigError::InvalidYouzhiyouxingCookie);
}
```

- [ ] **Step 2: Run tests to verify failure**

Run:

```bash
cargo test --test config_test
```

Expected: FAIL because `guixu::config` does not exist.

- [ ] **Step 3: Implement config module**

Create `src/config.rs`:

```rust
#[derive(Clone, PartialEq, Eq)]
pub struct SecretString(String);

impl SecretString {
    pub fn new(value: String) -> Self {
        Self(value)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn expose_for_test(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for SecretString {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("SecretString([redacted])")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppConfig {
    pub bind_addr: String,
    pub youzhiyouxing_cookie: SecretString,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ConfigError {
    #[error("missing required env var: {0}")]
    MissingEnv(&'static str),
    #[error("YOUZHIYOUXING_COOKIE must include _weasley_key=...")]
    InvalidYouzhiyouxingCookie,
}

pub fn load_config_from_env() -> Result<AppConfig, ConfigError> {
    let bind_addr = std::env::var("GUIXU_BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".to_string());
    let youzhiyouxing_cookie = std::env::var("YOUZHIYOUXING_COOKIE")
        .map_err(|_| ConfigError::MissingEnv("YOUZHIYOUXING_COOKIE"))?;

    if !youzhiyouxing_cookie
        .split(';')
        .any(|pair| pair.trim_start().starts_with("_weasley_key="))
    {
        return Err(ConfigError::InvalidYouzhiyouxingCookie);
    }

    Ok(AppConfig {
        bind_addr,
        youzhiyouxing_cookie: SecretString::new(youzhiyouxing_cookie),
    })
}
```

- [ ] **Step 4: Export config module**

Modify `src/lib.rs`:

```rust
pub mod app;
pub mod config;
pub mod health;
```

- [ ] **Step 5: Use config in main**

Modify `src/main.rs`:

```rust
use guixu::{app::build_app, config::load_config_from_env};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "guixu=info,tower_http=info".to_string()),
        )
        .init();

    let config = load_config_from_env().expect("invalid guixu configuration");
    let listener = tokio::net::TcpListener::bind(&config.bind_addr)
        .await
        .expect("failed to bind GUIXU_BIND_ADDR");

    tracing::info!(bind_addr = %config.bind_addr, "starting guixu");
    axum::serve(listener, build_app())
        .await
        .expect("server failed");
}
```

- [ ] **Step 6: Run config tests**

Run:

```bash
cargo test --test config_test
```

Expected: PASS.

- [ ] **Step 7: Commit config**

Run:

```bash
git add src/config.rs src/lib.rs src/main.rs tests/config_test.rs
git commit -m "feat: load guixu configuration from env"
```

### Task 3: Define Youzhiyouxing Types and Parser

**Files:**
- Create: `src/youzhiyouxing/mod.rs`
- Create: `src/youzhiyouxing/types.rs`
- Create: `src/youzhiyouxing/parse_youzhiyouxing_pages.rs`
- Modify: `src/lib.rs`
- Create: `tests/fixtures/youzhiyouxing/dashboard.html`
- Create: `tests/fixtures/youzhiyouxing/balance.html`
- Create: `tests/fixtures/youzhiyouxing/abooks.html`
- Create: `tests/fixtures/youzhiyouxing/cashflow.html`
- Test: `tests/parse_youzhiyouxing_pages_test.rs`

- [ ] **Step 1: Create sanitized fixtures**

Create `tests/fixtures/youzhiyouxing/dashboard.html`:

```html
<!doctype html>
<html>
  <head><title>家庭财务总览 · 有知有行</title></head>
  <body>
    <div data-phx-main>
      <p>退出</p>
      <h1>家庭总资产</h1>
      <p>6月摘要</p>
      <p>123,456.78</p>
      <p>元</p>
      <p>资产减少</p>
      <p>1,234.56 元</p>
      <p>财务晴雨表</p>
      <p>资产负债率</p>
      <p>12.34</p>
      <p>%</p>
      <p>年度现金流</p>
      <p>预估年度现金流 &gt;</p>
    </div>
  </body>
</html>
```

Create `tests/fixtures/youzhiyouxing/balance.html`:

```html
<!doctype html>
<html>
  <head><title>家庭资产记账 · 有知有行</title></head>
  <body>
    <main data-phx-main>
      <p>退出</p>
      <p>净资产</p>
      <p>100,000.00</p>
      <p>资产总额</p>
      <p>123,456.78</p>
      <p>负债总额</p>
      <p>23,456.78</p>
      <p>年度现金流</p>
    </main>
  </body>
</html>
```

Create `tests/fixtures/youzhiyouxing/abooks.html`:

```html
<!doctype html>
<html>
  <head><title>投资记账 · 有知有行</title></head>
  <body>
    <main data-phx-main>
      <p>退出</p>
      <p>总资产</p>
      <p>80,000.00</p>
      <p>累计收益</p>
      <p>-789.01</p>
      <p>资金加权收益率</p>
      <p>-1.63</p>
      <p>%</p>
    </main>
  </body>
</html>
```

Create `tests/fixtures/youzhiyouxing/cashflow.html`:

```html
<!doctype html>
<html>
  <head><title>年度现金流 · 有知有行</title></head>
  <body>
    <main data-phx-main>
      <p>退出</p>
      <p>年度现金流（2026）</p>
      <p>进入现金流预估</p>
    </main>
  </body>
</html>
```

- [ ] **Step 2: Write failing parser tests**

Create `tests/parse_youzhiyouxing_pages_test.rs`:

```rust
use guixu::youzhiyouxing::{
    parse_youzhiyouxing_pages::{parse_youzhiyouxing_pages, YouzhiyouxingHtmlPages},
    types::YouzhiyouxingParseError,
};

#[test]
fn parses_sanitized_youzhiyouxing_pages() {
    let pages = YouzhiyouxingHtmlPages {
        dashboard: include_str!("fixtures/youzhiyouxing/dashboard.html").to_string(),
        balance: include_str!("fixtures/youzhiyouxing/balance.html").to_string(),
        abooks: include_str!("fixtures/youzhiyouxing/abooks.html").to_string(),
        cashflow: include_str!("fixtures/youzhiyouxing/cashflow.html").to_string(),
    };

    let parsed = parse_youzhiyouxing_pages(&pages).expect("fixtures should parse");

    assert_eq!(parsed.dashboard.family_total_assets, 123_456.78);
    assert_eq!(parsed.dashboard.asset_change, Some(-1_234.56));
    assert_eq!(parsed.dashboard.debt_ratio, 12.34);
    assert!(!parsed.dashboard.cashflow_configured);
    assert_eq!(parsed.balance.net_assets, 100_000.00);
    assert_eq!(parsed.balance.total_assets, 123_456.78);
    assert_eq!(parsed.balance.total_liabilities, 23_456.78);
    assert_eq!(parsed.investment.total_assets, Some(80_000.00));
    assert_eq!(parsed.investment.accumulated_profit, Some(-789.01));
    assert_eq!(parsed.investment.money_weighted_return, Some(-1.63));
    assert!(!parsed.cashflow.configured);
}

#[test]
fn rejects_login_page_html() {
    let pages = YouzhiyouxingHtmlPages {
        dashboard: "<html><body>做聪明的投资者 登录</body></html>".to_string(),
        balance: include_str!("fixtures/youzhiyouxing/balance.html").to_string(),
        abooks: include_str!("fixtures/youzhiyouxing/abooks.html").to_string(),
        cashflow: include_str!("fixtures/youzhiyouxing/cashflow.html").to_string(),
    };

    let error = parse_youzhiyouxing_pages(&pages).expect_err("login page should be rejected");

    assert_eq!(error, YouzhiyouxingParseError::SessionExpired);
}

#[test]
fn reports_missing_required_field() {
    let pages = YouzhiyouxingHtmlPages {
        dashboard: "<html><body><p>退出</p><p>家庭总资产</p></body></html>".to_string(),
        balance: include_str!("fixtures/youzhiyouxing/balance.html").to_string(),
        abooks: include_str!("fixtures/youzhiyouxing/abooks.html").to_string(),
        cashflow: include_str!("fixtures/youzhiyouxing/cashflow.html").to_string(),
    };

    let error = parse_youzhiyouxing_pages(&pages).expect_err("missing number should fail");

    assert_eq!(
        error,
        YouzhiyouxingParseError::MissingRequiredField("dashboard.family_total_assets")
    );
}
```

- [ ] **Step 3: Run parser tests to verify failure**

Run:

```bash
cargo test --test parse_youzhiyouxing_pages_test
```

Expected: FAIL because `guixu::youzhiyouxing` does not exist.

- [ ] **Step 4: Create Youzhiyouxing module exports**

Create `src/youzhiyouxing/mod.rs`:

```rust
pub mod parse_youzhiyouxing_pages;
pub mod types;
```

Modify `src/lib.rs`:

```rust
pub mod app;
pub mod config;
pub mod health;
pub mod youzhiyouxing;
```

- [ ] **Step 5: Define response and parse types**

Create `src/youzhiyouxing/types.rs`:

```rust
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
```

- [ ] **Step 6: Implement parser**

Create `src/youzhiyouxing/parse_youzhiyouxing_pages.rs`:

```rust
use scraper::Html;

use super::types::{
    BalanceSummary, CashflowSummary, DashboardSummary, InvestmentSummary, YouzhiyouxingParseError,
    YouzhiyouxingResponse,
};

#[derive(Debug, Clone)]
pub struct YouzhiyouxingHtmlPages {
    pub dashboard: String,
    pub balance: String,
    pub abooks: String,
    pub cashflow: String,
}

pub fn parse_youzhiyouxing_pages(
    pages: &YouzhiyouxingHtmlPages,
) -> Result<YouzhiyouxingResponse, YouzhiyouxingParseError> {
    let dashboard_text = html_to_normalized_text(&pages.dashboard);
    let balance_text = html_to_normalized_text(&pages.balance);
    let abooks_text = html_to_normalized_text(&pages.abooks);
    let cashflow_text = html_to_normalized_text(&pages.cashflow);

    for text in [&dashboard_text, &balance_text, &abooks_text, &cashflow_text] {
        reject_login_page(text)?;
    }

    let dashboard = DashboardSummary {
        family_total_assets: number_after(&dashboard_text, "家庭总资产")
            .ok_or(YouzhiyouxingParseError::MissingRequiredField(
                "dashboard.family_total_assets",
            ))?,
        asset_change: signed_number_after(&dashboard_text, "资产减少").map(|value| -value),
        debt_ratio: number_after(&dashboard_text, "资产负债率").ok_or(
            YouzhiyouxingParseError::MissingRequiredField("dashboard.debt_ratio"),
        )?,
        cashflow_configured: !dashboard_text.contains("预估年度现金流 >")
            && !cashflow_text.contains("进入现金流预估"),
    };

    let balance = BalanceSummary {
        net_assets: number_after(&balance_text, "净资产").ok_or(
            YouzhiyouxingParseError::MissingRequiredField("balance.net_assets"),
        )?,
        total_assets: number_after(&balance_text, "资产总额").ok_or(
            YouzhiyouxingParseError::MissingRequiredField("balance.total_assets"),
        )?,
        total_liabilities: number_after(&balance_text, "负债总额").ok_or(
            YouzhiyouxingParseError::MissingRequiredField("balance.total_liabilities"),
        )?,
    };

    let investment = InvestmentSummary {
        total_assets: number_after(&abooks_text, "总资产"),
        accumulated_profit: signed_number_after(&abooks_text, "累计收益"),
        money_weighted_return: signed_number_after(&abooks_text, "资金加权收益率"),
    };

    let cashflow = CashflowSummary {
        configured: !cashflow_text.contains("进入现金流预估"),
    };

    Ok(YouzhiyouxingResponse {
        dashboard,
        balance,
        investment,
        cashflow,
    })
}

fn reject_login_page(text: &str) -> Result<(), YouzhiyouxingParseError> {
    if text.contains("做聪明的投资者") || text.contains("登录") && !text.contains("退出") {
        return Err(YouzhiyouxingParseError::SessionExpired);
    }

    Ok(())
}

fn html_to_normalized_text(html: &str) -> String {
    let document = Html::parse_document(html);
    document
        .root_element()
        .text()
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn number_after(text: &str, anchor: &str) -> Option<f64> {
    signed_number_after(text, anchor).map(f64::abs)
}

fn signed_number_after(text: &str, anchor: &str) -> Option<f64> {
    let after_anchor = text.split_once(anchor)?.1;
    for token in after_anchor.split_whitespace() {
        if let Some(value) = parse_number_token(token) {
            return Some(value);
        }
    }

    None
}

fn parse_number_token(token: &str) -> Option<f64> {
    let candidate = token
        .trim()
        .trim_end_matches('元')
        .trim_end_matches('%')
        .trim();
    let cleaned = candidate.replace(',', "");

    if cleaned.is_empty()
        || cleaned == "-"
        || cleaned == "."
        || !cleaned.chars().any(|ch| ch.is_ascii_digit())
        || !cleaned
            .chars()
            .all(|ch| ch.is_ascii_digit() || ch == '-' || ch == '.')
    {
        return None;
    }

    cleaned.parse::<f64>().ok()
}
```

- [ ] **Step 7: Run parser tests**

Run:

```bash
cargo test --test parse_youzhiyouxing_pages_test
```

Expected: PASS.

- [ ] **Step 8: Commit parser**

Run:

```bash
git add src/lib.rs src/youzhiyouxing tests/fixtures/youzhiyouxing tests/parse_youzhiyouxing_pages_test.rs
git commit -m "feat: parse youzhiyouxing html fixtures"
```

### Task 4: Add Authenticated Youzhiyouxing Fetcher

**Files:**
- Create: `src/youzhiyouxing/fetch_youzhiyouxing_pages.rs`
- Modify: `src/youzhiyouxing/mod.rs`
- Modify: `src/youzhiyouxing/types.rs`

- [ ] **Step 1: Add upstream error type**

Modify `src/youzhiyouxing/types.rs` to append this enum below `YouzhiyouxingParseError`:

```rust
#[derive(Debug, thiserror::Error)]
pub enum YouzhiyouxingFetchError {
    #[error("youzhiyouxing request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("youzhiyouxing session expired")]
    SessionExpired,
    #[error("youzhiyouxing returned unexpected status: {0}")]
    UnexpectedStatus(reqwest::StatusCode),
}
```

- [ ] **Step 2: Implement the fetch action**

Create `src/youzhiyouxing/fetch_youzhiyouxing_pages.rs`:

```rust
use std::{sync::Arc, time::Duration};

use reqwest::{header, Client, StatusCode};

use crate::config::SecretString;

use super::{
    parse_youzhiyouxing_pages::YouzhiyouxingHtmlPages,
    types::YouzhiyouxingFetchError,
};

const BASE_URL: &str = "https://yx.youzhiyouxing.cn";

#[derive(Clone)]
pub enum YouzhiyouxingPageSource {
    Live(YouzhiyouxingClient),
    Static(Arc<YouzhiyouxingHtmlPages>),
}

impl YouzhiyouxingPageSource {
    pub async fn fetch_pages(&self) -> Result<YouzhiyouxingHtmlPages, YouzhiyouxingFetchError> {
        match self {
            Self::Live(client) => client.fetch_pages().await,
            Self::Static(pages) => Ok((**pages).clone()),
        }
    }
}

#[derive(Clone)]
pub struct YouzhiyouxingClient {
    http: Client,
    cookie: SecretString,
}

impl YouzhiyouxingClient {
    pub fn new(cookie: SecretString) -> Result<Self, reqwest::Error> {
        let http = Client::builder()
            .timeout(Duration::from_secs(20))
            .user_agent("guixu/0.1")
            .redirect(reqwest::redirect::Policy::none())
            .build()?;

        Ok(Self { http, cookie })
    }

    pub async fn fetch_pages(&self) -> Result<YouzhiyouxingHtmlPages, YouzhiyouxingFetchError> {
        Ok(YouzhiyouxingHtmlPages {
            dashboard: self.fetch_html("/dashboard").await?,
            balance: self.fetch_html("/balance").await?,
            abooks: self.fetch_html("/abooks").await?,
            cashflow: self.fetch_html("/cashflow").await?,
        })
    }

    async fn fetch_html(&self, path: &str) -> Result<String, YouzhiyouxingFetchError> {
        let response = self
            .http
            .get(format!("{BASE_URL}{path}"))
            .header(header::COOKIE, self.cookie.as_str())
            .header(header::ACCEPT, "text/html,application/xhtml+xml")
            .send()
            .await?;

        if response.status() == StatusCode::FOUND || response.status() == StatusCode::SEE_OTHER {
            return Err(YouzhiyouxingFetchError::SessionExpired);
        }

        if !response.status().is_success() {
            return Err(YouzhiyouxingFetchError::UnexpectedStatus(response.status()));
        }

        let html = response.text().await?;
        if html.contains("做聪明的投资者") && html.contains("登录") {
            return Err(YouzhiyouxingFetchError::SessionExpired);
        }

        Ok(html)
    }
}
```

- [ ] **Step 3: Export fetcher**

Modify `src/youzhiyouxing/mod.rs`:

```rust
pub mod fetch_youzhiyouxing_pages;
pub mod parse_youzhiyouxing_pages;
pub mod types;
```

- [ ] **Step 4: Compile**

Run:

```bash
cargo test
```

Expected: PASS.

- [ ] **Step 5: Optional local authenticated smoke check**

Run only on a machine with `.env` containing a valid local `YOUZHIYOUXING_COOKIE`:

```bash
cargo test --test parse_youzhiyouxing_pages_test
```

Expected: PASS. This does not hit the network; the real upstream smoke check is done after the API route exists.

- [ ] **Step 6: Commit fetcher**

Run:

```bash
git add src/youzhiyouxing
git commit -m "feat: fetch youzhiyouxing html pages"
```

### Task 5: Add API Error Mapping

**Files:**
- Create: `src/error.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Create API error response type**

Create `src/error.rs`:

```rust
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

use crate::youzhiyouxing::types::{YouzhiyouxingFetchError, YouzhiyouxingParseError};

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub error: &'static str,
    pub message: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error(transparent)]
    YouzhiyouxingFetch(#[from] YouzhiyouxingFetchError),
    #[error(transparent)]
    YouzhiyouxingParse(#[from] YouzhiyouxingParseError),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error, message) = match self {
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
```

- [ ] **Step 2: Export error module**

Modify `src/lib.rs`:

```rust
pub mod app;
pub mod config;
pub mod error;
pub mod health;
pub mod youzhiyouxing;
```

- [ ] **Step 3: Compile**

Run:

```bash
cargo test
```

Expected: PASS.

- [ ] **Step 4: Commit API errors**

Run:

```bash
git add src/error.rs src/lib.rs
git commit -m "feat: map integration failures to api errors"
```

### Task 6: Add GET /youzhiyouxing Handler

**Files:**
- Create: `src/youzhiyouxing/respond_with_youzhiyouxing.rs`
- Modify: `src/youzhiyouxing/mod.rs`
- Modify: `src/app.rs`
- Modify: `src/main.rs`
- Test: `tests/api_youzhiyouxing_test.rs`

- [ ] **Step 1: Write failing API route test**

Create `tests/api_youzhiyouxing_test.rs`:

```rust
use std::sync::Arc;

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use serde_json::Value;
use tower::ServiceExt;

use guixu::{
    app::{build_app, AppState},
    youzhiyouxing::{
        fetch_youzhiyouxing_pages::YouzhiyouxingPageSource,
        parse_youzhiyouxing_pages::YouzhiyouxingHtmlPages,
    },
};

#[tokio::test]
async fn youzhiyouxing_route_returns_json() {
    let pages = YouzhiyouxingHtmlPages {
        dashboard: include_str!("fixtures/youzhiyouxing/dashboard.html").to_string(),
        balance: include_str!("fixtures/youzhiyouxing/balance.html").to_string(),
        abooks: include_str!("fixtures/youzhiyouxing/abooks.html").to_string(),
        cashflow: include_str!("fixtures/youzhiyouxing/cashflow.html").to_string(),
    };
    let app = build_app(AppState {
        youzhiyouxing_source: YouzhiyouxingPageSource::Static(Arc::new(pages)),
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/youzhiyouxing")
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

    assert_eq!(json["dashboard"]["family_total_assets"], 123456.78);
    assert_eq!(json["balance"]["net_assets"], 100000.00);
    assert_eq!(json["cashflow"]["configured"], false);
}
```

- [ ] **Step 2: Run API route test to verify failure**

Run:

```bash
cargo test --test api_youzhiyouxing_test
```

Expected: FAIL because `AppState`, `YouzhiyouxingPageSource`, and the `/youzhiyouxing` route do not exist.

- [ ] **Step 3: Add handler**

Create `src/youzhiyouxing/respond_with_youzhiyouxing.rs`:

```rust
use axum::{extract::State, Json};

use crate::{app::AppState, error::ApiError};

use super::{
    parse_youzhiyouxing_pages::parse_youzhiyouxing_pages,
    types::YouzhiyouxingResponse,
};

pub async fn respond_with_youzhiyouxing(
    State(state): State<AppState>,
) -> Result<Json<YouzhiyouxingResponse>, ApiError> {
    let pages = state.youzhiyouxing_source.fetch_pages().await?;
    let response = parse_youzhiyouxing_pages(&pages)?;

    Ok(Json(response))
}
```

- [ ] **Step 4: Export handler**

Modify `src/youzhiyouxing/mod.rs`:

```rust
pub mod fetch_youzhiyouxing_pages;
pub mod parse_youzhiyouxing_pages;
pub mod respond_with_youzhiyouxing;
pub mod types;
```

- [ ] **Step 5: Add app state and route**

Modify `src/app.rs`:

```rust
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
```

- [ ] **Step 6: Wire state in main**

Modify `src/main.rs`:

```rust
use guixu::{
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
    let app_state = AppState {
        youzhiyouxing_source: YouzhiyouxingPageSource::Live(youzhiyouxing_client),
    };
    let listener = tokio::net::TcpListener::bind(&config.bind_addr)
        .await
        .expect("failed to bind GUIXU_BIND_ADDR");

    tracing::info!(bind_addr = %config.bind_addr, "starting guixu");
    axum::serve(listener, build_app(app_state))
        .await
        .expect("server failed");
}
```

- [ ] **Step 7: Run tests**

Run:

```bash
cargo test
```

Expected: PASS. The route test must not call the real upstream because it uses `YouzhiyouxingPageSource::Static`.

- [ ] **Step 8: Run local upstream smoke test**

Run this only with valid local `.env`:

```bash
cargo run
```

In a second terminal:

```bash
curl -sS http://127.0.0.1:3000/youzhiyouxing
```

Expected: JSON response with top-level keys `dashboard`, `balance`, `investment`, and `cashflow`. Do not paste real financial values into commits or issue comments.

- [ ] **Step 9: Commit route**

Run:

```bash
git add src tests/api_youzhiyouxing_test.rs
git commit -m "feat: expose youzhiyouxing api route"
```

### Task 7: Add Minimal Documentation Updates

**Files:**
- Modify: `README.md`
- Modify: `reference/youzhiyouxing/yx-dashboard-data-source-summary.md`

- [ ] **Step 1: Update README project status only**

Modify the `Project status` section in `README.md` so it no longer points at missing `tests/youzhiyouxing/yx-dashboard-data-source-summary.md`. Use this replacement paragraph:

```markdown
Early development. The first Rust API surface is being built around Youzhiyouxing (有知有行), a finance dashboard backed by Phoenix LiveView rather than a clean JSON API. Research notes live in [`reference/youzhiyouxing/yx-dashboard-data-source-summary.md`](reference/youzhiyouxing/yx-dashboard-data-source-summary.md).
```

Do not change the README positioning from private backend to full-stack app.

- [ ] **Step 2: Add implementation note to research doc**

Append this section to `reference/youzhiyouxing/yx-dashboard-data-source-summary.md`:

```markdown
## Implementation status

The first Guixu implementation uses Rust, Reqwest, and server-side HTML parsing. Local validation on 2026-06-25 confirmed that a `YOUZHIYOUXING_COOKIE` value containing `_weasley_key=...` can fetch `/dashboard`, `/balance`, `/abooks`, and `/cashflow` as authenticated HTML without using the LiveView websocket.
```

- [ ] **Step 3: Run tests**

Run:

```bash
cargo test
```

Expected: PASS.

- [ ] **Step 4: Commit docs**

Run:

```bash
git add README.md reference/youzhiyouxing/yx-dashboard-data-source-summary.md
git commit -m "docs: align youzhiyouxing implementation notes"
```

### Task 8: Final Verification

**Files:**
- No file changes expected.

- [ ] **Step 1: Format**

Run:

```bash
cargo fmt --check
```

Expected: PASS.

- [ ] **Step 2: Run full tests**

Run:

```bash
cargo test
```

Expected: PASS.

- [ ] **Step 3: Start server with local env**

Run:

```bash
cargo run
```

Expected log includes `starting guixu` and bind address `127.0.0.1:3000`.

- [ ] **Step 4: Check health endpoint**

Run:

```bash
curl -i http://127.0.0.1:3000/healthz
```

Expected:

```http
HTTP/1.1 200 OK
```

Body:

```text
ok
```

- [ ] **Step 5: Check Youzhiyouxing endpoint**

Run:

```bash
curl -sS http://127.0.0.1:3000/youzhiyouxing | jq 'keys'
```

Expected:

```json
[
  "balance",
  "cashflow",
  "dashboard",
  "investment"
]
```

- [ ] **Step 6: Check expired-cookie behavior**

Temporarily run with an invalid cookie without editing `.env`:

```bash
YOUZHIYOUXING_COOKIE="_weasley_key=invalid" GUIXU_BIND_ADDR="127.0.0.1:3001" cargo run
```

In a second terminal:

```bash
curl -sS -i http://127.0.0.1:3001/youzhiyouxing
```

Expected status: `502 Bad Gateway`.

Expected JSON includes:

```json
{
  "error": "upstream_session_expired"
}
```

- [ ] **Step 7: Inspect git diff**

Run:

```bash
git status --short
git log --oneline main..HEAD
```

Expected: working tree clean after commits; branch contains focused commits for scaffold, config, parser, fetcher, route, docs.

## Self-Review

- Spec coverage: The plan covers Rust service scaffold, env cookie loading, authenticated upstream fetch, initial HTML parsing, `GET /youzhiyouxing`, expired-cookie error behavior, and docs alignment.
- Placeholder scan: No task uses unresolved placeholder markers. Each code-writing step includes concrete file content.
- Type consistency: `AppState`, `YouzhiyouxingClient`, `YouzhiyouxingHtmlPages`, `YouzhiyouxingResponse`, `ApiError`, and parse/fetch error names are consistent across tasks.
- Risk: The parser is intentionally text-anchor based. If upstream HTML shape differs from sanitized fixtures, Task 8 smoke testing will expose it; the next smallest correction is to adjust fixtures and parser anchors, not to jump to LiveView websocket.
