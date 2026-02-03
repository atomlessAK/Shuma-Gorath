# Quick Reference - WASM Bot Trap

## Common Commands

### Build & Run
```bash
make dev            # Build and run with file watching (auto-rebuild)
make run            # Build once and run (no watching)
make build          # Build release binary only
make prod           # Build for production and start server
make stop           # Stop running Spin server
make status         # Check if server is running
make clean          # Clean build artifacts
```

### Testing
```bash
# All tests (recommended)
make test                  # Run unit tests + integration if server running

# Unit tests only (34 tests, native Rust, NO Spin required)
make test-unit             # Run all 34 unit tests

# Integration tests only (15 scenarios, Spin environment required)
make dev                   # In terminal 1
make test-integration      # In terminal 2
```
**Important:** Unit tests run in native Rust. Integration tests MUST run in Spin environment.

### Manual Build
```bash
cargo build --target wasm32-wasip1 --release
spin up
```

## API Endpoints

### Public Endpoints
- `GET /` - Main bot trap (may show block page, JS challenge, or pass through)
- `GET /health` - Health check (localhost only)
- `GET /bot-trap` - Honeypot (triggers ban)
- `POST /quiz` - Submit quiz answer (if quiz re-enabled)

### Admin API (requires `Authorization: Bearer <API_KEY>`)
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

## Configuration

### API Key
Set in `spin.toml` or environment:
```toml
[component.bot-trap]
environment = { API_KEY = "your-secret-key-here" }
```

### Test Mode
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

### Default Config
Located in `src/config.rs`:
- **Ban duration**: 21600 seconds (6 hours)
- **Rate limit**: 80 requests/minute
- **Honeypots**: `/bot-trap`
- **Browser blocks**: Chrome <120, Firefox <115, Safari <15

## Dashboard

1. Open `dashboard/index.html` in browser
2. Enter API endpoint: `http://127.0.0.1:3000`
3. Enter API key (default: `changeme-supersecret`)
4. View analytics and manage bans

## Common Tasks

### Ban an IP manually
```bash
curl -X POST -H "Authorization: Bearer changeme-supersecret" \
  -H "Content-Type: application/json" \
  -d '{"ip":"1.2.3.4","reason":"spam","duration":3600}' \
  http://127.0.0.1:3000/admin/ban
```

### Unban an IP
```bash
curl -X POST -H "Authorization: Bearer changeme-supersecret" \
  "http://127.0.0.1:3000/admin/unban?ip=1.2.3.4"
```

### View recent events
```bash
curl -H "Authorization: Bearer changeme-supersecret" \
  "http://127.0.0.1:3000/admin/events?hours=24" | jq
```

### Test honeypot
```bash
curl -H "X-Forwarded-For: 1.2.3.4" http://127.0.0.1:3000/bot-trap
# Subsequent requests from 1.2.3.4 will be blocked
```

## Troubleshooting

### Build Errors
- Run `cargo clean` before switching between WASM and native builds
- Ensure `wasm32-wasip1` target installed: `rustup target add wasm32-wasip1`

### Port Already in Use
- `make local` automatically kills existing Spin instances
- Manual: `pkill -f spin && spin up`

### Tests Failing
- Use provided test scripts (they handle clean builds)
- Integration tests require Spin to be running

### Dashboard Not Loading
- Open `dashboard/index.html` as a local file (file://)
- Or configure static file serving in Spin (see `spin_static_dashboard.toml`)

## Project Structure
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
tests/              # Integration tests
```

## Security Notes

- **Never commit API keys** - Use environment variables
- **Rotate keys regularly** - Change API_KEY in production
- **Use HTTPS in production** - TLS required for API key security
- **Restrict admin access** - Use IP allowlist or VPN
- **Monitor event logs** - Review admin actions regularly

## Next Steps

1. **Production Deployment**: Deploy to Fermyon Cloud or compatible platform
2. **Custom Config**: Update config in KV store for your needs
3. **Monitor**: Use dashboard to track bans and events
4. **Tune**: Use test mode to validate before enforcing blocks
5. **Extend**: See roadmap in README for agentic AI features
