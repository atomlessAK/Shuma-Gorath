# 🐙 Security Hardening

This guide focuses on deployment security controls and operational hardening.

## 🐙 Run Behind a CDN / Reverse Proxy

- Terminate TLS at the edge
- Ensure the proxy injects `X-Forwarded-For`
- Firewall the origin to accept traffic only from proxy IP ranges

## 🐙 Forwarded IP Trust

When `SHUMA_FORWARDED_IP_SECRET` is set, the app only trusts `X-Forwarded-For` when the request also includes:

```
X-Shuma-Forwarded-Secret: <same value>
```

This prevents spoofing of client IP headers.

## 🐙 HTTPS Enforcement

Use `SHUMA_ENFORCE_HTTPS=true` in production to hard-reject non-HTTPS traffic.

When enabled:

- Requests without HTTPS context are rejected with `403 HTTPS required`.
- Forwarded proto headers (`X-Forwarded-Proto` / `Forwarded: proto=...`) are trusted only when forwarded-header trust is established with `SHUMA_FORWARDED_IP_SECRET` + `X-Shuma-Forwarded-Secret`.
- If proxy proto forwarding or forwarded-header trust is misconfigured, legitimate requests will fail closed (`403 HTTPS required`).

## 🐙 Fail-Open vs Fail-Closed

`SHUMA_KV_STORE_FAIL_OPEN` controls behavior when the KV store is unavailable during request handling:
- `true` (default): allow requests through and bypass checks
- `false`: return a 500 error and block

This is a **policy decision** and should be explicitly chosen for each deployment.

## 🐙 Admin API Protection

- Generate `SHUMA_API_KEY` with `make api-key-generate` (64-char hex), and rotate with `make api-key-rotate`
- Restrict access with `SHUMA_ADMIN_IP_ALLOWLIST`
- Add CDN/WAF rate limits for `POST /admin/login` and all `/admin/*`
- Keep `SHUMA_ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE` at a conservative value (default `10`)
- Use TLS for all admin traffic
- Keep admin API same-origin from the dashboard; no cross-origin admin API is enabled
- CORS preflight (`OPTIONS`) for `/admin/*` is rejected by design

## 🐙 Health Endpoint Access

`/health` only allows loopback IPs (`127.0.0.1` and `::1`) and trusted forwarded headers.
If your monitoring goes through a proxy, set:
- `X-Forwarded-For: 127.0.0.1`
- `X-Shuma-Forwarded-Secret: <SHUMA_FORWARDED_IP_SECRET>`

For defense in depth, set `SHUMA_HEALTH_SECRET` and require:
- `X-Shuma-Health-Secret: <SHUMA_HEALTH_SECRET>`

## 🐙 Test Mode vs Fail Mode

- Test mode logs actions without enforcing blocks
- Fail mode only applies when the KV store is unavailable

Do not treat test mode as a substitute for fail-open/closed behavior.

## 🐙 Event Retention

Control log retention with `SHUMA_EVENT_LOG_RETENTION_HOURS`:
- Default: 168 hours (7 days)
- Set to `0` to disable cleanup

## 🐙 Metrics Endpoint Exposure

`/metrics` is unauthenticated for Prometheus compatibility. Restrict it at the network edge or via firewall rules if needed.
