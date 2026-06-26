# 有知有行 Dashboard 数据读取总结

目标站点是 `https://yx.youzhiyouxing.cn`，dashboard 数据通过服务端渲染 HTML 返回，不是前端 REST API。

## 登录方式

自动化读取不需要打开浏览器，直接带登录态请求页面即可：

```http
GET https://yx.youzhiyouxing.cn/dashboard
Cookie: <你的登录 Cookie>
```

当前实现把 Cookie 放在环境变量 `YOUZHIYOUXING_COOKIE` 里，服务启动时读取。Cookie 过期后需要手动更新。

## 抓取页面

实现会请求以下 4 个页面：

| 路径 | 用途 |
|---|---|
| `/dashboard` | 总览页首屏 |
| `/balance` | 家庭资产记账 |
| `/abooks` | 投资记账默认汇总入口 |
| `/cashflow` | 年度现金流 |

返回都是 HTML，用 scraper 抽取文本后按锚点解析。

## 解析字段

### Dashboard

从 `/dashboard` 页面解析：

- `family_total_assets`：家庭总资产
- `asset_change`：资产变动（页面显示为"资产减少"或"资产增加"）
- `debt_ratio`：资产负债率
- `cashflow_configured`：是否已配置年度现金流

### Balance

从 `/balance` 页面解析：

- `net_assets`：净资产
- `total_assets`：资产总额
- `total_liabilities`：负债总额

### Investment

从 `/abooks` 页面解析默认汇总：

- `total_assets`：总资产
- `accumulated_profit`：累计收益
- `money_weighted_return`：资金加权收益率

### Cashflow

从 `/cashflow` 页面判断：

- `configured`：年度现金流是否已配置

## 输出结构

Guixu 的 `/youzhiyouxing` 端点返回如下 JSON：

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

## 会话失效

如果响应是 302/303 跳转，或者 HTML 里出现"做聪明的投资者"等登录页特征，接口返回 `502 Bad Gateway`，错误码为 `upstream_session_expired`。

## 注意点

- 这些路径返回 HTML，不是 JSON。
- 字段没有稳定 schema，靠文本锚点解析，页面结构改版时需要同步调整规则。
- Cookie 不会自动续期，过期后只能重新登录并更新 `YOUZHIYOUXING_COOKIE`。
