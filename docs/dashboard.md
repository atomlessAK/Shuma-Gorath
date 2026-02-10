# 🐙 Dashboard Documentation

## 🐙 Overview

The dashboard provides real-time monitoring, analytics, and admin controls for Shuma-Gorath.

## 🐙 Features

Stats:
- Total bans
- Active bans
- Total events
- Unique IPs
- Test mode banner
- Fail-open/closed indicator (read-only)

Charts:
- Event types distribution (doughnut)
- Top IPs by activity (bar)
- Events over time (line, 60m/24h/7d/30d)

Tables:
- Current bans (IP, reason, timestamps, signal badges, expandable fingerprint details, quick unban)
- Recent events (type, IP, timestamp, reason)
- CDP detections & bans table (time-windowed CDP-only feed)
- CDP cumulative totals (all-time detections + auto-bans)

Controls:
- Test mode toggle
- Manual ban/unban
- JS Required enforcement toggle
- Rate limit (requests/minute) config
- robots.txt configuration
- CDP detection controls
- PoW status panel and difficulty/TTL tuning (when `SHUMA_POW_CONFIG_MUTABLE=1`)
- Botness scoring controls:
- challenge threshold
- maze threshold
- per-signal weights (`js_required`, `geo_risk`, `rate_medium`, `rate_high`)
- read-only terminal signal catalog
- editable only when `SHUMA_BOTNESS_CONFIG_MUTABLE=1` (or `SHUMA_CHALLENGE_CONFIG_MUTABLE=1` fallback)
- GEO policy controls:
- risk scoring countries (`geo_risk`)
- tiered routing countries (`geo_allow`, `geo_challenge`, `geo_maze`, `geo_block`)
- Link maze stats
- Enter key submits inputs (API key, ban, unban)
- Auto-refresh every 30 seconds

## 🐙 Access

Development:
- `http://127.0.0.1:3000/dashboard/index.html`
- API key source: `SHUMA_API_KEY` from environment (local dev commonly loads this from `.env.local`)
- Login flow: unauthenticated visits to `/dashboard/index.html` are redirected to `/dashboard/login.html`; enter API key once to create a short-lived same-origin admin session cookie
- Admin API endpoint is inferred from the page origin (same-origin only)

Production (recommended):
- Protect the dashboard with auth
- Use `SHUMA_ADMIN_IP_ALLOWLIST` to restrict admin access
- Serve over HTTPS
- Store secrets in your platform’s secret store

## 🐙 Event Retention

Event log retention is controlled by `SHUMA_EVENT_LOG_RETENTION_HOURS` (default: `168`). Set to `0` to disable cleanup.

## 🐙 API Endpoints Used

- `GET  /admin/analytics`
- `GET  /admin/events?hours=24`
- `GET  /admin/cdp/events?hours=24&limit=500`
- `GET  /admin/config`
- `POST /admin/config`
- `POST /admin/ban`
- `POST /admin/unban`
- `GET  /admin/maze`
- `GET  /admin/robots`
- `GET  /admin/cdp`

## 🐙 Files

```
dashboard/
  login.html
  login.js
  index.html
  dashboard.js
  admin.js
  style.css
```

## 🐙 Data Flow (High Level)

1. Page loads and initializes charts
2. Config and analytics are fetched
3. Auto-refresh updates stats, charts, and tables

Note: `SHUMA_KV_STORE_FAIL_OPEN` is an environment-level policy and is shown read-only in the dashboard.
Note: PoW enable/disable is environment-level; difficulty/TTL are editable only if `SHUMA_POW_CONFIG_MUTABLE=1`.
Note: PoW config changes are logged to the event log as admin actions.
Note: Botness scoring changes are logged as `botness_config_update` admin actions.
