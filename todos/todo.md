# TODO

This and `todos/security-review.md` are the single sources of truth for active project work.
## Testing (recurring)
- [ ] review unit tests, integrations tests and the e2e and CI flow all pass and complete
- [ ] identify missing tests and prioritise
- [ ] cleanup defunct tests 

## Data retention
- [ ] reason about current data retention policy, identify any weaknesses or potential sources of maintenance problems due to too much data retained or at the other extreme records wiped too frequently

## Security and Platform
- [ ] Design strategy for syncing bans/unbans across global edge instances. (architecture, ops)
- [ ] Implement and tune CDN/WAF rate limits for `POST /admin/login` and all `/admin/*` as immediate first-layer abuse control. (ops, deployment)
- [ ] Design and implement atomic distributed rate limiting (Redis `INCR`/Lua) for main traffic and admin auth, aligned with global edge-state sync work. (architecture, src/rate.rs, src/auth.rs, spin.toml)
- [ ] Define outage posture for distributed limiter (`fail-open` vs `fail-closed`) and add monitoring/alerts for limiter backend health. (architecture, ops, docs/deployment.md)
- [ ] Design runtime-agnostic architecture that keeps core detection logic portable while preserving Fermyon-first performance paths. (architecture)
- [ ] Define platform scope boundaries to avoid overreach by leaning on upstream bot managers (for example Akamai) for features better handled there. (product architecture)
- [x] Keep dashboard/admin API same-origin by default: infer endpoint from page origin and remove normal endpoint editing; keep only a dev override path. (dashboard/index.html, dashboard/dashboard.js, docs/dashboard.md)
- [x] Keep CORS closed by default (no cross-origin admin API surface); document this deployment/security posture explicitly. (src, docs/security-hardening.md, docs/deployment.md)
- [x] Enforce HTTPS in production deployments (typically at CDN/proxy), with trusted-forwarded-header guidance. (src/lib.rs, dashboard/dashboard.js, docs/deployment.md, docs/security-hardening.md)
- [ ] Add stronger admin controls for production tuning: split read/write privileges and keep audit visibility for write actions. (src/auth.rs, src/admin.rs, dashboard, docs)
- [ ] Long-term possible: integrate upstream identity/proxy auth (OIDC/SAML) for dashboard/admin instead of app-level key login. (platform, docs/deployment.md)
- [x] Add Makefile key generation/rotation helpers using OS CSPRNG tools (no new dependency). (Makefile, docs/deployment.md)
- [ ] Add non-secret runtime config export for deploy handoff (exclude secrets), so dashboard-tuned settings can be applied in immutable redeploys. (src/admin.rs, docs/configuration.md)

## GEO Defense Maturity
- [ ] Add ASN/network dimensions in GEO policy logic (not just country list). (src/geo.rs, src/config.rs, src/admin.rs)
- [ ] Add GEO/ASN observability and alerting (metrics + dashboard panels + docs). (src/metrics.rs, dashboard, docs)

## Challenge
- [ ] Implement tarpit mode (slow-drip responses) with clear routing policy, metrics, and admin/status visibility. (src/lib.rs, src/metrics.rs, dashboard, docs)
- [ ] Implement Challenge Lite (`/challenge/not-a-bot-checkbox`) per `todos/challenge-lite-spec.md` with signed short-lived single-use nonce and IP-bucket binding. (src/challenge.rs, src/lib.rs)
- [ ] Implement Challenge Lite telemetry capture/validation and scoring model (`0..10`) with server-side threshold routing (`pass`, `escalate_puzzle`, `maze_or_block`). (src/challenge.rs, src/lib.rs)
- [ ] Add Challenge Lite verification marker/token issuance after pass and enforce it in routing flow. (src/challenge.rs, src/lib.rs, src/auth.rs)
- [ ] Add Challenge Lite admin visibility/config controls for thresholds, TTL, and attempt caps (read-only defaults + optional mutability controls). (src/admin.rs, dashboard)
- [ ] Add Challenge Lite metrics and dashboard exposure (`served/pass/escalate/fail/replay`, latency). (src/metrics.rs, dashboard)
- [ ] Add unit/integration/e2e coverage for Challenge Lite lifecycle and replay/abuse paths. (src/*_tests.rs, scripts/tests/integration.sh, e2e)
- [ ] Add an accessibility-equivalent challenge modality with the same verification strength (same expiry, single-use, signature, and IP-bucket checks as the visual puzzle). (src/challenge.rs, docs/challenge-verification.md)
- [ ] Add post-success human-verification token issuance and enforcement for protected flows. (src/challenge.rs, src/lib.rs, src/auth.rs, docs)
- [ ] Add challenge operational metrics for abandonment/latency (for example median solve time and incomplete challenge rate). (src/metrics.rs, dashboard, docs/testing.md)

## Config and Naming Clarity
- [ ] Evaluate renaming `SHUMA_CHALLENGE_RISK_THRESHOLD` to `SHUMA_BOTNESS_CHALLENGE_THRESHOLD` to reflect that it is a botness cutoff, not a parallel risk model. (src/config.rs, docs, dashboard)
- [ ] Standardize terminology across code/UI/docs so `honeypot` and `maze` are used consistently instead of interchangeably. (src, dashboard, docs)
- [ ] Initialize Ban IP pane duration controls from the current Admin Manual Ban default duration, so Ban IP and Ban Durations panes stay consistent. (dashboard/admin.js, dashboard/index.html, src/admin.rs)
- [ ] Document setup-time config bootstrapping clearly: how `make setup` creates/populates local env, how env-only vars are sourced, and how KV defaults are seeded and later overridden. (docs/configuration.md, docs/deployment.md, README.md)
