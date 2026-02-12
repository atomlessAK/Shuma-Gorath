# 🐙 Deployment & Configuration

Shuma-Gorath is designed to run on Spin (local or cloud). Use the Makefile paths as the official workflow.

Shuma-Gorath is intended to complement enterprise bot defenses (for example Akamai Bot Manager), but can run standalone.

## 🐙 Runtime Configuration Model

- Tunables are loaded from KV (`config:default`) only.
- Env vars are secrets/guardrails only.
- `make setup` seeds KV tunables from `config/defaults.env` using `make config-seed`.
- Runtime config is process-cached for a short TTL (2 seconds) to reduce hot-path KV reads.
- `POST /admin/config` invalidates cache on the handling instance; other instances converge on their TTL window.

If KV config is missing/invalid at runtime, config-dependent request handling fails with `500 Configuration unavailable`.

## 🐙 Required Env-Only Keys

Set these in your deployment secret/config system:

- `SHUMA_API_KEY`
- `SHUMA_JS_SECRET`
- `SHUMA_FORWARDED_IP_SECRET` (required when trusting forwarded headers)
- `SHUMA_HEALTH_SECRET` (recommended; required if you want header-authenticated `/health`)
- `SHUMA_ADMIN_CONFIG_WRITE_ENABLED`
- `SHUMA_KV_STORE_FAIL_OPEN`
- `SHUMA_ENFORCE_HTTPS`
- `SHUMA_DEBUG_HEADERS`

For the full env-only list and per-variable behavior, use `docs/configuration.md`.
Template source: run `make setup` and use `.env.local` (gitignored) as your env-only override baseline.

## 🐙 Security Baseline

- Keep `SHUMA_DEBUG_HEADERS=false` in production.
- Keep `SHUMA_ENFORCE_HTTPS=true` in production.
- Keep `SHUMA_ADMIN_CONFIG_WRITE_ENABLED=false` unless you explicitly need live tuning.
- Generate a strong `SHUMA_API_KEY` with `make api-key-generate` (or rotate with `make api-key-rotate`).
- Set `SHUMA_HEALTH_SECRET` and require `X-Shuma-Health-Secret` for `/health`.
- Restrict `/admin/*` with `SHUMA_ADMIN_IP_ALLOWLIST` and upstream network controls.
- Apply CDN/WAF rate limits to `POST /admin/login` and all `/admin/*`.

Validation helper before deploy:

```bash
make deploy-env-validate
```

## 🐙 CDN/WAF Rate Limits (Cloudflare + Akamai)

Treat this as first-layer abuse control. Keep app-level auth and rate-limiting logic enabled as a second layer.

Recommended baseline policies:

- `POST /admin/login`: strict limit (start around `5 requests/minute/IP`, burst up to `10`).
- All other `/admin/*`: moderate limit (start around `60 requests/minute/IP`).
- Exempt trusted operator and monitoring source IPs/CIDRs to avoid self-lockout.

### 🐙 Cloudflare

Use WAF Rate Limiting rules with client IP as the key characteristic.

Suggested rules:

1. Login endpoint:
   - Match expression: `http.request.method eq "POST" and http.request.uri.path eq "/admin/login"`
   - Initial action: `Managed Challenge` (or `Block` for API-only admin workflows)
2. Admin surface:
   - Match expression: `starts_with(http.request.uri.path, "/admin/") and http.request.uri.path ne "/admin/login"`
   - Initial action: `Managed Challenge` or `Block` based on your operator UX requirements

Operational notes:

- Start in monitor/challenge mode, review false positives, then tighten.
- Ensure Cloudflare uses the real client IP signal from your edge chain.
- Keep `/admin/*` route protections in place even after app-level distributed limiter work.

### 🐙 Akamai

Use App & API Protector rate controls/rate policies keyed by client IP.

Suggested policies:

1. Login endpoint policy:
   - Match target: path `/admin/login` + method `POST`
   - Threshold: strict (same baseline as above; tune with observed traffic)
2. Admin surface policy:
   - Match target: path prefix `/admin/` excluding login
   - Threshold: moderate (same baseline as above; tune with observed traffic)

Operational notes:

- Roll out in alert/monitor mode first, then enforce deny/challenge actions.
- Confirm client IP restoration (`True-Client-IP`/equivalent) so limits key on users, not intermediate proxies.
- Keep these policies as a permanent first layer; they are not throwaway once distributed app-level limiting is added.

## 🐙 Forwarded Header Trust

When `SHUMA_FORWARDED_IP_SECRET` is set, forwarded client/proto headers are trusted only if request includes:

```http
X-Shuma-Forwarded-Secret: <same secret>
```

Configure your CDN/reverse proxy to inject this header.
Also sanitize incoming `X-Forwarded-For` / `X-Real-IP` from untrusted clients and overwrite with edge-observed values.

## 🐙 Health Endpoint Hardening

- `/health` allows loopback IPs only (`127.0.0.1`, `::1`) after trusted forwarded-IP extraction.
- For defense in depth, set `SHUMA_HEALTH_SECRET` and require monitors/proxies to send:

```http
X-Shuma-Health-Secret: <same secret>
```

- Strip `X-Shuma-Health-Secret` from public inbound traffic at your edge and only inject it from trusted monitoring/proxy paths.

## 🐙 Fail-Open vs Fail-Closed

`SHUMA_KV_STORE_FAIL_OPEN` controls behavior when KV is unavailable:

- `true`: allow requests through (reduced protection)
- `false`: block with server error (stricter posture)

Choose deliberately for your production risk posture.

## 🐙 Outbound Policy

Outbound HTTP(S) is disabled by default:

```toml
allowed_outbound_hosts = []
```

Only add explicit hosts if a new feature requires outbound calls.

## 🐙 Fermyon / Spin Cloud

Example variable wiring:

```toml
[variables]
api_key = { default = "" }
js_secret = { default = "" }
forwarded_ip_secret = { default = "" }

[component.bot-defence]
environment = {
  SHUMA_API_KEY = "{{ api_key }}",
  SHUMA_JS_SECRET = "{{ js_secret }}",
  SHUMA_FORWARDED_IP_SECRET = "{{ forwarded_ip_secret }}"
}
```

Deploy:

```bash
spin cloud login
make deploy
```

## 🐙 Local Dev

`make setup` creates `.env.local`, generates dev secrets, and seeds KV defaults.

```bash
make setup
make dev
make api-key-show
```

`make dev` enables dev-mode defaults for local operation and dashboard testing.
