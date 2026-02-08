# 🐙 Configuration

Shuma-Gorath supports two config source modes controlled by `SHUMA_CONFIG_MODE`:

- `hybrid` (default): load from KV (`config:<site_id>`) and then apply env var overrides.
- `env_only`: ignore KV, build config from defaults + env vars only.

In `env_only`, `POST /admin/config` is disabled and returns `403`.

You can read current runtime config via `GET /admin/config`.

## 🐙 Update Config (Example)

```bash
curl -X POST -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"test_mode": true}' \
  http://127.0.0.1:3000/admin/config
```

## 🐙 Core Mode & Policy Env Vars

- `SHUMA_CONFIG_MODE` - `hybrid` (default) or `env_only`
- `SHUMA_KV_STORE_FAIL_MODE` - `open` or `closed` when KV is unavailable
- `TEST_MODE` - Log-only mode (`true/false`, `1/0`)

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
- `SHUMA_GEO_RISK`
- `SHUMA_WHITELIST`
- `SHUMA_PATH_WHITELIST`
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
- `POW_DIFFICULTY`
- `POW_TTL_SECONDS`
- `CHALLENGE_RISK_THRESHOLD`
- `BOTNESS_MAZE_THRESHOLD`
- `BOTNESS_WEIGHT_JS_REQUIRED`
- `BOTNESS_WEIGHT_GEO_RISK`
- `BOTNESS_WEIGHT_RATE_MEDIUM`
- `BOTNESS_WEIGHT_RATE_HIGH`

Supporting control vars:

- `POW_ENABLED`
- `POW_SECRET`
- `POW_CONFIG_MUTABLE`
- `CHALLENGE_CONFIG_MUTABLE`
- `BOTNESS_CONFIG_MUTABLE`
- `CHALLENGE_TRANSFORM_COUNT`

## 🐙 Env Value Formats

- Booleans: `true/false`, `1/0`, `yes/no`, `on/off`
- String lists (`SHUMA_HONEYPOTS`, `SHUMA_GEO_RISK`, `SHUMA_WHITELIST`, `SHUMA_PATH_WHITELIST`):
  - JSON array: `["/bot-trap","/canary"]`
  - CSV: `/bot-trap,/canary`
- Browser lists (`SHUMA_BROWSER_BLOCK`, `SHUMA_BROWSER_WHITELIST`):
  - JSON array of tuples: `[["Chrome",120],["Firefox",115]]`
  - CSV tuples: `Chrome:120,Firefox:115`

## 🐙 Mutability Notes

- `POW_DIFFICULTY` and `POW_TTL_SECONDS` are always env-controlled unless `POW_CONFIG_MUTABLE=1`.
- `CHALLENGE_RISK_THRESHOLD`, `BOTNESS_MAZE_THRESHOLD`, and `BOTNESS_WEIGHT_*` are env-controlled unless `BOTNESS_CONFIG_MUTABLE=true` (or legacy fallback `CHALLENGE_CONFIG_MUTABLE=true`).
- In `env_only`, admin writes are blocked regardless of mutability flags.

## 🐙 Example Config (Partial)

```json
{
  "test_mode": false,
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
