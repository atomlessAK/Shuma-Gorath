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
Template file: `/.env.full.example`.

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
