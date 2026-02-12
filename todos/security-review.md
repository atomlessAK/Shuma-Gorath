# Security Review Tracker

Last updated: 2026-02-12

Purpose: track review finding validity and closure status.
Active implementation planning lives in `todos/todo.md`.

## Open Findings (Actionable)

### P1
- [ ] Rate-limiter TOCTOU remains for high concurrency and multi-instance scenarios; fix requires atomic distributed counters (Redis `INCR`/Lua path tracked in `todos/todo.md`).
- [ ] Admin abuse controls are partially operational and must be enforced in deployment: `SHUMA_ADMIN_IP_ALLOWLIST` plus CDN/WAF limits for `POST /admin/login` and `/admin/*`.
- [ ] Ban/unban propagation is not yet synchronized across edge instances; consistency drift remains possible under scale/failover.

### P2
- [ ] KV-backed operational telemetry remains acceptable at current scale but needs periodic reassessment against write volume and retention growth.
- [ ] Logging is still largely unstructured (`eprintln!`-first); request correlation and incident triage ergonomics can be improved.

## Closed or Invalid Findings (Audit Trail)
- [x] Event-log append race fixed (`85bff68`).
- [x] Panic on invalid bool env parsing fixed (`69603c5`).
- [x] Health endpoint spoofing risk hardened with strict trust gate plus optional secret (`163e0dc`).
- [x] Admin login brute-force gap fixed in-app (`add999d`), with deployment-layer guidance added (`40e120c`).
- [x] Unsanitized ban reason storage fixed with sanitization/truncation and dashboard escaping (`4b65e49`).
- [x] Per-request runtime config KV reads fixed with in-memory TTL cache (`09e0017`, docs `88155ab`).
- [x] Browser version parsing robustness improved for edge cases (`b44eeca`).
- [x] "Missing SameSite cookie" report assessed as false positive in current implementation.
- [x] Silent KV error suppression significantly reduced by logging critical write/delete failures (`393e0b1`); low-impact cases remain opportunistic cleanup.
