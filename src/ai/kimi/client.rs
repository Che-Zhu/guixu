use std::time::Duration;

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
                parse_kimi_usage(body, fetched_at).map_err(|_| KimiFetchError::UnexpectedStatus {
                    status: StatusCode::UNPROCESSABLE_ENTITY,
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
            .header(
                header::AUTHORIZATION,
                format!("Bearer {}", self.token.as_str()),
            )
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

        let body_text = response.text().await?;
        let body: serde_json::Value = match serde_json::from_str(&body_text) {
            Ok(value) => value,
            Err(_) => {
                return Err(KimiFetchError::UnexpectedStatus {
                    status: StatusCode::UNPROCESSABLE_ENTITY,
                });
            }
        };
        let fetched_at = now_iso8601();
        parse_kimi_usage(&body, fetched_at).map_err(|_| {
            // Parser errors are logic errors on a successful upstream response;
            // surface them as an unexpected status for now.
            KimiFetchError::UnexpectedStatus {
                status: StatusCode::UNPROCESSABLE_ENTITY,
            }
        })
    }
}

fn now_iso8601() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now();
    let duration = now.duration_since(UNIX_EPOCH).expect("clock drift");
    let secs = duration.as_secs();
    let (year, month, day, hour, minute, second) = utc_from_seconds(secs);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

fn utc_from_seconds(secs: u64) -> (u32, u32, u32, u32, u32, u32) {
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
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut month = 1;
    for dim in days_in_month {
        if remaining_days < dim {
            break;
        }
        remaining_days -= dim;
        month += 1;
    }

    (
        year,
        month,
        (remaining_days + 1) as u32,
        hour,
        minute,
        second,
    )
}

fn is_leap_year(year: u32) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;

    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
    };

    use crate::config::SecretString;

    use super::*;

    #[tokio::test]
    async fn live_client_returns_parsed_usage() {
        let base_url = spawn_test_server(
            r#"HTTP/1.1 200 OK
content-type: application/json
content-length: 291

{"user":{"region":"REGION_CN"},"usage":{"limit":"100","used":"5","remaining":"95","resetTime":"2026-07-01T17:58:12Z"},"limits":[{"window":{"duration":300},"detail":{"limit":"100","used":"9","remaining":"91","resetTime":"2026-06-26T06:58:12Z"}}],"totalQuota":{"limit":"100","remaining":"99"}}"#,
        )
        .await;

        let client =
            KimiClient::new_with_base_url(SecretString::new("sk-test".to_string()), base_url)
                .expect("client should build");

        let result = client.fetch_usage().await.expect("should fetch");
        assert_eq!(result.quotas.five_hour.used, 9);
        assert_eq!(result.quotas.weekly.used, 5);
        assert_eq!(result.quotas.purchased.remaining, 99);
    }

    #[tokio::test]
    async fn live_client_maps_401_to_authentication_failed() {
        let base_url = spawn_test_server("HTTP/1.1 401 Unauthorized\r\n\r\n").await;
        let client =
            KimiClient::new_with_base_url(SecretString::new("sk-test".to_string()), base_url)
                .expect("client should build");

        let error = client.fetch_usage().await.expect_err("401 should fail");
        assert!(matches!(error, KimiFetchError::AuthenticationFailed));
    }

    #[test]
    fn utc_from_seconds_epoch() {
        assert_eq!(utc_from_seconds(0), (1970, 1, 1, 0, 0, 0));
    }

    #[test]
    fn utc_from_seconds_known_instant() {
        // 2026-06-26T10:00:00Z == 1782468000
        assert_eq!(utc_from_seconds(1_782_468_000), (2026, 6, 26, 10, 0, 0));
    }

    #[test]
    fn is_leap_year_rules() {
        assert!(is_leap_year(2000));
        assert!(!is_leap_year(1900));
        assert!(is_leap_year(2024));
        assert!(!is_leap_year(2026));
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
