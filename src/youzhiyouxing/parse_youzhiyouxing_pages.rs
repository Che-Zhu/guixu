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
