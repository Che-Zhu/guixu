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
