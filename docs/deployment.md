# 🐙 Deployment & Configuration

This project is designed to run on Spin (local or cloud). Use the Makefile for official flows where possible.

Shuma-Gorath is designed to **complement enterprise bot defenses** (such as Akamai Bot Manager) as a second-layer of application-specific protection, but it can also run standalone.

## 🐙 Required Secrets / Env Vars

- `SHUMA_API_KEY` - Admin API bearer token
- `SHUMA_JS_SECRET` - Signs the `js_verified` cookie
- `SHUMA_FORWARDED_IP_SECRET` - Required when trusting `X-Forwarded-For`
- `SHUMA_ADMIN_IP_ALLOWLIST` - Optional; CIDR/IP allowlist for admin access
- `SHUMA_EVENT_LOG_RETENTION_HOURS` - Event retention window
- `SHUMA_CONFIG_MODE` - `hybrid` (default) or `env_only`
- `SHUMA_KV_STORE_FAIL_MODE` - `open` (default) or `closed`
- `SHUMA_DEBUG_HEADERS` - Optional; expose internal health/fail-mode headers (dev only)

`SHUMA_CONFIG_MODE=env_only` makes runtime config fully env-driven and disables `POST /admin/config`.
Use this for immutable infrastructure-style deployments.

## 🐙 Forwarded IP Trust

When `SHUMA_FORWARDED_IP_SECRET` is set, the app only trusts `X-Forwarded-For` when the request also includes:

```
X-Shuma-Forwarded-Secret: <same value>
```

Configure your CDN/reverse proxy to add that header on inbound requests.

## 🐙 Fail-Open vs Fail-Closed (Critical Policy)

`SHUMA_KV_STORE_FAIL_MODE` controls what happens when the KV store is unavailable during request handling:

- `open` (default): allow requests through and bypass checks
- `closed`: return a 500 error and block

Choose this deliberately for your security posture. See `docs/security-hardening.md` for guidance.

## 🐙 Production Proxy Requirements

Shuma-Gorath is designed to run behind a CDN or reverse proxy that sets `X-Forwarded-For`.
In production, do not expose the Spin origin directly. Instead:

- Terminate TLS at your CDN/proxy
- Ensure the proxy injects `X-Forwarded-For` (and `X-Shuma-Forwarded-Secret` if enabled)
- Firewall the origin to accept traffic only from the CDN/proxy IP ranges

If the origin is reachable directly, client IPs may appear as `unknown`. This is safe for the `/health` endpoint but not a substitute for proper origin protection.

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
