# 🐙 Security Hardening

This guide focuses on deployment security controls and operational hardening.

## 🐙 Run Behind a CDN / Reverse Proxy

- Terminate TLS at the edge
- Ensure the proxy injects `X-Forwarded-For`
- Firewall the origin to accept traffic only from proxy IP ranges

## 🐙 Forwarded IP Trust

When `FORWARDED_IP_SECRET` is set, the app only trusts `X-Forwarded-For` when the request also includes:

```
X-Shuma-Forwarded-Secret: <same value>
```

This prevents spoofing of client IP headers.

## 🐙 Fail-Open vs Fail-Closed

`SHUMA_FAIL_MODE` controls behavior when the KV store is unavailable during request handling:
- `open` (default): allow requests through and bypass checks
- `closed`: return a 500 error and block

This is a **policy decision** and should be explicitly chosen for each deployment.

## 🐙 Admin API Protection

- Rotate `API_KEY` regularly
- Restrict access with `ADMIN_IP_ALLOWLIST`
- Limit `/admin/*` access at the CDN or firewall
- Use TLS for all admin traffic

## 🐙 Health Endpoint Access

`/health` only allows loopback IPs (`127.0.0.1` and `::1`) and trusted forwarded headers.
If your monitoring goes through a proxy, set:
- `X-Forwarded-For: 127.0.0.1`
- `X-Shuma-Forwarded-Secret: <FORWARDED_IP_SECRET>`

## 🐙 Test Mode vs Fail Mode

- Test mode logs actions without enforcing blocks
- Fail mode only applies when the KV store is unavailable

Do not treat test mode as a substitute for fail-open/closed behavior.

## 🐙 Event Retention

Control log retention with `EVENT_LOG_RETENTION_HOURS`:
- Default: 168 hours (7 days)
- Set to `0` to disable cleanup

## 🐙 Metrics Endpoint Exposure

`/metrics` is unauthenticated for Prometheus compatibility. Restrict it at the network edge or via firewall rules if needed.
