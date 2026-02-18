# 🐙 Quick Reference - WASM Bot Defence

For full documentation, see `docs/index.md`.

## 🐙 Common Commands

### 🐙 Setup
```bash
make setup          # Install dependencies (Rust, Spin, cargo-watch, Node toolchain, pnpm deps, Playwright Chromium)
make verify         # Verify dependencies are installed
```

### 🐙 Build & Run
```bash
make dev            # Build and run with file watching (auto-rebuild)
make run            # Build once and run (no watching)
make build          # Build release binary only
make prod           # Build for production and start server
make stop           # Stop running Spin server
make status         # Check if server is running
make clean          # Clean build artifacts
```

### 🐙 Testing
```bash
# All tests (recommended)
make test                  # Full suite: unit + integration + dashboard e2e (waits for existing server readiness)

# Unit tests only (native Rust, NO Spin required)
make test-unit             # Run all unit tests

# Dashboard module unit tests only (NO Spin required)
make test-dashboard-unit   # Run dashboard JS/module contract tests

# Integration tests only (Spin environment required)
make dev                   # In terminal 1
make test-integration      # In terminal 2

# Dashboard e2e smoke tests only (Spin environment required)
make dev                   # In terminal 1
make test-dashboard-e2e    # In terminal 2
```
**Important:** Unit tests run in native Rust. Integration and dashboard e2e tests MUST run against a running Spin server; test targets do not start Spin.

## 🐙 API Endpoints

### 🐙 Public Endpoints
- `GET /` - Main bot defence (may show block page, JS challenge, or pass through)
- `GET /health` - Health check (localhost only)
- `GET /instaban` - Honeypot (triggers ban)
- `GET /metrics` - Prometheus metrics
- `GET /robots.txt` - robots.txt (configurable)
- `GET /pow` - PoW seed (when enabled)
- `POST /pow/verify` - PoW verification
- `POST /cdp-report` - CDP automation report intake
- `POST /fingerprint-report` - External/edge fingerprint intake (Akamai-first mapping)
- `POST /challenge/puzzle` - Submit puzzle challenge answer (if challenge is served)

### 🐙 Admin API (requires `Authorization: Bearer <SHUMA_API_KEY>`)
- `GET /admin/ban` - List all bans
- `POST /admin/ban` - Manually ban IP (JSON: `{"ip":"x.x.x.x","duration":3600}`; reason is always `manual_ban`)
- `POST /admin/unban?ip=x.x.x.x` - Unban an IP
- `GET /admin/analytics` - Get ban statistics
- `GET /admin/events?hours=24` - Get recent events
- `GET /admin/monitoring?hours=24&limit=10` - Get consolidated monitoring summaries + detail payload (`analytics`, `events`, `bans`, `maze`, `cdp`, `cdp_events`) for dashboard Monitoring refresh
- `GET /admin/config` - Get current configuration
- `POST /admin/config` - Update configuration (test_mode, ban_durations, robots serving, AI bot policy, CDP, etc.)
- `GET /admin/config/export` - Export non-secret runtime config for immutable redeploy handoff
- `GET /admin/maze` - maze statistics
- `GET /admin/maze/preview?path=<maze_entry_path>...` - non-operational maze preview surface
- `GET /admin/robots` - robots.txt configuration and preview
- `GET /admin/cdp` - CDP + fingerprint detection configuration and stats
- `GET /admin` - API help

## 🐙 Configuration

### 🐙 API Key
Set in `spin.toml` or environment:
```toml
[component.bot-defence]
environment = { SHUMA_API_KEY = "your-secret-key-here", SHUMA_JS_SECRET = "your-js-secret-here", SHUMA_EVENT_LOG_RETENTION_HOURS = "168", SHUMA_HEALTH_SECRET = "your-health-secret-here", SHUMA_ADMIN_IP_ALLOWLIST = "203.0.113.0/24,198.51.100.10" }
```

`SHUMA_JS_SECRET` is used to sign the `js_verified` cookie for the JS challenge.
`SHUMA_FORWARDED_IP_SECRET` is optional and is used to trust `X-Forwarded-For` from your proxy/CDN (it must also send `X-Shuma-Forwarded-Secret`). If you set it, include that header in integration tests.
`SHUMA_HEALTH_SECRET` is optional and, when set, `/health` also requires `X-Shuma-Health-Secret`.
`SHUMA_EVENT_LOG_RETENTION_HOURS` controls how long event logs are kept (set to `0` to disable cleanup).
`SHUMA_ADMIN_IP_ALLOWLIST` limits admin API access to specific IPs/CIDRs (comma-separated).
`SHUMA_KV_STORE_FAIL_OPEN` controls fail-open/closed behavior when the KV store is unavailable (`true`=open, `false`=closed).
`SHUMA_POW_ENABLED` enables proof-of-work before JS verification (default: true in dev).
`SHUMA_POW_DIFFICULTY` sets the leading-zero bit target (default: 15).
`SHUMA_POW_TTL_SECONDS` controls PoW seed expiry (default: 90).
`SHUMA_POW_SECRET` optionally overrides the PoW signing secret (falls back to `SHUMA_JS_SECRET`).
`SHUMA_MAZE_PREVIEW_SECRET` optionally sets a dedicated secret for `/admin/maze/preview` entropy/signing isolation.
`SHUMA_ADMIN_CONFIG_WRITE_ENABLED` controls whether admin config updates are allowed (default: false).

### 🐙 Forwarded IP Secret (Deployment)
Local dev (Makefile): `make dev` sets a dev-only default and passes it to Spin. Override as needed:
```bash
make dev SHUMA_FORWARDED_IP_SECRET="your-dev-secret"
```

Fermyon / Spin Cloud (recommended):
1. Define an application variable in `spin.toml`.
2. Map it into the component environment.
3. Set the variable in your cloud environment (CLI or console) at deploy time.

Example `spin.toml` wiring (no secret committed):
```toml
[variables]
forwarded_ip_secret = { default = "" }

[component.bot-defence]
environment = { SHUMA_FORWARDED_IP_SECRET = "{{ forwarded_ip_secret }}" }
```

Other deploy targets:
- Set `SHUMA_FORWARDED_IP_SECRET` as an environment variable in your platform's secrets/config (Kubernetes, Docker, systemd, etc.).
- Ensure your proxy/CDN sends `X-Shuma-Forwarded-Secret` with the same value on each request.

For more deployment detail, see `docs/deployment.md`.

### 🐙 Test Mode
Enable for safe production testing (logs but doesn't block):

**Via Dashboard:** Use the Test Mode toggle in Admin Controls

**Via API:**
```bash
# Enable test mode
curl -X POST -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"test_mode": true}' \
  http://127.0.0.1:3000/admin/config

# Check current status
curl -H "Authorization: Bearer YOUR_API_KEY" \
  http://127.0.0.1:3000/admin/config
```

Test mode is a KV-backed runtime tunable; use dashboard or `POST /admin/config` to change it.

### 🐙 Default Config
Defaults are defined in `config/defaults.env` and seeded into KV:
- **Ban duration**: 21600 seconds (6 hours)
- **Rate limit**: 80 requests/minute
- **Honeypots**: `/instaban`
- **Browser blocks**: Chrome <120, Firefox <115, Safari <15

Full configuration reference: `docs/configuration.md`.

## 🐙 Dashboard

1. Open `http://127.0.0.1:3000/dashboard/index.html` in browser
2. Enter API endpoint: `http://127.0.0.1:3000`
3. Enter API key from `make api-key-show` (local dev) or deployed `SHUMA_API_KEY`
4. View analytics and manage bans

## 🐙 Common Tasks

### 🐙 Ban an IP manually
```bash
curl -X POST -H "Authorization: Bearer $SHUMA_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"ip":"1.2.3.4","reason":"spam","duration":3600}' \
  http://127.0.0.1:3000/admin/ban
```

### 🐙 Unban an IP
```bash
curl -X POST -H "Authorization: Bearer $SHUMA_API_KEY" \
  "http://127.0.0.1:3000/admin/unban?ip=1.2.3.4"
```

### 🐙 View recent events
```bash
curl -H "Authorization: Bearer $SHUMA_API_KEY" \
  "http://127.0.0.1:3000/admin/events?hours=24" | jq
```

### 🐙 Export runtime config for immutable redeploy handoff
```bash
curl -H "Authorization: Bearer $SHUMA_API_KEY" \
  "http://127.0.0.1:3000/admin/config/export" | jq -r '.env_text'
```

### 🐙 Test honeypot
If `SHUMA_FORWARDED_IP_SECRET` is set, include the matching header:
```bash
curl -H "X-Forwarded-For: 1.2.3.4" \
  -H "X-Shuma-Forwarded-Secret: $SHUMA_FORWARDED_IP_SECRET" \
  http://127.0.0.1:3000/instaban
# Subsequent requests from 1.2.3.4 will be blocked
```

## 🐙 Troubleshooting

### 🐙 Build Errors
- If switching targets and you see build issues, run `make clean`
- Ensure dependencies are installed: `make setup` then `make verify`

### 🐙 Port Already in Use
- Use `make stop` then `make dev`

### 🐙 Tests Failing
- Use Makefile targets (`make test`, `make test-unit`, `make test-dashboard-unit`, `make test-integration`, `make test-dashboard-e2e`)
- `make test` waits for existing Spin readiness (`/health`) before running integration/dashboard suites
- If startup is slow, increase wait timeout: `make test SPIN_READY_TIMEOUT_SECONDS=180`
- Check logs with `make logs`

### 🐙 Dashboard Not Loading
- Ensure Spin is running: `make status`
- Open `http://127.0.0.1:3000/dashboard/index.html`
- Confirm API key and check logs: `make logs`

## 🐙 Project Structure
```
src/
├── lib.rs                 # Main orchestration entrypoint
├── admin/                 # Admin API + auth/session flow
├── challenge/             # Puzzle + challenge flows
├── config/                # Runtime config loading/defaults
├── enforcement/           # Ban/block/rate/honeypot actions
├── maze/                  # Maze barrier logic
├── observability/         # Metrics/export
├── providers/             # Provider contracts + registry + internal adapters
├── runtime/               # Request router/policy pipeline/test-mode helpers
└── signals/               # Browser/CDP/GEO/IP/JS/whitelist signals

dashboard/                 # Web dashboard UI
scripts/tests/integration.sh # Spin integration scenarios
```

## 🐙 Security Notes

- **Never commit API keys** - Use environment variables
- **Rotate keys regularly** - Change SHUMA_API_KEY in production
- **Use HTTPS in production** - TLS required for API key security
- **Restrict admin access** - Use IP allowlist or VPN
- **Monitor event logs** - Review admin actions regularly

## 🐙 Next Steps

1. **Production Deployment**: Deploy to Fermyon Cloud or compatible platform
2. **Custom Config**: Update config in KV store for your needs
3. **Monitor**: Use dashboard to track bans and events
4. **Tune**: Use test mode to validate before enforcing blocks
5. **Extend**: See roadmap in README for agentic AI features
