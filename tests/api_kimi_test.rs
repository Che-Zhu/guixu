use std::sync::Arc;

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
    assert_eq!(
        json["quotas"]["five_hour"]["resets_at"],
        "2026-06-26T06:58:12Z"
    );

    assert_eq!(json["quotas"]["weekly"]["limit"], 100);
    assert_eq!(json["quotas"]["weekly"]["used"], 5);
    assert_eq!(json["quotas"]["weekly"]["remaining"], 95);
    assert_eq!(
        json["quotas"]["weekly"]["resets_at"],
        "2026-07-01T17:58:12Z"
    );

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
