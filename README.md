![Shuma-Gorath](dashboard/assets/shuma-gorath.png)

# 🐙 Shuma-Gorath

Many-Angled Bot Defence for Spin. Shuma-Gorath is a WebAssembly bot trap that detects, blocks, and monitors automated abuse with honeypots, rate limiting, JS challenges, CDP signals, a link maze, and a real-time admin dashboard.

Shuma-Gorath is designed to **complement enterprise bot defenses** (such as Akamai Bot Manager) as a second-layer of application-specific protection, but it can also run standalone.

## 🐙 Quickstart (Official)

```bash
# One-time setup (installs Rust, Spin, cargo-watch)
make setup

# If commands are missing, open a new terminal or run:
# source ~/.zshrc

# Start the dev server (official path)
make dev
```

Dashboard:
- `http://127.0.0.1:3000/dashboard/index.html`

Notes:
- Run setup in an interactive terminal (it may prompt for sudo to install Spin).
- Use the Makefile for all build/run/test flows.

## 🐙 Common Commands

```bash
make dev              # Start dev server with file watching
make test             # Unit tests + integration if server running
make test-unit        # Unit tests only
make test-integration # Integration tests (requires running server)
make stop             # Stop Spin server
make status           # Check server status
make help             # Show all commands
```

## 🐙 Configuration (Short List)

Key environment variables:
- `API_KEY` - Admin API bearer token
- `JS_SECRET` - Signs the `js_verified` cookie
- `FORWARDED_IP_SECRET` - Trusts `X-Forwarded-For` only when `X-Shuma-Forwarded-Secret` matches
- `ADMIN_IP_ALLOWLIST` - CIDR/IP allowlist for admin access
- `EVENT_LOG_RETENTION_HOURS` - Event log retention window
- `SHUMA_FAIL_MODE` - `open` or `closed`
- `POW_ENABLED` - Enable proof-of-work before JS verification
- `POW_DIFFICULTY` - Leading zero bits required (default: 15)
- `POW_TTL_SECONDS` - PoW seed expiry in seconds (default: 90)
- `POW_SECRET` - Optional PoW signing secret (defaults to `JS_SECRET`)
- `POW_CONFIG_MUTABLE` - Allow admin API to tune PoW difficulty/TTL
- `CHALLENGE_RISK_THRESHOLD` - Risk score to trigger step-up challenge (default: 3)
- `CHALLENGE_CONFIG_MUTABLE` - Allow admin API to tune challenge threshold
- `CHALLENGE_TRANSFORM_COUNT` - Number of transforms offered in challenge UI (4-8, default: 6)

Deployment policy note: `SHUMA_FAIL_MODE` is a critical choice (fail-open vs fail-closed) when the KV store is unavailable. See `docs/security-hardening.md` and `docs/deployment.md`.

See `docs/deployment.md` for deployment wiring and secret handling.

## 🐙 Documentation

- [`docs/index.md`](docs/index.md) - Docs index
- [`QUICK_REFERENCE.md`](QUICK_REFERENCE.md) - Command and API cheat sheet
- [`docs/testing.md`](docs/testing.md) - Testing guide (Makefile-only)
- [`docs/dashboard.md`](docs/dashboard.md) - Dashboard and admin UI
- [`docs/deployment.md`](docs/deployment.md) - Production/deploy configuration
- [`docs/api.md`](docs/api.md) - API usage and endpoint details
- [`docs/configuration.md`](docs/configuration.md) - Runtime configuration reference
- [`docs/security-hardening.md`](docs/security-hardening.md) - Deployment security checklist
- [`docs/observability.md`](docs/observability.md) - Prometheus/Grafana integration
- [`docs/akamai-bot-manager.md`](docs/akamai-bot-manager.md) - Positioning and layered defense
- [`docs/features.md`](docs/features.md) - Feature list and roadmap
- [`docs/challenge-verification.md`](docs/challenge-verification.md) - Human verification strategy
- [`docs/maze.md`](docs/maze.md) - Link maze honeypot
- [`SECURITY_REVIEW.md`](SECURITY_REVIEW.md) - Security audit notes

## 🐙 Repository Structure (High Level)

```
src/        # Core bot trap logic (Spin component)
dashboard/  # Admin dashboard UI
scripts/    # Build helpers (Makefile used by default)
```
