# 🐙 Configuration

Shuma-Gorath runtime config control is governed by two independent env vars:

- `SHUMA_CONFIG_USE_KV`:
- `false` (default): env-only runtime config; KV `config:<site_id>` is ignored.
- `true`: runtime loads KV `config:<site_id>` as base, then applies env overrides.
- `SHUMA_ADMIN_CONFIG_WRITE_ENABLED`:
- `false` (default): `POST /admin/config` is disabled.
- `true`: admin config writes to KV are enabled.

Operational note:
- If `SHUMA_ADMIN_CONFIG_WRITE_ENABLED=true` and `SHUMA_CONFIG_USE_KV=false`, admin saves persist in KV but do not affect effective runtime until `SHUMA_CONFIG_USE_KV=true`.

You can read current runtime config via `GET /admin/config`.

## 🐙 Full Copy/Paste Templates

Canonical code template:
- `/.env.full.example`

Maintainer note:
- When adding/changing env vars in code, update both `/.env.full.example` and this document.

Full `spin.toml` environment object (copy/paste, then edit values):

```toml
[component.bot-trap]
environment = {
  SHUMA_API_KEY = "changeme-prod-api-key",
  SHUMA_JS_SECRET = "changeme-prod-js-secret",
  SHUMA_FORWARDED_IP_SECRET = "changeme-prod-forwarded-ip-secret",
  SHUMA_ADMIN_IP_ALLOWLIST = "",
  SHUMA_EVENT_LOG_RETENTION_HOURS = "168",
  SHUMA_CONFIG_USE_KV = "false",
  SHUMA_ADMIN_CONFIG_WRITE_ENABLED = "false",
  SHUMA_KV_STORE_FAIL_OPEN = "true",
  SHUMA_ENFORCE_HTTPS = "false",
  SHUMA_DEBUG_HEADERS = "false",
  SHUMA_TEST_MODE = "false",
  SHUMA_JS_REQUIRED_ENFORCED = "true",
  SHUMA_POW_ENABLED = "true",
  SHUMA_POW_SECRET = "",
  SHUMA_POW_DIFFICULTY = "15",
  SHUMA_POW_TTL_SECONDS = "90",
  SHUMA_POW_CONFIG_MUTABLE = "false",
  SHUMA_CHALLENGE_SECRET = "",
  SHUMA_CHALLENGE_TRANSFORM_COUNT = "6",
  SHUMA_CHALLENGE_RISK_THRESHOLD = "3",
  SHUMA_CHALLENGE_CONFIG_MUTABLE = "false",
  SHUMA_BOTNESS_CONFIG_MUTABLE = "false",
  SHUMA_BOTNESS_MAZE_THRESHOLD = "6",
  SHUMA_BOTNESS_WEIGHT_JS_REQUIRED = "1",
  SHUMA_BOTNESS_WEIGHT_GEO_RISK = "2",
  SHUMA_BOTNESS_WEIGHT_RATE_MEDIUM = "1",
  SHUMA_BOTNESS_WEIGHT_RATE_HIGH = "2",
  SHUMA_BAN_DURATION = "21600",
  SHUMA_BAN_DURATION_HONEYPOT = "86400",
  SHUMA_BAN_DURATION_RATE_LIMIT = "3600",
  SHUMA_BAN_DURATION_BROWSER = "21600",
  SHUMA_BAN_DURATION_ADMIN = "21600",
  SHUMA_BAN_DURATION_CDP = "43200",
  SHUMA_RATE_LIMIT = "80",
  SHUMA_HONEYPOTS = "[\"/bot-trap\"]",
  SHUMA_BROWSER_BLOCK = "[[\"Chrome\",120],[\"Firefox\",115],[\"Safari\",15]]",
  SHUMA_BROWSER_WHITELIST = "[]",
  SHUMA_GEO_RISK_COUNTRIES = "[]",
  SHUMA_GEO_ALLOW_COUNTRIES = "[]",
  SHUMA_GEO_CHALLENGE_COUNTRIES = "[]",
  SHUMA_GEO_MAZE_COUNTRIES = "[]",
  SHUMA_GEO_BLOCK_COUNTRIES = "[]",
  SHUMA_WHITELIST = "[]",
  SHUMA_PATH_WHITELIST = "[]",
  SHUMA_MAZE_ENABLED = "true",
  SHUMA_MAZE_AUTO_BAN = "true",
  SHUMA_MAZE_AUTO_BAN_THRESHOLD = "50",
  SHUMA_ROBOTS_ENABLED = "true",
  SHUMA_ROBOTS_BLOCK_AI_TRAINING = "true",
  SHUMA_ROBOTS_BLOCK_AI_SEARCH = "false",
  SHUMA_ROBOTS_ALLOW_SEARCH_ENGINES = "true",
  SHUMA_ROBOTS_CRAWL_DELAY = "2",
  SHUMA_CDP_DETECTION_ENABLED = "true",
  SHUMA_CDP_AUTO_BAN = "true",
  SHUMA_CDP_DETECTION_THRESHOLD = "0.8"
}
```

## 🐙 Update Config (Example)

```bash
curl -X POST -H "Authorization: Bearer $SHUMA_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"test_mode": true}' \
  http://127.0.0.1:3000/admin/config
```

## 🐙 Core Mode & Policy Env Vars

- `SHUMA_CONFIG_USE_KV` - `false` (default) or `true`
- `SHUMA_ADMIN_CONFIG_WRITE_ENABLED` - `false` (default) or `true`
- `SHUMA_KV_STORE_FAIL_OPEN` - `true` (default) or `false` when KV is unavailable
- `SHUMA_ENFORCE_HTTPS` - `false` (default) or `true` to reject non-HTTPS requests
- `SHUMA_TEST_MODE` - Log-only mode (`true/false`, `1/0`)
- `SHUMA_JS_REQUIRED_ENFORCED` - Enable/disable JS-required enforcement (`true/false`, `1/0`)

## 🐙 JS Verification + PoW Interaction

These controls are related but not identical:

- `SHUMA_JS_REQUIRED_ENFORCED` controls whether normal request routing enforces the JS verification gate.
- `SHUMA_POW_ENABLED` controls whether that JS gate requires server-verified PoW before issuing `js_verified`.

Effective behavior:

- `JS_REQUIRED=true` + `POW=true`:
  - strongest mode; interstitial requires PoW and server sets `js_verified` on `/pow/verify`.
  - after a valid `js_verified` cookie is set, the originally requested page reloads and access is re-evaluated.
- `JS_REQUIRED=true` + `POW=false`:
  - interstitial still runs, but sets `js_verified` directly in browser JS (weaker, lower friction).
  - after a valid `js_verified` cookie is set, the originally requested page reloads and access is re-evaluated.
- `JS_REQUIRED=false` + `POW=true/false`:
  - normal routing does not use the JS gate; PoW endpoints can exist but are not on the default access path.

## 🐙 Full Runtime Config Via Env Vars

All runtime config fields can be set with env vars:

- `SHUMA_BAN_DURATION`
- `SHUMA_BAN_DURATION_HONEYPOT`
- `SHUMA_BAN_DURATION_RATE_LIMIT`
- `SHUMA_BAN_DURATION_BROWSER`
- `SHUMA_BAN_DURATION_ADMIN`
- `SHUMA_BAN_DURATION_CDP`
- `SHUMA_RATE_LIMIT`
- `SHUMA_HONEYPOTS`
- `SHUMA_BROWSER_BLOCK`
- `SHUMA_BROWSER_WHITELIST`
- `SHUMA_GEO_RISK_COUNTRIES`
- `SHUMA_GEO_ALLOW_COUNTRIES`
- `SHUMA_GEO_CHALLENGE_COUNTRIES`
- `SHUMA_GEO_MAZE_COUNTRIES`
- `SHUMA_GEO_BLOCK_COUNTRIES`
- `SHUMA_WHITELIST`
- `SHUMA_PATH_WHITELIST`
- `SHUMA_ENFORCE_HTTPS`
- `SHUMA_MAZE_ENABLED`
- `SHUMA_MAZE_AUTO_BAN`
- `SHUMA_MAZE_AUTO_BAN_THRESHOLD`
- `SHUMA_ROBOTS_ENABLED`
- `SHUMA_ROBOTS_BLOCK_AI_TRAINING`
- `SHUMA_ROBOTS_BLOCK_AI_SEARCH`
- `SHUMA_ROBOTS_ALLOW_SEARCH_ENGINES`
- `SHUMA_ROBOTS_CRAWL_DELAY`
- `SHUMA_CDP_DETECTION_ENABLED`
- `SHUMA_CDP_AUTO_BAN`
- `SHUMA_CDP_DETECTION_THRESHOLD`
- `SHUMA_JS_REQUIRED_ENFORCED`
- `SHUMA_POW_DIFFICULTY`
- `SHUMA_POW_TTL_SECONDS`
- `SHUMA_CHALLENGE_RISK_THRESHOLD`
- `SHUMA_BOTNESS_MAZE_THRESHOLD`
- `SHUMA_BOTNESS_WEIGHT_JS_REQUIRED`
- `SHUMA_BOTNESS_WEIGHT_GEO_RISK`
- `SHUMA_BOTNESS_WEIGHT_RATE_MEDIUM`
- `SHUMA_BOTNESS_WEIGHT_RATE_HIGH`

GEO trust boundary and policy routing:

- GEO headers are only trusted when forwarded headers are trusted (`SHUMA_FORWARDED_IP_SECRET` must be set and `X-Shuma-Forwarded-Secret` must match).
- If forwarded trust is not established for the request, GEO routing/scoring is skipped.
- GEO policy precedence is `block > maze > challenge > allow`.
- `allow` suppresses GEO botness scoring for matching countries.

CDP auto-ban policy:

- Auto-ban applies only to strong CDP detections.
- Hard checks such as `webdriver` and `automation_props` are treated as strong signals.
- If hard checks are absent, a report is medium when `score >= SHUMA_CDP_DETECTION_THRESHOLD` (or when `cdp_timing` is present).
- Without hard checks, strong requires `score >= SHUMA_CDP_DETECTION_THRESHOLD + 0.4` and at least two soft checks (`cdp_timing`, `chrome_obj`, `plugins`).
- Automatic ban applies only when final CDP tier is strong and `SHUMA_CDP_AUTO_BAN=true`.

Supporting control vars:

- `SHUMA_POW_ENABLED`
- `SHUMA_POW_SECRET`
- `SHUMA_POW_CONFIG_MUTABLE`
- `SHUMA_CHALLENGE_CONFIG_MUTABLE`
- `SHUMA_BOTNESS_CONFIG_MUTABLE`
- `SHUMA_CHALLENGE_TRANSFORM_COUNT`

## 🐙 Env Value Formats

- Booleans: `true/false`, `1/0`, `yes/no`, `on/off`
- String lists (`SHUMA_HONEYPOTS`, `SHUMA_GEO_RISK_COUNTRIES`, `SHUMA_GEO_ALLOW_COUNTRIES`, `SHUMA_GEO_CHALLENGE_COUNTRIES`, `SHUMA_GEO_MAZE_COUNTRIES`, `SHUMA_GEO_BLOCK_COUNTRIES`, `SHUMA_WHITELIST`, `SHUMA_PATH_WHITELIST`):
  - JSON array: `["/bot-trap","/canary"]`
  - CSV: `/bot-trap,/canary`
- Browser lists (`SHUMA_BROWSER_BLOCK`, `SHUMA_BROWSER_WHITELIST`):
  - JSON array of tuples: `[["Chrome",120],["Firefox",115]]`
  - CSV tuples: `Chrome:120,Firefox:115`

## 🐙 Mutability Notes

- `SHUMA_POW_DIFFICULTY` and `SHUMA_POW_TTL_SECONDS` are always env-controlled unless `SHUMA_POW_CONFIG_MUTABLE=1`.
- `SHUMA_CHALLENGE_RISK_THRESHOLD`, `SHUMA_BOTNESS_MAZE_THRESHOLD`, and `SHUMA_BOTNESS_WEIGHT_*` are env-controlled unless `SHUMA_BOTNESS_CONFIG_MUTABLE=true` (or legacy fallback `SHUMA_CHALLENGE_CONFIG_MUTABLE=true`).
- `SHUMA_ADMIN_CONFIG_WRITE_ENABLED=false` blocks admin writes regardless of mutability flags.
- `SHUMA_CONFIG_USE_KV=false` makes runtime config env-only (KV config ignored).
- Local `make dev` sets both `SHUMA_CONFIG_USE_KV=true` and `SHUMA_ADMIN_CONFIG_WRITE_ENABLED=true` by default for easier dashboard testing.

## 🐙 Example Config (Partial)

```json
{
  "test_mode": false,
  "js_required_enforced": true,
  "rate_limit": 80,
  "honeypots": ["/bot-trap"],
  "ban_durations": {
    "honeypot": 86400,
    "rate_limit": 3600,
    "browser": 21600,
    "admin": 21600,
    "cdp": 43200
  },
  "browser_block": [["Chrome", 120], ["Firefox", 115], ["Safari", 15]],
  "browser_whitelist": [],
  "geo_risk": ["CN", "RU"],
  "geo_allow": ["GB"],
  "geo_challenge": ["BR"],
  "geo_maze": ["RU"],
  "geo_block": ["KP"],
  "whitelist": ["203.0.113.0/24 # corp"],
  "path_whitelist": ["/webhook/stripe", "/api/integration/*"],
  "maze_enabled": true,
  "maze_auto_ban": true,
  "maze_auto_ban_threshold": 50,
  "robots_enabled": true,
  "robots_block_ai_training": true,
  "robots_allow_search_engines": true,
  "robots_crawl_delay": 2,
  "cdp_detection_enabled": true,
  "cdp_auto_ban": true,
  "cdp_detection_threshold": 0.8,
  "pow_difficulty": 15,
  "pow_ttl_seconds": 90,
  "challenge_risk_threshold": 3
}
```
