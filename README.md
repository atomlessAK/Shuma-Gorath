# WASM Stealth Bot Trap (Fermyon Spin)

This project implements a customizable, behavior-based bot defense system for deployment at the edge using Fermyon Spin and WebAssembly.

## Structure
- `src/`: Rust source code for the Spin app
- `spin.toml`: Spin app manifest
- `README.md`: Project overview and setup
- `.gitignore`: Standard ignores

## Quick Start
1. Install [Spin](https://developer.fermyon.com/spin/install)
2. Build: `spin build --release`
3. Run locally: `spin up`
4. Deploy to Fermyon Cloud as needed

---


## Usage

### As a Site Owner
- Deploy the app to your edge environment (Fermyon Cloud or compatible platform).
- Configure honeypot URLs, rate limits, browser blocklist, geo risk countries, and whitelist via the admin API.
- Monitor and manage bans and analytics via the admin API.

### Endpoints

- `/health` — Health check endpoint. Returns `OK` only when accessed from localhost (127.0.0.1 or ::1). Used for liveness/readiness probes. All other sources receive 403 Forbidden.
- `/` — Main endpoint. Applies bot trap logic: whitelist, ban, honeypot, rate limit, JS challenge, browser/geo checks.
- `/admin/*` — Admin API endpoints (see below).

### Admin API Endpoints
All endpoints require an `Authorization: Bearer <API_KEY>` header. The API key is configurable via the `API_KEY` environment variable (see below).

- `GET /admin/ban` — List all current bans (JSON: IP, reason, expiry)
- `POST /admin/unban?ip=...` — Unban a specific IP (removes ban immediately)
- `GET /admin/analytics` — Get ban count analytics
- `GET /admin` — Usage help

#### API Key Configuration
- The admin API key is set via the `API_KEY` environment variable in your Spin manifest or deployment environment. If not set, it defaults to `changeme-supersecret` for development.
- Example (in `spin.toml`):
	```toml
	[component.bot-trap]
	environment = { API_KEY = "changeme-supersecret" }
	```

### Configuration
- Ban duration, rate limit, honeypot URLs, browser blocklist, geo risk, and whitelist are stored in edge KV and can be managed via future admin endpoints or direct KV updates.

---



## Testing

### Integration Tests (Automated)

Run the full integration test suite with:

```sh
spin build && ./test_spin_colored.sh
```

This script will:
- Test `/health` endpoint (OK from localhost only)
- Test root endpoint (`/`) for JS challenge and normal OK
- Test honeypot ban and verify ban logic
- Test admin unban (`/admin/unban?ip=...`)
- Test `/health` after ban/unban

All results are color-coded for easy review. See `test_spin_colored.sh` for details.

### Manual Testing: Triggering Bot Trap Responses

To manually trigger and test each bot trap response in your browser or with curl, you can simulate the following scenarios:

1. **Whitelist**: Add your IP to the whitelist in the config (or remove it to test blocks).
2. **Ban**: Manually ban your IP using the admin API, or trigger a honeypot or rate limit.
3. **Honeypot**: Visit a honeypot path (e.g., http://127.0.0.1:3000/bot-trap).
4. **Rate Limit**: Send many requests quickly (e.g., with a script or curl loop) to exceed the rate limit.
5. **JS Challenge**: Clear cookies and visit the root endpoint; you should see the JS challenge page.
6. **Outdated Browser**: Use a custom User-Agent string with an old version (e.g., Chrome/100) to trigger a block.
7. **Geo Risk**: Add a high-risk country to the config and set the X-Geo-Country header.

You can use browser dev tools or curl to set headers and test these scenarios. See the admin API section above for ban management.

### Unit Tests

Unit tests for ban logic are in `src/ban_tests.rs`. Run with:

```sh
cargo test --lib
```

---

- Modular Rust code: see `src/` for ban, rate, JS, browser, geo, whitelist, honeypot, and admin logic.
- Integration test script: see `test_spin_colored.sh` for automated end-to-end tests.
- Unit tests: see `src/ban_tests.rs` for ban logic tests.
- Logging: Security events and ban actions are logged using Spin's logging macros.
- Performance: Early returns, minimal KV access, lightweight parsing, and optimized WASM build.

## Performance Checklist
- Early returns: Whitelist and ban checks short-circuit further logic
- Minimal key-value store reads/writes per request
- Lightweight header/cookie parsing
- Fixed time window for rate limiting
- No large in-memory state; all persistent state in edge KV
- Build with `--release` for optimized WASM

---

## Security
- All admin endpoints require API key authentication.
- Input validation and sanitization for all admin operations.
- JS challenge uses a secure, tamper-proof token tied to the visitor's IP.

---

## Roadmap
- Expand admin API for full configuration management
- Add more analytics and export options
- Integrate with additional edge geo/IP sources
- Add more unit and integration tests

---

See `src/` for implementation details and extend as needed.
