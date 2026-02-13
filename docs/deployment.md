# 🐙 Deployment & Configuration

Shuma-Gorath is designed to run on Spin (local or cloud). Use the Makefile paths as the official workflow.

Shuma-Gorath is intended to complement enterprise bot defenses (for example Akamai Bot Manager), but can run standalone.

## 🐙 Runtime Configuration Model

- Tunables are loaded from KV (`config:default`) only.
- Env vars are secrets/guardrails only.
- `make setup` seeds KV tunables from `config/defaults.env` using `make config-seed`.
- Runtime config is process-cached for a short TTL (2 seconds) to reduce hot-path KV reads.
- `POST /admin/config` invalidates cache on the handling instance; other instances converge on their TTL window.
- `GET /admin/config/export` provides a non-secret `KEY=value` handoff snapshot for immutable redeploy workflows.

If KV config is missing/invalid at runtime, config-dependent request handling fails with `500 Configuration unavailable`.

## 🐙 Deployment Personas (Provider Scope)

Use one of these operating profiles as your baseline:

| Persona | Who it is for | Provider posture | Edge mode posture | Default recommendation |
| --- | --- | --- | --- | --- |
| `self_hosted_minimal` | Small/self-hosted deployments without managed edge bot tooling | All `provider_backends.*=internal` | `off` | Recommended default for all new installs |
| `enterprise_akamai` (advisory) | Enterprise deployments with Akamai edge/Bot Manager telemetry | Start internal, then selectively enable external per capability after validation | `advisory` | First enterprise cutover stage |
| `enterprise_akamai` (authoritative) | Mature enterprise deployments with validated external adapters and explicit rollback drills | External only for capabilities with proven parity/SLOs | `authoritative` | Optional, advanced stage only |

Current implementation note:

- `fingerprint_signal=external` is currently a stub contract and reports external signal state as unavailable (or disabled when CDP detection is disabled).
- `rate_limiter`, `ban_store`, `challenge_engine`, and `maze_tarpit` currently use explicit unsupported external adapters with safe internal fallback semantics.
- Keep production deployments on internal providers unless you are explicitly exercising a staged integration plan.

## 🐙 Required Env-Only Keys

Set these in your deployment secret/config system:

- `SHUMA_API_KEY`
- `SHUMA_ADMIN_READONLY_API_KEY` (optional; recommended when operators/automation need read-only admin API access)
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

`make deploy-env-validate` enforces:

- `SHUMA_DEBUG_HEADERS=false`
- non-empty and non-overbroad `SHUMA_ADMIN_IP_ALLOWLIST` (rejects wildcard and global-range entries)
- explicit operator attestation that admin edge limits are configured:
  `SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED=true`
- explicit operator attestation that admin API key rotation is complete for the deployment cadence:
  `SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED=true`

### 🐙 Admin Surface Pre-Deploy Checklist

Run this checklist for every production deployment:

1. Admin exposure:
   - Confirm `/admin/*` is reachable only via trusted ingress (CDN/WAF/VPN path), not open origin exposure.
2. Admin allowlist:
   - Confirm `SHUMA_ADMIN_IP_ALLOWLIST` contains only trusted operator/VPN IPs or CIDRs.
   - Confirm no wildcard/global ranges are present.
3. Login and admin edge rate limits:
   - Confirm edge/CDN policy exists for `POST /admin/login` (strict threshold).
   - Confirm edge/CDN policy exists for `/admin/*` (moderate threshold).
   - Set `SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED=true` in deploy-time environment after verification.
4. App-side auth failure limiter:
   - Confirm `SHUMA_ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE` is set to a conservative value for the environment.
5. API key rotation cadence:
   - Rotate `SHUMA_API_KEY` on a regular cadence (recommended 90 days) using `make gen-admin-api-key` / `make api-key-rotate`.
   - Set `SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED=true` in deploy-time environment after rotation verification.

## 🐙 External Provider Rollout & Rollback Runbook

This runbook is required before enabling any external provider in non-dev environments.

### 1. Prerequisites (Do Not Skip)

- Record a baseline while fully internal (`provider_backends.*=internal`, `edge_integration_mode=off`) for at least one representative traffic window.
- Ensure dashboards include:
  - `bot_defence_provider_implementation_effective_total`
  - `bot_defence_botness_signal_state_total`
  - `bot_defence_edge_integration_mode_total`
  - challenge/block rates and p95 latency from your platform metrics.
- Ensure operators can quickly apply config changes (immutable redeploy or controlled `POST /admin/config` workflow).
- Ensure rollback authority and on-call ownership are assigned before cutover.

### 2. Staged Cutover Sequence

1. Internal baseline (required):
   - Keep all providers `internal`.
   - Keep `edge_integration_mode=off`.
   - Confirm stable baseline metrics and normal challenge/block behavior.
2. Advisory stage (first external stage):
   - Enable one capability at a time, beginning with `fingerprint_signal`.
   - Set `edge_integration_mode=advisory`.
   - Keep all other providers internal during this stage.
   - Soak in staging, then production, and confirm expected metrics/outcomes before expanding scope.
3. Authoritative stage (optional):
   - Enter only after advisory stage shows stable behavior and clear operational benefit.
   - Set `edge_integration_mode=authoritative` only for capabilities with explicit authoritative semantics and rollback confidence.
   - Maintain safety-critical local controls and admin protections regardless of edge mode.

### 3. Success Gates Per Stage

- Provider selection gate:
  - `bot_defence_provider_implementation_effective_total` shows expected capability/backend/implementation labels.
- Edge mode gate:
  - `bot_defence_edge_integration_mode_total` confirms requested mode (`off`, `advisory`, `authoritative`).
- Signal health gate:
  - `bot_defence_botness_signal_state_total{state="unavailable"}` does not spike unexpectedly for enabled external signal paths.
- Outcome gate:
  - challenge/block rates remain within expected variance versus baseline.
  - no unexplained increase in user-facing friction or false positives.

### 4. Rollback Triggers

Trigger immediate rollback when any of the following occurs:

- sustained increase in `state="unavailable"` for an enabled external signal provider,
- sudden challenge/block rate jump not explained by traffic or attack context,
- operational instability (timeouts/errors) attributable to external integration,
- operator confidence loss in explainability of decisions/outcomes.

### 5. Rollback Procedure (Immediate)

1. Set affected `provider_backends` capability back to `internal`.
2. Set `edge_integration_mode=off`.
3. Redeploy/reload config via your standard production change path.
4. Verify post-rollback metrics:
   - provider implementation metric returns to `implementation="internal"` for affected capability,
   - edge integration metric reflects `mode="off"`,
   - challenge/block behavior returns toward baseline.
5. Capture incident notes and defer re-enable until root cause and safeguards are documented.

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
