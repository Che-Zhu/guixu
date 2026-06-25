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
