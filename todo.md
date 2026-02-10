# TODO

Single source of truth for active project work.

## Security and Platform
- [ ] Design strategy for syncing bans/unbans across global edge instances. (architecture, ops)
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

## Config and Naming Clarity
- [ ] Evaluate renaming `SHUMA_CHALLENGE_RISK_THRESHOLD` to `SHUMA_BOTNESS_CHALLENGE_THRESHOLD` to reflect that it is a botness cutoff, not a parallel risk model. (src/config.rs, docs, dashboard)
- [ ] Standardize terminology across code/UI/docs so `honeypot` and `link maze` are used consistently instead of interchangeably. (src, dashboard, docs)
- [ ] Initialize Ban IP pane duration controls from the current Admin Manual Ban default duration, so Ban IP and Ban Durations panes stay consistent. (dashboard/admin.js, dashboard/index.html, src/admin.rs)
