# Monitoring Prometheus Parity Audit

Date: 2026-02-19
Scope: Dashboard Monitoring tab parity against `/metrics` Prometheus output.

Status values used in this audit:
- `already_exported`: direct equivalent series exists in `/metrics`.
- `derivable_from_existing_series`: can be computed from existing `/metrics` series (possibly with caveats).
- `missing_export`: no equivalent low-cardinality metric series currently exported.

## Parity Matrix

| Monitoring widget/signal | Dashboard source | Parity status | Prometheus mapping | Notes |
| --- | --- | --- | --- | --- |
| Total Bans | `details.analytics.ban_count` / bans list | `derivable_from_existing_series` | `sum(bot_defence_bans_total{reason=...})` | Cumulative counter; not a point-in-time active value. |
| Active Bans | `details.bans.bans` | `already_exported` | `bot_defence_active_bans` | Direct gauge parity. |
| Events (24h) | `details.events.recent_events` | `missing_export` | N/A | Event-feed window count is not represented in `/metrics`. |
| Unique IPs (24h) | `details.events.unique_ips` | `missing_export` | N/A | Would require bounded cardinality strategy or estimator. |
| Event Types (24h) | `details.events.event_counts` | `missing_export` | N/A | No event-type counter family currently exported. |
| Top IPs by Events | `details.events.top_ips` | `missing_export` | N/A | High-cardinality by design; keep summary API only. |
| Events Over Time | `details.events.recent_events` | `missing_export` | N/A | Time-window histogram currently requires event feed. |
| Recent Events table | `details.events.recent_events` | `missing_export` | N/A | Log-level view; should remain API/event-log surface. |
| CDP Total Detections | `details.cdp.stats.total_detections` | `missing_export` | Metric key exists (`cdp_detections_total`) but not rendered in `/metrics` output | Export path gap in `render_metrics`. |
| CDP Auto-Bans | `details.cdp.stats.auto_bans` | `derivable_from_existing_series` | `bot_defence_bans_total{reason="cdp_automation"}` | Equivalent counter exists by ban reason. |
| CDP FP mismatch/flow counts | `details.cdp.fingerprint_stats.*` | `missing_export` | N/A | No Prometheus family for these fingerprint counters. |
| Maze Total Hits | `details.maze.total_hits` | `already_exported` | `bot_defence_maze_hits_total` | Direct parity. |
| Maze Unique Crawlers | `details.maze.unique_crawlers` | `missing_export` | N/A | Would require bounded estimator or heavy labels. |
| Maze Auto-Bans | `details.maze.maze_auto_bans` | `derivable_from_existing_series` | `bot_defence_bans_total{reason="maze_crawler"}` | Ban-reason parity exists. |
| Challenge total/reason breakdown | `summary.challenge.*` | `missing_export` | Partial only: `bot_defence_challenge_incorrect_total`, `bot_defence_challenge_expired_replay_total` | Missing `sequence_violation`, `invalid_output`, `forbidden` in `/metrics`. |
| Challenge trend | `summary.challenge.trend` | `missing_export` | Partial derivation for existing counters only | Full reason-trend parity not possible with current exports. |
| PoW attempts/success/failure/reasons/outcomes | `summary.pow.*` | `missing_export` | N/A | No PoW verify counter family in `/metrics`. |
| PoW trend | `summary.pow.trend` | `missing_export` | N/A | Requires new low-cardinality PoW series. |
| Rate violations/outcomes | `summary.rate.*` | `missing_export` | N/A | Existing rate-limiter outage metrics are different semantics. |
| GEO violations/actions | `summary.geo.*` | `missing_export` | N/A | No GEO enforcement counter family exported yet. |
| GEO top countries | `summary.geo.top_countries` | `missing_export` | N/A | Requires strict country-label guardrails if added. |
| Honeypot total/unique/top-crawlers/top-paths | `summary.honeypot.*` | `missing_export` | N/A | Ban-reason honeypot is not equivalent to honeypot-hit telemetry. |
| External Monitoring helper | `prometheus.*` | `already_exported` | `/metrics` endpoint + examples | Documentation/helper surface already present. |

## Prioritized Add-List (before implementation)

### Priority 1 (low-cardinality, direct operator value)
1. Export `bot_defence_cdp_detections_total` from `render_metrics` (already tracked internally).
2. Add challenge failure series with full reason vocabulary:
   - `bot_defence_monitoring_challenge_failures_total{reason="incorrect|expired_replay|sequence_violation|invalid_output|forbidden"}`.
3. Add PoW verification series:
   - `bot_defence_monitoring_pow_verifications_total{outcome="success|failure"}`.
   - `bot_defence_monitoring_pow_failures_total{reason="invalid_proof|missing_seed_nonce|sequence_violation|expired_replay|binding_timing_mismatch"}`.
4. Add rate-violation outcome series:
   - `bot_defence_monitoring_rate_violations_total{outcome="limited|banned|fallback_allow|fallback_deny"}`.
5. Add GEO action series:
   - `bot_defence_monitoring_geo_violations_total{action="block|challenge|maze"}`.

### Priority 2 (bounded but optional)
1. Add honeypot-hit aggregate series:
   - `bot_defence_monitoring_honeypot_hits_total`.
2. Evaluate GEO country split only if capped to ISO code set + `UNKNOWN` bucket:
   - `bot_defence_monitoring_geo_violations_total{country="..."}`.

### Keep summary API only (do not export as Prometheus labels)
1. Top offenders by IP bucket.
2. Top honeypot paths.
3. Recent events/CDP event rows.

## Implementation Status (2026-02-19)

Priority 1 exports in this audit are now implemented:
- `bot_defence_cdp_detections_total`
- `bot_defence_monitoring_challenge_failures_total{reason=...}`
- `bot_defence_monitoring_pow_verifications_total{outcome=...}`
- `bot_defence_monitoring_pow_failures_total{reason=...}`
- `bot_defence_monitoring_rate_violations_total{outcome=...}`
- `bot_defence_monitoring_geo_violations_total{action=...}`

Regression coverage now includes:
- `/metrics` label-vocabulary guardrails for these families.
- `/metrics` parity assertions against `/admin/monitoring` summary/detail counters.

## Cardinality and Cost Guardrails

1. Never export raw IPs or paths as Prometheus labels.
2. Restrict labels to fixed vocabularies (reason/outcome/action enums).
3. If country labels are added, normalize to ISO-3166 alpha-2 + `UNKNOWN` and cap to known values.
4. Keep `/admin/monitoring` as the high-detail UX contract; use `/metrics` for low-cardinality time series.
5. Reuse existing buffered metric write path; avoid per-request high-fanout writes.
