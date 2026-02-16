# 🐙 Dashboard Documentation

## 🐙 Overview

The dashboard provides real-time monitoring, analytics, and admin controls for Shuma-Gorath.

## 🐙 Tabbed SPA Layout

The dashboard is now organized as a tabbed SPA with URL-backed hash routes:

- `#monitoring`
- `#ip-bans`
- `#status`
- `#config`
- `#tuning`

Behavior:
- Selected tab is reflected in the URL hash.
- Reload preserves the selected tab.
- Keyboard navigation is supported on tabs (`Left`, `Right`, `Home`, `End`).
- Tab panels expose explicit loading/empty/error states.

Refresh model:
- Polling is scoped to the active tab only.
- One bounded timer is used (no per-tab timer accumulation).
- Polling pauses while the page is hidden and resumes on visibility restore.
- Default cadence: `Monitoring=30s`, `IP Bans=45s`, `Status/Config/Tuning=60s`.

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
- PoW status panel and difficulty/TTL tuning (when `SHUMA_POW_CONFIG_MUTABLE=true`)
- Botness scoring controls:
- challenge threshold
- maze threshold
- per-signal weights (`js_required`, `geo_risk`, `rate_medium`, `rate_high`)
- read-only terminal signal catalog
- editable only when `SHUMA_BOTNESS_CONFIG_MUTABLE=true`
- GEO policy controls:
- risk scoring countries (`geo_risk`)
- tiered routing countries (`geo_allow`, `geo_challenge`, `geo_maze`, `geo_block`)
- maze stats
- non-operational Maze Preview link in Maze config
- Enter key submits inputs (API key, ban, unban)
- Active-tab scoped auto-refresh (no background full-dashboard refresh on hidden tabs)

## 🐙 Access

Development:
- `http://127.0.0.1:3000/dashboard/index.html`
- `http://127.0.0.1:3000/dashboard` (canonical redirect to `/dashboard/index.html`)
- API key source: `SHUMA_API_KEY` from environment (local dev commonly loads this from `.env.local`)
- Login flow: unauthenticated visits to `/dashboard/index.html` are redirected to `/dashboard/login.html`; enter API key once to create a short-lived same-origin admin session cookie
- Admin API endpoint is inferred from the page origin (same-origin only)
- Runtime mutability flags are read from `.env.local` (for example `SHUMA_POW_CONFIG_MUTABLE`, `SHUMA_BOTNESS_CONFIG_MUTABLE`); if false, corresponding controls render read-only even in local dev.

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
- `GET  /admin/maze/preview`
- `GET  /admin/robots`
- `GET  /admin/cdp`

## 🐙 Files

```
dashboard/
  login.html
  login.js
  index.html
  dashboard.js
  assets/vendor/chart-lite-1.0.0.min.js
  modules/api-client.js
  modules/dashboard-state.js
  modules/tab-lifecycle.js
  style.css
```

## 🐙 Data Flow (High Level)

1. Page loads and initializes local chart runtime + dashboard modules.
2. `modules/tab-lifecycle.js` resolves hash routing and tab lifecycle (`init`, `mount`, `unmount`, `refresh`).
3. `modules/api-client.js` handles typed request/response adaptation and centralized API errors.
4. `modules/dashboard-state.js` tracks shared snapshots, invalidation, and tab-local state.
5. Active tab refresh pipeline fetches only required data for that tab.

## 🐙 Local Asset Provenance

Chart runtime is vendored locally to avoid runtime CDN dependency and supply-chain variability:

- Asset: `dashboard/assets/vendor/chart-lite-1.0.0.min.js`
- Version: `chart-lite-1.0.0`
- SHA-256: `5eec3d4b98e9ddc1fb88c44e0953b8bded137779a4d930c6ab2647a431308388`
- Policy: update only via reviewed commit; recompute SHA-256 and update this section when changed.

## 🐙 Rollback Notes

If a regression appears in the tabbed SPA shell, use this rollback sequence:

1. Revert `dashboard/index.html` tabbed-shell/module include changes.
2. Revert `dashboard/dashboard.js` tab-scoped refresh scheduler changes.
3. Revert `dashboard/modules/api-client.js` and `dashboard/modules/dashboard-state.js` integration points.
4. Run `make test` and confirm dashboard e2e returns green before deploy.
5. Keep session/csrf hardening (`modules/admin-session.js`) unchanged unless rollback requires it.

Note: `SHUMA_KV_STORE_FAIL_OPEN` is an environment-level policy and is shown read-only in the dashboard.
Note: PoW enable/disable is environment-level; difficulty/TTL are editable only if `SHUMA_POW_CONFIG_MUTABLE=true`.
Note: PoW config changes are logged to the event log as admin actions.
Note: Botness scoring changes are logged as `botness_config_update` admin actions.
