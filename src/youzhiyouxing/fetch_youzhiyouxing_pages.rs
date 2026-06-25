use std::{sync::Arc, time::Duration};

use reqwest::{header, Client, StatusCode};

use crate::config::SecretString;

use super::{parse_youzhiyouxing_pages::YouzhiyouxingHtmlPages, types::YouzhiyouxingFetchError};

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
    base_url: String,
}

impl YouzhiyouxingClient {
    pub fn new(cookie: SecretString) -> Result<Self, reqwest::Error> {
        let http = Client::builder()
            .timeout(Duration::from_secs(20))
            .user_agent("guixu/0.1")
            .redirect(reqwest::redirect::Policy::none())
            .build()?;

        Ok(Self {
            http,
            cookie,
            base_url: BASE_URL.to_string(),
        })
    }

    #[cfg(test)]
    fn new_with_base_url(cookie: SecretString, base_url: String) -> Result<Self, reqwest::Error> {
        let http = Client::builder()
            .timeout(Duration::from_secs(20))
            .user_agent("guixu/0.1")
            .redirect(reqwest::redirect::Policy::none())
            .build()?;

        Ok(Self {
            http,
            cookie,
            base_url,
        })
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
            .get(format!("{}{}", self.base_url, path))
            .header(header::COOKIE, self.cookie.as_str())
            .header(header::ACCEPT, "text/html,application/xhtml+xml")
            .send()
            .await?;

        if response.status() == StatusCode::FOUND || response.status() == StatusCode::SEE_OTHER {
            return Err(YouzhiyouxingFetchError::SessionExpired);
        }

        if !response.status().is_success() {
            return Err(YouzhiyouxingFetchError::UnexpectedStatus {
                path: path.to_string(),
                status: response.status(),
            });
        }

        let html = response.text().await?;
        if html.contains("做聪明的投资者") && html.contains("登录") {
            return Err(YouzhiyouxingFetchError::SessionExpired);
        }

        Ok(html)
    }
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;

    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
    };

    use super::*;

    #[tokio::test]
    async fn static_source_returns_pages_without_network() {
        let pages = YouzhiyouxingHtmlPages {
            dashboard: "dashboard".to_string(),
            balance: "balance".to_string(),
            abooks: "abooks".to_string(),
            cashflow: "cashflow".to_string(),
        };
        let source = YouzhiyouxingPageSource::Static(Arc::new(pages.clone()));

        let fetched = source
            .fetch_pages()
            .await
            .expect("static pages should clone");

        assert_eq!(fetched.dashboard, pages.dashboard);
        assert_eq!(fetched.balance, pages.balance);
        assert_eq!(fetched.abooks, pages.abooks);
        assert_eq!(fetched.cashflow, pages.cashflow);
    }

    #[tokio::test]
    async fn live_source_maps_redirect_to_session_expired() {
        let base_url = spawn_test_server("HTTP/1.1 302 Found\r\nLocation: /\r\n\r\n").await;
        let client = YouzhiyouxingClient::new_with_base_url(
            SecretString::new("_weasley_key=test".to_string()),
            base_url,
        )
        .expect("client should build");

        let error = client
            .fetch_html("/dashboard")
            .await
            .expect_err("redirect should fail");

        assert!(matches!(error, YouzhiyouxingFetchError::SessionExpired));
    }

    #[tokio::test]
    async fn live_source_reports_unexpected_status_with_path() {
        let base_url = spawn_test_server("HTTP/1.1 500 Internal Server Error\r\n\r\n").await;
        let client = YouzhiyouxingClient::new_with_base_url(
            SecretString::new("_weasley_key=test".to_string()),
            base_url,
        )
        .expect("client should build");

        let error = client
            .fetch_html("/balance")
            .await
            .expect_err("500 should fail");

        match error {
            YouzhiyouxingFetchError::UnexpectedStatus { path, status } => {
                assert_eq!(path, "/balance");
                assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
            }
            other => panic!("expected unexpected status, got {other:?}"),
        }
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
