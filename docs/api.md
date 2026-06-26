# Guixu API

Guixu exposes backend JSON APIs for private frontends. Replace `<base-url>` with the deployed service origin, for example `https://api.example.com`.

## Health Check

```http
GET <base-url>/healthz
```

Successful response:

```http
HTTP/1.1 200 OK
content-type: text/plain; charset=utf-8
```

```text
ok
```

## Youzhiyouxing Summary

```http
GET <base-url>/youzhiyouxing
```

Returns the first normalized data slice from Youzhiyouxing authenticated HTML pages.

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
    "net_assets": 100000.0,
    "total_assets": 123456.78,
    "total_liabilities": 23456.78
  },
  "investment": {
    "total_assets": 80000.0,
    "accumulated_profit": -789.01,
    "money_weighted_return": -1.63
  },
  "cashflow": {
    "configured": false
  }
}
```

Nullable fields:

- `dashboard.asset_change`
- `investment.total_assets`
- `investment.accumulated_profit`
- `investment.money_weighted_return`

Session expired or invalid cookie response:

```http
HTTP/1.1 502 Bad Gateway
content-type: application/json
```

```json
{
  "error": "upstream_session_expired",
  "message": "Youzhiyouxing session is expired or invalid. Refresh YOUZHIYOUXING_COOKIE."
}
```

Other upstream fetch or parse failures also return `502 Bad Gateway` with an `error` and `message` field.

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
