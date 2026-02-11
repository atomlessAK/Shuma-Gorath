# 🐙 Configuration

Shuma-Gorath uses one runtime configuration model:

- Tunables are stored in KV under `config:<site_id>` (default `config:default`).
- Env vars are reserved for secrets and runtime guardrails.
- `config/defaults.env` is the canonical source for defaults (no hidden tunable defaults in Rust).

## 🐙 Startup Model

`make setup`:

1. Creates `.env.local` from `config/defaults.env` if missing.
2. Generates local dev secrets in `.env.local` (`SHUMA_API_KEY`, `SHUMA_JS_SECRET`, `SHUMA_FORWARDED_IP_SECRET`).
3. Runs `make config-seed`, which writes KV tunables only when `config:default` does not already exist.

At runtime:

- Tunables are loaded from KV.
- Env-only keys are loaded from process env.
- Missing/invalid KV config returns `500 Configuration unavailable` for config-dependent requests.

## 🐙 Runtime Config Cache

- KV-backed runtime config is cached in-process for a short TTL (currently 2 seconds).
- `POST /admin/config` invalidates the cache on the handling instance immediately after a successful KV write.
- In multi-instance deployments, other instances refresh on their own TTL window, so brief config staleness (up to TTL) is expected.

## 🐙 Canonical Variable Reference

Source of truth files:

- `config/defaults.env` (all defaults)
- `/.env.full.example` (env-only template for deployment)

### 🐙 Env-Only Runtime Keys

These are read from process env at runtime (not from KV).

| Variable | Required | Default in `config/defaults.env` | Purpose |
| --- | --- | --- | --- |
| `SHUMA_API_KEY` | Yes | `changeme-prod-api-key` | Admin authentication key for dashboard login and `Authorization: Bearer` admin API calls. |
| `SHUMA_JS_SECRET` | Yes | `changeme-prod-js-secret` | Signs and verifies `js_verified` cookie. |
| `SHUMA_POW_SECRET` | No | empty | Optional dedicated PoW signing secret. Falls back to `SHUMA_JS_SECRET` when unset. |
| `SHUMA_CHALLENGE_SECRET` | No | empty | Optional dedicated challenge signing secret. Falls back to `SHUMA_JS_SECRET` when unset. |
| `SHUMA_FORWARDED_IP_SECRET` | Yes | `changeme-prod-forwarded-ip-secret` | Trust boundary secret for forwarded IP/proto headers (`X-Shuma-Forwarded-Secret`). |
| `SHUMA_HEALTH_SECRET` | No | empty | Optional shared secret for `/health` via `X-Shuma-Health-Secret`. |
| `SHUMA_ADMIN_IP_ALLOWLIST` | No | empty | Optional CIDR/IP allowlist for `/admin/*`. |
| `SHUMA_ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE` | No | `10` | Per-IP per-minute limit for failed admin authentication attempts before returning `429`. |
| `SHUMA_EVENT_LOG_RETENTION_HOURS` | Yes | `168` | Event retention window in hours (`0` disables cleanup). |
| `SHUMA_ADMIN_CONFIG_WRITE_ENABLED` | Yes | `false` | Enables/disables admin config writes to KV (`POST /admin/config`). |
| `SHUMA_KV_STORE_FAIL_OPEN` | Yes | `true` | KV failure policy (`true` fail-open, `false` fail-closed). |
| `SHUMA_ENFORCE_HTTPS` | Yes | `false` | Rejects non-HTTPS requests when `true` (proxy/header trust rules still apply). |
| `SHUMA_DEBUG_HEADERS` | Yes | `false` | Enables internal debug headers (keep `false` in production). |
| `SHUMA_POW_CONFIG_MUTABLE` | Yes | `false` | Allows runtime edits of PoW difficulty/TTL from admin config. |
| `SHUMA_CHALLENGE_CONFIG_MUTABLE` | Yes | `false` | Allows runtime edits of challenge transform count/threshold from admin config. |
| `SHUMA_BOTNESS_CONFIG_MUTABLE` | Yes | `false` | Allows runtime edits of botness thresholds/weights from admin config. |

Use `make env-help` for the supported env-only override list.

### 🐙 KV Tunables (Seeded From `config/defaults.env`)

These keys are seeded into KV and loaded from KV at runtime.

| Variable | Default | Purpose |
| --- | --- | --- |
| `SHUMA_TEST_MODE` | `false` | Enables test-mode behavior for controlled local testing. |
| `SHUMA_JS_REQUIRED_ENFORCED` | `true` | Enforces JS verification (`js_verified` cookie gate). |
| `SHUMA_POW_ENABLED` | `true` | Enables PoW in JS verification flow. |
| `SHUMA_POW_DIFFICULTY` | `15` | PoW cost level (clamped to supported range). |
| `SHUMA_POW_TTL_SECONDS` | `90` | PoW seed lifetime in seconds (clamped). |
| `SHUMA_CHALLENGE_TRANSFORM_COUNT` | `6` | Number of transform options shown in the puzzle challenge (4-8). |
| `SHUMA_CHALLENGE_RISK_THRESHOLD` | `3` | Botness score threshold for serving challenge step-up. |
| `SHUMA_BOTNESS_MAZE_THRESHOLD` | `6` | Botness score threshold for routing to maze. |
| `SHUMA_BOTNESS_WEIGHT_JS_REQUIRED` | `1` | Score weight for missing JS verification signal. |
| `SHUMA_BOTNESS_WEIGHT_GEO_RISK` | `2` | Score weight for GEO risk-country signal. |
| `SHUMA_BOTNESS_WEIGHT_RATE_MEDIUM` | `1` | Score weight for medium request-rate pressure. |
| `SHUMA_BOTNESS_WEIGHT_RATE_HIGH` | `2` | Score weight for high request-rate pressure. |
| `SHUMA_BAN_DURATION` | `21600` | Legacy/default ban duration fallback (seconds). |
| `SHUMA_BAN_DURATION_HONEYPOT` | `86400` | Ban duration for honeypot/instaban trigger (seconds). |
| `SHUMA_BAN_DURATION_RATE_LIMIT` | `3600` | Ban duration for rate-limit ban (seconds). |
| `SHUMA_BAN_DURATION_BROWSER` | `21600` | Ban duration for browser-policy based bans (seconds). |
| `SHUMA_BAN_DURATION_ADMIN` | `21600` | Ban duration for manual admin bans (seconds). |
| `SHUMA_BAN_DURATION_CDP` | `43200` | Ban duration for CDP automation bans (seconds). |
| `SHUMA_RATE_LIMIT` | `80` | Requests per minute threshold for rate limiting. |
| `SHUMA_HONEYPOTS` | `['/instaban']` | Honeypot endpoints that immediately trigger ban flow. |
| `SHUMA_BROWSER_BLOCK` | `[["Chrome",120],["Firefox",115],["Safari",15]]` | Browser/version minimums used by browser policy checks. |
| `SHUMA_BROWSER_WHITELIST` | `[]` | Optional browser/version allowlist exceptions. |
| `SHUMA_GEO_RISK_COUNTRIES` | `[]` | 2-letter countries that add GEO botness score. |
| `SHUMA_GEO_ALLOW_COUNTRIES` | `[]` | 2-letter countries explicitly allowed in GEO routing precedence. |
| `SHUMA_GEO_CHALLENGE_COUNTRIES` | `[]` | 2-letter countries forced to challenge tier. |
| `SHUMA_GEO_MAZE_COUNTRIES` | `[]` | 2-letter countries forced to maze tier. |
| `SHUMA_GEO_BLOCK_COUNTRIES` | `[]` | 2-letter countries forced to block tier. |
| `SHUMA_WHITELIST` | `[]` | IP/CIDR allowlist bypassing bot defenses. |
| `SHUMA_PATH_WHITELIST` | `[]` | URL path allowlist bypassing bot defenses. |
| `SHUMA_MAZE_ENABLED` | `true` | Enables maze feature. |
| `SHUMA_MAZE_AUTO_BAN` | `true` | Enables maze auto-ban when threshold is exceeded. |
| `SHUMA_MAZE_AUTO_BAN_THRESHOLD` | `50` | Maze hit threshold for auto-ban. |
| `SHUMA_ROBOTS_ENABLED` | `true` | Enables robots.txt endpoint and policy generation. |
| `SHUMA_ROBOTS_BLOCK_AI_TRAINING` | `true` | Adds AI training bot disallow directives. |
| `SHUMA_ROBOTS_BLOCK_AI_SEARCH` | `false` | Adds AI search bot disallow directives. |
| `SHUMA_ROBOTS_ALLOW_SEARCH_ENGINES` | `true` | Allows mainstream search engines in robots policy. |
| `SHUMA_ROBOTS_CRAWL_DELAY` | `2` | robots.txt crawl-delay value (seconds). |
| `SHUMA_CDP_DETECTION_ENABLED` | `true` | Enables CDP automation detection script/processing. |
| `SHUMA_CDP_AUTO_BAN` | `true` | Enables auto-ban path when strong CDP automation is detected. |
| `SHUMA_CDP_DETECTION_THRESHOLD` | `0.8` | CDP score threshold used when hard CDP checks are absent. |

## 🐙 Admin Config Writes

- `GET /admin/config` reads effective KV-backed config.
- `POST /admin/config` writes to KV only when `SHUMA_ADMIN_CONFIG_WRITE_ENABLED=true`.
- Successful writes invalidate runtime config cache on the instance that processed the request.
- KV writes persist across restarts.

## 🐙 JS Verification + PoW

- `js_required_enforced=true` routes visitors without a valid `js_verified` cookie to JS verification.
- `pow_enabled=true` adds server-verified PoW to that verification flow.
- `js_required_enforced=false` bypasses JS verification for normal requests (and therefore bypasses PoW on that path).

## 🐙 GEO Trust Boundary

GEO/proto headers are trusted only when:

- `SHUMA_FORWARDED_IP_SECRET` is configured, and
- request includes matching `X-Shuma-Forwarded-Secret`.

Without trust, forwarded IP/proto/GEO-derived routing and GEO scoring are skipped.
