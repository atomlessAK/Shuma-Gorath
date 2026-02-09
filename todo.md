# TODO

Single source of truth for active project work.

## Security and Platform
- [ ] Design strategy for syncing bans/unbans across global edge instances. (architecture, ops)
- [ ] Design runtime-agnostic architecture that keeps core detection logic portable while preserving Fermyon-first performance paths. (architecture)
- [ ] Define platform scope boundaries to avoid overreach by leaning on upstream bot managers (for example Akamai) for features better handled there. (product architecture)

## GEO Defense Maturity
- [ ] Add ASN/network dimensions in GEO policy logic (not just country list). (src/geo.rs, src/config.rs, src/admin.rs)
- [ ] Add GEO/ASN observability and alerting (metrics + dashboard panels + docs). (src/metrics.rs, dashboard, docs)

## Config and Naming Clarity
- [ ] Evaluate renaming `SHUMA_CHALLENGE_RISK_THRESHOLD` to `SHUMA_BOTNESS_CHALLENGE_THRESHOLD` to reflect that it is a botness cutoff, not a parallel risk model. (src/config.rs, docs, dashboard)
