# 🐙 Quick Reference - WASM Bot Trap

For full documentation, see `docs/index.md`.

## 🐙 Common Commands

### 🐙 Setup
```bash
make setup          # Install dependencies (Rust, Spin, cargo-watch)
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
make test                  # Run unit tests + integration if server running

# Unit tests only (native Rust, NO Spin required)
make test-unit             # Run all unit tests

# Integration tests only (16 scenarios, Spin environment required)
make dev                   # In terminal 1
make test-integration      # In terminal 2
```
**Important:** Unit tests run in native Rust. Integration tests MUST run in Spin environment.

## 🐙 API Endpoints

### 🐙 Public Endpoints
- `GET /` - Main bot trap (may show block page, JS challenge, or pass through)
- `GET /health` - Health check (localhost only)
- `GET /bot-trap` - Honeypot (triggers ban)
- `GET /metrics` - Prometheus metrics
- `GET /robots.txt` - robots.txt (configurable)
- `GET /pow` - PoW seed (when enabled)
- `POST /pow/verify` - PoW verification
- `POST /cdp-report` - CDP automation report intake
- `POST /quiz` - Submit quiz answer (if quiz re-enabled)

### 🐙 Admin API (requires `Authorization: Bearer <API_KEY>`)
- `GET /admin/ban` - List all bans
- `POST /admin/ban` - Manually ban IP (JSON: `{"ip":"x.x.x.x","reason":"...","duration":3600}`)
- `POST /admin/unban?ip=x.x.x.x` - Unban an IP
- `GET /admin/analytics` - Get ban statistics
- `GET /admin/events?hours=24` - Get recent events
- `GET /admin/config` - Get current configuration
- `POST /admin/config` - Update configuration (test_mode, ban_durations, robots, CDP, etc.)
- `GET /admin/maze` - Link maze honeypot statistics
- `GET /admin/robots` - robots.txt configuration and preview
- `GET /admin/cdp` - CDP detection configuration and stats
- `GET /admin` - API help

## 🐙 Configuration

### 🐙 API Key
Set in `spin.toml` or environment:
```toml
[component.bot-trap]
environment = { API_KEY = "your-secret-key-here", JS_SECRET = "your-js-secret-here", EVENT_LOG_RETENTION_HOURS = "168", ADMIN_IP_ALLOWLIST = "203.0.113.0/24,198.51.100.10" }
```

`JS_SECRET` is used to sign the `js_verified` cookie for the JS challenge.
`FORWARDED_IP_SECRET` is optional and is used to trust `X-Forwarded-For` from your proxy/CDN (it must also send `X-Shuma-Forwarded-Secret`). If you set it, include that header in integration tests.
`EVENT_LOG_RETENTION_HOURS` controls how long event logs are kept (set to `0` to disable cleanup).
`ADMIN_IP_ALLOWLIST` limits admin API access to specific IPs/CIDRs (comma-separated).
`SHUMA_FAIL_MODE` controls fail-open/closed behavior when the KV store is unavailable.
`POW_ENABLED` enables proof-of-work before JS verification (default: true in dev).
`POW_DIFFICULTY` sets the leading-zero bit target (default: 15).
`POW_TTL_SECONDS` controls PoW seed expiry (default: 90).
`POW_SECRET` optionally overrides the PoW signing secret (falls back to `JS_SECRET`).

### 🐙 Forwarded IP Secret (Deployment)
Local dev (Makefile): `make dev` sets a dev-only default and passes it to Spin. Override as needed:
```bash
make dev FORWARDED_IP_SECRET="your-dev-secret"
```

Fermyon / Spin Cloud (recommended):
1. Define an application variable in `spin.toml`.
2. Map it into the component environment.
3. Set the variable in your cloud environment (CLI or console) at deploy time.

Example `spin.toml` wiring (no secret committed):
```toml
[variables]
forwarded_ip_secret = { default = "" }

[component.bot-trap]
environment = { FORWARDED_IP_SECRET = "{{ forwarded_ip_secret }}" }
```

Other deploy targets:
- Set `FORWARDED_IP_SECRET` as an environment variable in your platform's secrets/config (Kubernetes, Docker, systemd, etc.).
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

**Via environment (requires restart):**
```toml
environment = { TEST_MODE = "1" }
```

### 🐙 Default Config
Located in `src/config.rs`:
- **Ban duration**: 21600 seconds (6 hours)
- **Rate limit**: 80 requests/minute
- **Honeypots**: `/bot-trap`
- **Browser blocks**: Chrome <120, Firefox <115, Safari <15

Full configuration reference: `docs/configuration.md`.

## 🐙 Dashboard

1. Open `http://127.0.0.1:3000/dashboard/index.html` in browser
2. Enter API endpoint: `http://127.0.0.1:3000`
3. Enter API key (default: `changeme-supersecret`)
4. View analytics and manage bans

## 🐙 Common Tasks

### 🐙 Ban an IP manually
```bash
curl -X POST -H "Authorization: Bearer changeme-supersecret" \
  -H "Content-Type: application/json" \
  -d '{"ip":"1.2.3.4","reason":"spam","duration":3600}' \
  http://127.0.0.1:3000/admin/ban
```

### 🐙 Unban an IP
```bash
curl -X POST -H "Authorization: Bearer changeme-supersecret" \
  "http://127.0.0.1:3000/admin/unban?ip=1.2.3.4"
```

### 🐙 View recent events
```bash
curl -H "Authorization: Bearer changeme-supersecret" \
  "http://127.0.0.1:3000/admin/events?hours=24" | jq
```

### 🐙 Test honeypot
If `FORWARDED_IP_SECRET` is set, include the matching header:
```bash
curl -H "X-Forwarded-For: 1.2.3.4" \
  -H "X-Shuma-Forwarded-Secret: $FORWARDED_IP_SECRET" \
  http://127.0.0.1:3000/bot-trap
# Subsequent requests from 1.2.3.4 will be blocked
```

## 🐙 Troubleshooting

### 🐙 Build Errors
- If switching targets and you see build issues, run `make clean`
- Ensure dependencies are installed: `make setup` then `make verify`

### 🐙 Port Already in Use
- Use `make stop` then `make dev`

### 🐙 Tests Failing
- Use Makefile targets (`make test`, `make test-unit`, `make test-integration`)
- Integration tests require Spin to be running (`make dev`)
- Check logs with `make logs`

### 🐙 Dashboard Not Loading
- Ensure Spin is running: `make status`
- Open `http://127.0.0.1:3000/dashboard/index.html`
- Confirm API key and check logs: `make logs`

## 🐙 Project Structure
```
src/
├── lib.rs          # Main handler
├── admin.rs        # Admin API
├── auth.rs         # Authentication
├── ban.rs          # Ban management
├── block_page.rs   # Block page HTML
├── browser.rs      # Browser detection
├── config.rs       # Configuration
├── geo.rs          # Geo detection
├── honeypot.rs     # Honeypot logic
├── js.rs           # JS challenge
├── quiz.rs         # Math quiz (disabled)
├── rate.rs         # Rate limiting
├── whitelist.rs    # Whitelisting
└── *_tests.rs      # Unit tests

dashboard/          # Web dashboard
test_spin_colored.sh # Integration tests (shell)
```

## 🐙 Security Notes

- **Never commit API keys** - Use environment variables
- **Rotate keys regularly** - Change API_KEY in production
- **Use HTTPS in production** - TLS required for API key security
- **Restrict admin access** - Use IP allowlist or VPN
- **Monitor event logs** - Review admin actions regularly

## 🐙 Next Steps

1. **Production Deployment**: Deploy to Fermyon Cloud or compatible platform
2. **Custom Config**: Update config in KV store for your needs
3. **Monitor**: Use dashboard to track bans and events
4. **Tune**: Use test mode to validate before enforcing blocks
5. **Extend**: See roadmap in README for agentic AI features
