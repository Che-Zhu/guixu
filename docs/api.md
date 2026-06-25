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
