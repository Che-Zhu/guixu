# 有知有行 Dashboard 数据读取总结

## 登录方法

目标站点是 `https://yx.youzhiyouxing.cn`，当前 dashboard 数据主要通过服务端渲染 HTML 返回，不是前端 REST API 拉取。

自动化读取时不需要打开浏览器页面，核心是带登录态请求页面：

```http
GET https://yx.youzhiyouxing.cn/dashboard
Cookie: <你的登录 Cookie>
```

建议实现方式：

1. 首次手动登录网站。
2. 从浏览器开发者工具里复制该站点请求的 `Cookie` 请求头，写入本机环境变量或本地密钥存储。
3. 抓取脚本使用 cookie jar 发请求，并自动保存响应里的 `Set-Cookie`。
4. 每次请求后检查 HTML 是否仍包含登录后特征，例如 `退出`、`家庭总资产`。
5. 如果响应变成登录页，说明登录态失效，需要重新登录并更新初始 Cookie。

Cookie 是否能自动续期，取决于服务端：

- 如果页面响应返回新的 `Set-Cookie`，脚本保存后即可滚动续期。
- 如果没有新的 `Set-Cookie`，只能使用当前 Cookie 到过期为止。
- 如果 Cookie 过期后没有 refresh/remember-me 机制，就不能无感刷新，只能重新登录。

不要把 Cookie 写死进代码仓库。建议使用 `.env`、系统 keychain 或只存在本机的配置文件。

## 前端运行方式

页面使用 Phoenix LiveView。

已观察到的前端资源：

```text
/assets/app-2b22b5c78a141423c3248fb5bc61ed1a.js?vsn=d
/images/sensorsdata.min-be53415294e03192e38aa0656b282440.js?vsn=d
/assets/app-c34dd85814279efb0c4113c7bb9bacc6.css?vsn=d
```

主脚本会连接：

```text
/live
```

它带 `meta[name="csrf-token"]` 中的 token 连接 LiveView。对于“只读数据抓取”，不建议实现 `/live` websocket 协议；直接 GET 服务端渲染页面并解析 HTML 更短。

## 可用接口和可获取数据

这里的“接口”是可直接请求的页面路径，不是 JSON API。返回内容是 HTML，需要解析 DOM 或页面文本。

### `GET /dashboard`

用途：总览页首屏。

可获取：

- 用户显示名
- 日期
- 我的记账汇总列表
- 家庭总资产卡片
- 投资记账卡片
- 年度现金流入口状态
- 财务晴雨表指标
- 家庭保单入口状态
- 心理账户列表

脱敏后的 dashboard 数据示例：

```text
家庭总资产: 123,456.78
资产减少: 1,234.56
投资记账默认汇总收益: -789.01
今年净投入: 12,345.67
资产负债率: 12.34%
财务自由度: 0.00%
躺平度: 0.00%
```

心理账户按钮通过 LiveView 事件标记：

```text
phx-click="pick_goal"
phx-value-mid="<goal_id>" -> 车贷资金
phx-value-mid="<goal_id>" -> 结婚储蓄
phx-value-mid="<goal_id>" -> 生活备用金
phx-click="add_goal"   -> 添加心理账户
```

### `GET /balance`

用途：家庭资产记账。

可获取：

- 净资产
- 资产总额
- 负债总额
- 总资产变化
- 流动资产变化
- 投资理财变化
- 固定资产变化
- 应收款变化
- 资产构成
- 四笔钱配置
- 保险金额和保险数量

脱敏后的数据示例：

```text
净资产: 100,000.00
资产总额: 123,456.78
负债总额: 23,456.78
总资产变化: -1,234.56
保险: 0 元 / 0 项保险
```

资产负债率可由这里计算：

```text
资产负债率 = 负债总额 / 资产总额
           = 23,456.78 / 123,456.78
           = 19.00%
```

页面内 LiveView 事件：

```text
pick_family_member
pick_money_detail
toggle_active_tab
toggle_expand_category
```

### `GET /balance/inventory?category=insurance`

用途：保险库存/家庭保单。

可获取：

- 保险项目列表
- 当前分类是否有记录
- 添加保险项目入口

已看到的数据示例：

```text
保险项目: 无记录
说明: 通过配置医疗险、意外险、重疾险、定期寿险等保障类险种，应对可能遇到的风险。
```

页面内 LiveView 事件：

```text
pick_category
pick_variety
```

### `GET /abooks`

用途：投资记账默认汇总页。

等价于默认汇总：

```text
/abooks/summary/<summary_id>
```

可获取：

- 汇总列表
- 总资产
- 累计收益
- 资金加权收益率
- 年化收益率
- 收益率曲线数据的页面文本
- 资产账户列表
- 年度收益对比
- 资产构成
- 投入、转出、净投入、资产盈亏、期末金额

脱敏后的数据示例：

```text
总资产: 80,000.00
累计收益: -789.01
资金加权收益率: -1.63%
净投入: 80,789.01
```

页面内 LiveView 事件：

```text
summary_selected
pick_chart_type
roi_benchmark_selected
roi_contrast_benchmark_selected
assets_contrast_period_selected
```

### `GET /abooks/summary/<default_summary_id>`

用途：默认汇总。

可获取：

```text
总资产: 80,000.00
累计收益: -789.01
资金加权收益率: -1.63%
净投入: 80,789.01
```

### `GET /abooks/summary/<investment_summary_id>`

用途：投资理财汇总。

可获取：

```text
总资产: 70,000.00
累计收益: -800.00
资金加权收益率: -1.86%
```

### `GET /abooks/summary/<cash_summary_id>`

用途：流动资金汇总。

可获取：

```text
总资产: 10,000.00
累计收益: 10.00
资金加权收益率: 0.07%
```

### `GET /abooks/summary/<overseas_summary_id>`

用途：海外资金汇总。

可获取：

```text
总资产: 30,000.00
累计收益: -500.00
资金加权收益率: -2.27%
```

### `GET /cashflow`

用途：年度现金流。

可获取：

- 是否已配置年度现金流
- 年度现金流入口状态
- 如果已配置，理论上可解析预估年收入、预估年支出、储蓄率等

当前观察到的是未配置/引导状态：

```text
年度现金流（2026）
进入现金流预估
```

Dashboard 上的 `预估储蓄率 **`、`保费收入比 **`、`财务自由度 0.00%`、`躺平度 0.00%` 都和 `/cashflow` 是否有预估数据相关。

## 推荐抓取流程

```text
1. GET /dashboard
   - 判断登录态
   - 解析首屏总览、心理账户、汇总 id

2. GET /balance
   - 解析资产总额、负债总额、净资产、资产构成、保险概览

3. GET /balance/inventory?category=insurance
   - 解析保单记录

4. GET /abooks/summary/:id
   - 对每个汇总 id 解析投资数据

5. GET /cashflow
   - 判断现金流是否已配置
   - 若已配置，再解析现金流指标
```

## 输出建议

建议脚本输出 JSON，结构可以这样：

```json
{
  "dashboard": {
    "date": "2026/06/04",
    "family_total_assets": 123456.78,
    "asset_change": -1234.56,
    "debt_ratio": 19.00
  },
  "balance": {
    "net_assets": 100000.00,
    "total_assets": 123456.78,
    "total_liabilities": 23456.78
  },
  "abooks": [
    {
      "id": "<summary_id>",
      "name": "默认汇总",
      "total_assets": 80000.00,
      "acc_profit": -789.01
    }
  ],
  "cashflow": {
    "configured": false
  },
  "insurance": {
    "count": 0,
    "amount": 0
  }
}
```

## 注意点

- 这些路径返回 HTML，不是 JSON。
- 数据字段没有稳定的 API schema，需要用 DOM 结构或文本锚点解析。
- 页面是 LiveView，交互会走 `/live`，但只读抓取不需要 websocket。
- Cookie 可能过期，脚本必须检测登录态失效。
- 如果后续页面结构改版，解析规则需要调整。

## Implementation status

The first Guixu implementation uses Rust, Reqwest, and server-side HTML parsing. Local validation on 2026-06-25 confirmed that a `YOUZHIYOUXING_COOKIE` value containing `_weasley_key=...` can fetch `/dashboard`, `/balance`, `/abooks`, and `/cashflow` as authenticated HTML without using the LiveView websocket.
