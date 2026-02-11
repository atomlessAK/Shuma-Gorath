Security Review: KV write abuse and related risks
===============================================

Date: 2026-02-03
Scope: Review of key-value (`Store`) write usage and related security vulnerabilities across the repository. This file captures findings, risk analysis, prioritized mitigations, and suggested next steps so the audit is preserved in-repo.

1) Findings — locations of KV writes (uses of `store.set`)
---------------------------------------------------------
Files & notable usages:
- `src/lib.rs`
  - Health check: `store.set("health:test", b"ok")`
  - Maze hit increment: `store.set("maze_hits:{ip}", (hits+1))`
- `src/rate.rs`
  - Per-IP per-minute counter: `store.set("rate:{site}:{ip}:{window}", count+1)`
- `src/ban.rs`
  - `ban_ip` writes ban entries: `store.set("ban:{site}:{ip}", serialized_entry)`
- `src/admin.rs`
  - `log_event` reads hourly bucket, pushes an event, then `store.set("eventlog:{hour}", vec_of_events)`
  - Config save: `store.set("config:{site}", serialized_config)`
  - Admin flows: some admin actions write keys (ban/unban, config updates)
  - Many admin views call `get_keys()` and enumerate keys
- `src/challenge.rs` & `src/challenge_tests.rs`
  - Challenge-related writes used during challenge handling and tests
- `src/metrics.rs`
  - `metrics::increment` performs `store.set("metric:...", current+1)` per metric increment

2) Risk summary
----------------
Primary risk: Unbounded or inexpensive writes on high-frequency code paths can be abused to:
- Exhaust KV IOPS or cause throttling/latency spikes (denial-of-service via storage pressure)
- Grow keyspace and storage usage (cost/retention issues)
- Create hotspots leading to slow requests and degraded availability

High-risk patterns found:
- Per-request writes on hot paths (`metrics::increment`, per-request maze/log writes).
- Writes keyed directly by unbounded user-controlled values (raw IPs, path segments), leading to high cardinality.
- Appending event logs by reading/modifying an unbounded vector (`log_event`) — risk of very large values and expensive read/modify/write cycles.
- Admin endpoints that call `get_keys()` to enumerate entire keyspace — expensive and potentially blocking when keyspace grows.
- Many `let _ = store.set(...)` calls ignore errors and lack backoff/retry logic.

Secondary issues:
- `spin.toml` initially contained a placeholder `SHUMA_API_KEY` — now noted and commented; sensitive values should be moved to CI secrets.
- `allowed_outbound_hosts = ["http://*", "https://*"]` is permissive and can enable unexpected outbound behavior.
- `X-KV-Status` and `X-Shuma-Fail-Mode` are useful for health checks but leak internal state if exposed in production.

3) Prioritized mitigations (Immediate -> Medium -> Long)
--------------------------------------------------------
Immediate (apply ASAP):
- Reduce per-IP key cardinality via bucketing/hashing.
  - Replace raw per-IP keys with either IP-masked buckets (e.g. IPv4 /24, IPv6 /64) or hashed buckets (e.g. sha256(ip) % N).
  - Apply to `src/rate.rs` (rate counters), maze hit metrics in `src/lib.rs`, and any place where IP is used as key.
- Stop writing to KV on every request for high-frequency counters.
  - Implement in-process ephemeral counters (Map) + periodic flush (e.g., every 5–30s) to KV.
  - Alternatively, batch multiple increments into one write per interval.
- Add write-side backoff when KV returns errors.
  - Track KV error rate and switch to degraded mode (stop writes) for a short cooldown when errors spike.
- Sanitize and validate all key components and enforce value size caps (e.g., max 4 KB) before persisting.

High / Medium (next):
- Replace `log_event` vector append with paged/append-only buckets and per-page size limits.
  - Use `eventlog:{hour}:{page}` pages with a capped number of entries per page.
  - Implement retention (delete buckets older than retention period) via background cleanup or on-write housekeeping.
- Avoid `get_keys()` full scans. Maintain secondary indexes for frequently-listed sets (e.g., `ban_index:{site}`) that are updated on ban/unban operations.
- Add rate-limiting and auth hardening for admin endpoints. Consider IP allowlist for admin UI/API and stronger auth flows.
- Restrict `allowed_outbound_hosts` to minimum required hosts.

Longer-term / Hardening:
- Add monitoring and alerting for unusual increases in unique key creation, KV error rates, and value sizes.
- Integrate a proper metrics backend (Prometheus push/remote) rather than relying solely on KV for high-frequency counters.
- Add automatic retention/compaction tasks for event logs and large metrics/keys.

4) Per-file actionable suggestions
---------------------------------
- `src/rate.rs` (rate limiting)
  - Replace `let key = format!("rate:{}:{}", site_id, ip)` with a bucketed key:
    - Example: `let bucket = ip_bucket(ip); let key = format!("rate:{}:{}:{}", site_id, bucket, window);`
    - `ip_bucket(ip)` can mask IPv4 last octet or hash to fixed N buckets.
  - Add a cap on distinct buckets per site per window; if cap exceeded, treat as high-traffic attack and fail-safe (e.g., degrade to stricter blocking or sample).

- `src/lib.rs` (maze hits, metrics)
  - Use `ip_bucket` when storing `maze_hits:{ip}`.
  - For `metrics::increment`, replace per-request writes with an in-memory counter map and periodic flush. Alternatively, sample metrics when under attack.
  - In auto-ban paths, ensure actions (ban + logs) are rate-limited per-IP to avoid cascading writes.

- `src/admin.rs` (event logs, config)
  - Rework `log_event` to use paged buckets: `eventlog:{hour}:{page}` with max entries/page.
  - Enforce max length and cap events stored per bucket; drop or sample lower-priority events when buckets full.
  - Replace `get_keys()` scans for listing bans with a maintained index `ban_index:{site}` (bounded) that is updated at ban/unban time.

- `src/metrics.rs`
  - Implement local counters and flush to KV periodically; reduce per-request KV writes.

- `src/ban.rs`
  - Validate IP formats before using them in keys; consider storing ban indices separately to avoid scanning keys.

5) Concrete code patterns and examples
------------------------------------
- IP bucketing (concept):
  - IPv4: `1.2.3.45` -> bucket `1.2.3.0` (mask last octet) => caps distinct keys per /24
  - Hashing: `let bucket = format!("b{}", (sha256(ip) % 1024));` then use `rate:{site}:{bucket}:{window}`
- Buffered metrics (concept):
  - Use a `HashMap<String, u64>` in memory; on each request increment the map; periodically (timer or request count) write deltas to KV and reset the map.
- Paged hourly event logs:
  - `eventlog:{hour}:1`, `eventlog:{hour}:2` ... Each page stores up to N events (e.g., 1000). When full, create next page.
  - On reads, fetch latest pages only.

6) Operational recommendations
------------------------------
- Add load tests that simulate large numbers of unique IPs to validate KV behavior under attack conditions.
- Add alerts for rapid growth in `get_keys()` results, high KV error rates, and unusual value sizes.
- Move secrets (SHUMA_API_KEY) into CI secret store and do not commit them into `spin.toml`.

7) Immediate next steps I can implement (pick one or more)
---------------------------------------------------------
- Implement IP bucketing in `src/rate.rs` and `src/lib.rs` (maze hits). (Recommended first step)
- Implement buffered metrics in `src/metrics.rs` (moderate effort).
- Rework `log_event` to paged buckets with capped size.
- Replace `get_keys()` scans in admin APIs with indexed lists.
- Remove placeholder `SHUMA_API_KEY` from `spin.toml` and add CI secret docs (if desired).

Appendix: original quick notes from the audit
-------------------------------------------
(Kept here verbatim for traceability)
- Primary risk: unbounded/cheap writes on hot paths let attackers exhaust KV IOPS/space or cause throttling/latency (DDoS).
- High-risk patterns: per-request writes, keys using user-controlled strings, get_keys() enumerations, log_event unbounded arrays.
- Mitigation highlights: IP bucketing/hashing, buffering metrics, paged event logs, retention & cleanup, restrict allowed outbound hosts, move secrets to CI.


Prepared by: Automated code review assistant (saved to repo)


-- End of SECURITY_REVIEW.md
