# 🐙 Project Audit TODO

This file tracks the audit findings and remediation tasks discussed on 2026-02-04.

## 🐙 High Priority (Security)
- [x] Move JS challenge HMAC secret to environment (`SHUMA_JS_SECRET`) and document it. (src/js.rs, spin.toml, docs)
- [x] Tighten `/health` access: do not allow "unknown" IP; rely on explicit loopback headers. (src/lib.rs)

## 🐙 Medium Priority (Correctness / Ops)
- [x] Respect `maze_enabled` config before serving maze paths. (src/lib.rs, src/config.rs)
- [x] Align event log paging: reader should scan the same max pages as writer. (src/admin.rs)
- [x] Filter expired bans in `/admin/ban` list and `/admin/analytics` count; clean up expired keys. (src/admin.rs)
- [x] Include `maze_crawler` and `cdp_automation` in metrics output. (src/metrics.rs)
- [x] Handle KV store open failures in admin endpoints without panicking. (src/admin.rs)
- [x] Reduce `get_keys()` scans for bans by maintaining a ban index. (src/ban.rs, src/admin.rs, src/metrics.rs)
- [x] Add event log retention cleanup by hour (configurable). (src/admin.rs, spin.toml, docs)
- [x] Add optional admin IP allowlist (env-driven). (src/auth.rs, src/admin.rs, docs)
- [ ] Design strategy for syncing bans/unbans across global edge instances. (architecture, ops)
- [ ] Design runtime-agnostic architecture that keeps core detection logic portable while preserving Fermyon-first performance paths. (architecture)
- [ ] Define platform scope boundaries to avoid overreach by leaning on upstream bot managers (for example Akamai) for features better handled there. (product architecture)
- [ ] Mature GEO defense model: trust boundary for geo headers, add ASN/network dimensions, endpoint-aware GEO policy tiers, admin policy editing, and geo/ASN observability + alerting. (src/geo.rs, src/lib.rs, src/admin.rs, dashboard, docs)

## 🐙 Low Priority (DX / Docs / Hygiene)
- [x] Fix dashboard time-series request param: use `hours` instead of `limit`. (dashboard/dashboard.js)
- [x] Expose `unique_ips` in `/admin/events` and use it in the dashboard. (src/admin.rs, dashboard/dashboard.js)
- [x] Resolve doc mismatches (Authorization header vs `X-API-Key`; test counts; defunct references). (docs/dashboard.md, QUICK_REFERENCE.md, docs/testing.md)
- [x] Remove/ignore stray artifacts (`unit_test_output.log`), align `.gitignore`. (.gitignore)
- [x] Remove unused `tests/` crate and keep CDN `chartjs-plugin-colorschemes`.

## 🐙 In Progress
- [x] Add trusted proxy gating for forwarded IP headers and document it. (src/lib.rs, README.md, QUICK_REFERENCE.md, spin.toml)
