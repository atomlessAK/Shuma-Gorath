What Needs Improving
## Critical Issues
1. [Done] Panic in production code — config.rs:372 calls panic!() on invalid bool config values. Should return Result and surface errors gracefully instead of crashing the WASM module.

2. [Done] Health endpoint IP spoofing — lib.rs:442-444 restricts health to 127.0.0.1/::1 but uses extract_client_ip() which may trust forwarded headers. An attacker behind a proxy could spoof the health check.

3. [DONE] No rate limiting on admin endpoints — Only main traffic is rate-limited. The login endpoint accepts unlimited auth attempts, enabling brute-force attacks on the API key.

4. [Done] Unsanitized ban reasons — lib.rs:881-883 stores raw User-Agent strings as ban reasons without truncation or sanitization, risking KV storage abuse or log injection.

High Priority
5. [Done] Config loaded from KV on every request — lib.rs:596 reads configuration per-request. At scale this becomes a significant KV bottleneck. Needs in-memory caching with TTL.

6. [Done] Browser version parsing is fragile — browser.rs:19 uses split(|c: char| !c.is_digit(10)) which captures only the first numeric segment. Chrome/120.0.1 parses as 120 but edge cases with non-standard UA strings could break detection.

7. [FALSE] Missing SameSite attribute on session cookies — auth.rs:206-213 sets session cookies without SameSite=Strict, leaving the dashboard potentially vulnerable to CSRF in browsers that default to Lax.

8. [ADDED TO TODO] Race condition in rate limiting — rate.rs:23-36 has a TOCTOU gap between reading the counter and writing the increment. Under high concurrency, two requests could both pass the limit check before either increments.

9. [Mostly Done] Silent error suppression — Multiple let _ = patterns (e.g., lib.rs:387-388) discard errors silently. KV write failures go unreported, making production debugging difficult.

## Medium Priority
10. Test mode block is 150+ lines inline — lib.rs:625-774 embeds the entire test-mode logic in the main handler. Should be extracted to a dedicated function/module for maintainability.

11. In-memory metrics lost on crash — metrics.rs buffers metrics in memory before flushing to KV. If the WASM module restarts, buffered data is lost with no recovery mechanism.

12. Ban index scanning is O(n) — ban.rs:71-107 loads the entire ban index into memory on every list_active_bans() call. At scale with thousands of bans, this becomes a bottleneck.

13. No structured logging — All logging uses eprintln!. No log levels, no request IDs, no structured format (JSON). Makes production debugging and monitoring difficult.

14. No admin audit trail — Admin actions (config changes, manual bans/unbans) are not logged with the admin identity or IP. The todos/todo.md notes this as a TODO.

15. Cookie parsing is naive — auth.rs:96-108 uses simple split(';') which could break on cookies with special characters in values.

## Low Priority
16. No API versioning — Endpoints lack a version prefix (e.g., /v1/admin/...), making future breaking changes harder to manage.

17. Same 403 status for different block reasons — Banned, geo-blocked, and rate-limited requests all return 403. Different status codes or response headers would aid debugging.

18. E2E test coverage is thin — Only 6 smoke tests covering basic UI. Missing coverage for CSV export, chart interactivity, pagination, and cross-browser testing.

19. No backup/restore docs for KV — Production deployment docs don't cover KV backup procedures or disaster recovery.

20. Repeated event logging patterns — Multiple files repeat the same EventLog creation pattern with minor variations. A builder or helper would reduce duplication.

## Scores Summary
Category	Score	Rationale
Architecture	9/10	Excellent modular design, defence-in-depth, clean config model
Security	7/10	Strong crypto patterns, but missing admin rate-limiting, audit trail, cookie hardening
Code Quality	7/10	Clean and readable overall, some long functions and silent error patterns
Performance	6/10	Per-request KV reads, O(n) ban scanning, no caching layer
Testing	8/10	126 unit tests + integration + E2E, but missing KV failure and metrics coverage
Documentation	8/10	Comprehensive and well-organized, minor gaps in DR and config bootstrapping
DevEx	9/10	Excellent Makefile, one-command setup, file watching, key management

## Top 5 Recommended Actions
1. Add admin endpoint rate limiting and login attempt throttling — prevents brute-force attacks
2. Cache config in memory with TTL — eliminates per-request KV reads, the biggest performance bottleneck
3. Replace panic!() with Result returns in config validation — prevents production crashes
4. Add structured logging with request IDs — enables production debugging and monitoring
5. Implement admin audit logging — tracks who changed what and when for accountability