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

### Admin API Endpoints
All endpoints require an `Authorization: Bearer <API_KEY>` header.

- `GET /admin/ban` — List all current bans (JSON: IP, reason, expiry)
- `POST /admin/unban?ip=...` — Unban a specific IP
- `GET /admin/analytics` — Get ban count analytics
- `GET /admin` — Usage help

### Configuration
- Ban duration, rate limit, honeypot URLs, browser blocklist, geo risk, and whitelist are stored in edge KV and can be managed via future admin endpoints or direct KV updates.

---

## Developer Notes

- Modular Rust code: see `src/` for ban, rate, JS, browser, geo, whitelist, honeypot, and admin logic.
- Unit tests: see `src/ban_tests.rs` for ban logic tests (expand as needed).
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
