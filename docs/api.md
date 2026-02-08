# 🐙 API & Endpoints

## 🐙 Authentication

All admin endpoints require:
- `Authorization: Bearer <API_KEY>`
- If `ADMIN_IP_ALLOWLIST` is set, the client IP must be in the allowlist

If `FORWARDED_IP_SECRET` is configured, any request that relies on `X-Forwarded-For` must also include:
- `X-Shuma-Forwarded-Secret: <FORWARDED_IP_SECRET>`

If `API_KEY` is missing or set to the insecure default (`changeme-supersecret`), `/admin/*` endpoints are disabled.

## 🐙 Public Endpoints

- `GET /` - Main bot trap handler
- `GET /health` - Health check (loopback only)
- `GET /metrics` - Prometheus metrics (no auth)
- `GET /bot-trap` - Honeypot (triggers ban)
- `GET /pow` - PoW challenge seed (when enabled)
- `POST /pow/verify` - PoW verification (sets js_verified cookie)
- `POST /cdp-report` - Client automation reports (JSON)
- `GET /robots.txt` - robots.txt (configurable)
- `GET /dashboard/...` - Dashboard static assets
- `GET /challenge` - Dev-only challenge page (TEST_MODE only)
- `POST /challenge` - Challenge answer submission

### 🐙 Challenge Submission Format

`POST /challenge` expects:
- `seed` (signed challenge seed)
- `output` (base-3 string, length 16 for 4x4 grids)

Output encoding:
- `0` = empty
- `1` = black cell
- `2` = pink cell

### 🐙 Challenge Seed Lifecycle

- Seeds are short-lived and single-use.
- Any submit attempt consumes the seed, including incorrect attempts.
- Re-submitting a consumed or expired seed returns `403 Expired`.
- Invalid or tampered seed/token data returns `403 Forbidden. Please request a new challenge.`

Challenge submit responses:
- `200` - Correct answer (`Thank you! Challenge complete.`)
- `403` - Incorrect answer (`Incorrect.` + `Request new challenge.` link)
- `403` - Expired/replay (`Expired` + `Request new challenge.` link)
- `403` - Invalid token/signature/IP binding (`Forbidden. Please request a new challenge.` + link)

### 🐙 Health Check Example

```bash
curl -H "X-Forwarded-For: 127.0.0.1" \
  -H "X-Shuma-Forwarded-Secret: $FORWARDED_IP_SECRET" \
  http://127.0.0.1:3000/health
```

When `SHUMA_DEBUG_HEADERS=true`, the health response includes:
- `X-KV-Status` (available/unavailable)
- `X-Shuma-Fail-Mode` (open/closed)

## 🐙 Admin Endpoints

- `GET /admin` - API help
- `GET /admin/ban` - List active bans
- `POST /admin/ban` - Ban an IP (JSON body: `{"ip":"x.x.x.x","reason":"...","duration":3600}`)
- `POST /admin/unban?ip=x.x.x.x` - Unban an IP
- `GET /admin/analytics` - Ban/event statistics
- `GET /admin/events?hours=N` - Recent events + summary stats
- `GET /admin/config` - Read configuration
- `POST /admin/config` - Update configuration (partial JSON)
- `GET /admin/maze` - Link maze stats
- `GET /admin/robots` - robots.txt config and preview
- `GET /admin/cdp` - CDP detection config and stats

### 🐙 Analytics Response

`GET /admin/analytics` returns:
- `ban_count`
- `test_mode`
- `fail_mode`

### 🐙 Admin Events Response

`GET /admin/events?hours=24` returns:
- `recent_events` (up to 100 events)
- `event_counts` (counts per event type)
- `top_ips` (top 10 IPs by event count)
- `unique_ips` (distinct IP count)

### 🐙 Example: List Bans

```bash
curl -H "Authorization: Bearer $API_KEY" \
  http://127.0.0.1:3000/admin/ban
```

Each ban entry includes:
- `ip`
- `reason`
- `banned_at` (unix seconds)
- `expires` (unix seconds)
- `fingerprint` (optional):
- `score` (0-10 or null)
- `signals` (array of triggering signal keys)
- `summary` (human-readable context)

### 🐙 Example: Ban an IP

```bash
curl -X POST -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"ip":"1.2.3.4","reason":"admin_ban","duration":3600}' \
  http://127.0.0.1:3000/admin/ban
```

### 🐙 Example: Fetch Events

```bash
curl -H "Authorization: Bearer $API_KEY" \
  http://127.0.0.1:3000/admin/events?hours=24
```

## 🐙 Botness Policy Fields (`/admin/config`)

The unified botness model uses weighted scored signals plus terminal hard-ban signals.

Scored thresholds:
- `challenge_risk_threshold` - score at/above which challenge is served
- `botness_maze_threshold` - score at/above which requests are routed to maze

Scored weights:
- `botness_weights.js_required`
- `botness_weights.geo_risk`
- `botness_weights.rate_medium`
- `botness_weights.rate_high`

Mutability:
- `botness_config_mutable` indicates whether score/weight settings can be changed at runtime.
- Runtime mutation is disabled by default; enable with `BOTNESS_CONFIG_MUTABLE=true`.
- For dev compatibility, `CHALLENGE_CONFIG_MUTABLE=true` also enables botness mutation if `BOTNESS_CONFIG_MUTABLE` is unset.

Signal catalog:
- `botness_signal_definitions.scored_signals` lists weighted contributors.
- `botness_signal_definitions.terminal_signals` lists immediate actions that bypass scoring.
