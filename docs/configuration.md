# 🐙 Configuration

Configuration is stored in the Spin KV store under `config:<site_id>` and loaded per request. You can read and update it via `GET /admin/config` and `POST /admin/config`.

## 🐙 Update Config (Example)

```bash
curl -X POST -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"test_mode": true}' \
  http://127.0.0.1:3000/admin/config
```

## 🐙 Key Fields

- `test_mode` (bool) - Log actions without enforcing blocks
- `SHUMA_FAIL_MODE` (env) - `open` or `closed` when KV is unavailable (see `docs/security-hardening.md`)
- `rate_limit` (u32) - Requests per minute
- `honeypots` (string[]) - Honeypot paths (default: `/bot-trap`)
- `ban_duration` (u64) - Legacy single-duration fallback
- `ban_durations` (object) - Per-reason durations using keys `honeypot`, `rate_limit`, `browser`, `admin`, `cdp`
- `browser_block` (array of `[name, min_version]`) - Block outdated browsers
- `browser_whitelist` (array of `[name, min_version]`) - Bypass JS challenge
- `geo_risk` (string[]) - High-risk country codes (matches `X-Geo-Country`)
- `whitelist` (string[]) - CIDR/IP allowlist (supports inline comments)
- `path_whitelist` (string[]) - Exact paths or wildcard prefixes
- `maze_enabled` (bool) - Enable link maze
- `maze_auto_ban` (bool) - Auto-ban after threshold
- `maze_auto_ban_threshold` (u32) - Threshold for maze auto-ban
- `robots_enabled` (bool) - Serve `/robots.txt`
- `robots_block_ai_training` (bool)
- `robots_block_ai_search` (bool)
- `robots_allow_search_engines` (bool)
- `robots_crawl_delay` (u32)
- `cdp_detection_enabled` (bool)
- `cdp_auto_ban` (bool)
- `cdp_detection_threshold` (f32)
- `POW_ENABLED` (env) - Enable proof-of-work step before JS verification
- `POW_DIFFICULTY` (env) - Leading zero bits required (default: 15)
- `POW_TTL_SECONDS` (env) - Seed expiry in seconds (default: 90)
- `POW_SECRET` (env, optional) - HMAC secret for PoW seeds (falls back to `JS_SECRET`)

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
  "cdp_detection_threshold": 0.8
}
```
