# 🐙 Deployment & Configuration

This project is designed to run on Spin (local or cloud). Use the Makefile for official flows where possible.

Shuma-Gorath is designed to **complement enterprise bot defenses** (such as Akamai Bot Manager) as a second-layer of application-specific protection, but it can also run standalone.

## 🐙 Required Secrets / Env Vars

- `SHUMA_API_KEY` - Admin API bearer token
- `SHUMA_JS_SECRET` - Signs the `js_verified` cookie
- `SHUMA_FORWARDED_IP_SECRET` - Required when trusting `X-Forwarded-For`
- `SHUMA_ADMIN_IP_ALLOWLIST` - Optional; CIDR/IP allowlist for admin access
- `SHUMA_EVENT_LOG_RETENTION_HOURS` - Event retention window
- `SHUMA_CONFIG_USE_KV` - `false` (default) or `true`
- `SHUMA_ADMIN_CONFIG_WRITE_ENABLED` - `false` (default) or `true`
- `SHUMA_KV_STORE_FAIL_OPEN` - `true` (default) or `false`
- `SHUMA_ENFORCE_HTTPS` - `false` (default) or `true`
- `SHUMA_DEBUG_HEADERS` - Optional; expose internal health/fail-mode headers (dev only)

API key requirements:

- Use a high-entropy 64-hex-character value for `SHUMA_API_KEY`.
- Placeholder keys are rejected by `make deploy`.
- Use `make api-key-generate` to create a valid key.

JS/PoW deployment recommendation:

- Keep `SHUMA_JS_REQUIRED_ENFORCED=true` in production.
- Keep `SHUMA_POW_ENABLED=true` for stronger, server-verified JS gate completion.
- Setting `SHUMA_JS_REQUIRED_ENFORCED=false` bypasses JS gate routing even if PoW endpoints are enabled.

`SHUMA_CONFIG_USE_KV=false` makes runtime config fully env-driven (KV config ignored).
`SHUMA_ADMIN_CONFIG_WRITE_ENABLED=false` disables `POST /admin/config`.
For immutable infrastructure-style deployments, set both to `false`.

Admin API surface defaults:

- Dashboard admin calls are same-origin by default (endpoint inferred from current page origin).
- Cross-origin admin API use is intentionally closed; `/admin/*` CORS preflight is rejected.
- Dashboard operators authenticate by entering `SHUMA_API_KEY` once, which creates a short-lived same-origin admin session cookie.

HTTPS enforcement:

- Set `SHUMA_ENFORCE_HTTPS=true` in production to reject non-HTTPS requests.
- Enforced failure mode is `403 HTTPS required`.
- The app trusts forwarded HTTPS proto headers only when forwarded-header trust is established (`SHUMA_FORWARDED_IP_SECRET` is configured and `X-Shuma-Forwarded-Secret` matches).

## 🐙 Forwarded IP Trust

When `SHUMA_FORWARDED_IP_SECRET` is set, the app only trusts `X-Forwarded-For` when the request also includes:

```
X-Shuma-Forwarded-Secret: <same value>
```

Configure your CDN/reverse proxy to add that header on inbound requests.

## 🐙 Fail-Open vs Fail-Closed (Critical Policy)

`SHUMA_KV_STORE_FAIL_OPEN` controls what happens when the KV store is unavailable during request handling:

- `true` (default): allow requests through and bypass checks
- `false`: return a 500 error and block

Choose this deliberately for your security posture. See `docs/security-hardening.md` for guidance.

## 🐙 Production Proxy Requirements

Shuma-Gorath is designed to run behind a CDN or reverse proxy that sets `X-Forwarded-For`.
In production, do not expose the Spin origin directly. Instead:

- Terminate TLS at your CDN/proxy
- Ensure the proxy injects `X-Forwarded-For`, `X-Forwarded-Proto` (or `Forwarded: proto=https`), and `X-Shuma-Forwarded-Secret` when enabled
- Firewall the origin to accept traffic only from the CDN/proxy IP ranges

If the origin is reachable directly, client IPs may appear as `unknown`. This is safe for the `/health` endpoint but not a substitute for proper origin protection.
If `SHUMA_ENFORCE_HTTPS=true` and forwarded proto trust is missing/misconfigured, requests are rejected with `403 HTTPS required`.

## 🐙 Outbound Network Policy

Runtime outbound HTTP(S) from the bot component is intentionally disabled via:

```toml
allowed_outbound_hosts = []
```

If you add a feature that needs external calls, explicitly allow only required hosts. Avoid wildcard allowlists.

## 🐙 Fermyon / Spin Cloud (Recommended)

Use Spin application variables to avoid committing secrets.
For a complete env var template, see `/.env.full.example` and `docs/configuration.md`.

Example `spin.toml` wiring:

```toml
[variables]
forwarded_ip_secret = { default = "" }
api_key = { default = "" }
js_secret = { default = "" }

[component.bot-trap]
environment = {
  SHUMA_API_KEY = "{{ api_key }}",
  SHUMA_JS_SECRET = "{{ js_secret }}",
  SHUMA_FORWARDED_IP_SECRET = "{{ forwarded_ip_secret }}"
}
```

Set variables in your cloud environment (CLI or console) before deploying. Use `make deploy` for the official deploy path.

### 🐙 Step-by-Step (Fermyon Cloud)

```bash
spin cloud login
make deploy
```

Optional custom domain:

```bash
spin cloud link --domain your-domain.example.com
```

### 🐙 Monitoring (Fermyon Cloud)

```bash
spin cloud logs
spin cloud apps info
spin cloud apps metrics
```

### 🐙 Forwarded IP Secret on Fermyon Cloud

If you set `SHUMA_FORWARDED_IP_SECRET`, you must inject the matching `X-Shuma-Forwarded-Secret` header upstream. If you do not inject that header, requests that rely on `X-Forwarded-For` are treated as untrusted.

## 🐙 Other Deploy Targets

- Set environment variables in your platform’s secret/config system (Kubernetes, Docker, systemd, etc.)
- Ensure your proxy/CDN adds `X-Shuma-Forwarded-Secret` when `SHUMA_FORWARDED_IP_SECRET` is set
- Use TLS for all admin traffic
- Restrict `/admin/*` access using `SHUMA_ADMIN_IP_ALLOWLIST` or platform firewall rules

## 🐙 Local Dev Defaults

`make dev` sets dev-only defaults for `SHUMA_FORWARDED_IP_SECRET` and `SHUMA_API_KEY`, enables internal debug headers, and passes them to Spin. Override as needed:

```bash
make dev SHUMA_FORWARDED_IP_SECRET="your-dev-secret" SHUMA_API_KEY="your-dev-api-key"
```

`make dev` sets `SHUMA_CONFIG_USE_KV=true` and `SHUMA_ADMIN_CONFIG_WRITE_ENABLED=true` by default.
To preview ENV-only behavior locally:

```bash
make dev DEV_CONFIG_USE_KV=false DEV_ADMIN_CONFIG_WRITE_ENABLED=false
```
Generate/rotate helper:

- `make api-key-generate` prints a new high-entropy API key
- `make api-key-rotate` prints a new key plus rotation steps
- `make api-key-validate` checks key format before deployment
