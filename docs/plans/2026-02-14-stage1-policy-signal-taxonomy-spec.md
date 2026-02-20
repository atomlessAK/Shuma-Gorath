# Stage 1 Policy And Signal Taxonomy Spec

Date: 2026-02-14
Status: Draft (implementation-ready)
Scope: Stage 1 prerequisites for adaptive escalation

## Purpose

Define a canonical, implementation-ready escalation taxonomy for Shuma that:

- replaces coarse `allow/monitor/challenge/maze/block` with explicit levels,
- standardizes transition rules between levels,
- anchors where each signal is collected in current request flow,
- keeps `self_hosted_minimal` internal-first while allowing enterprise edge inputs.

This spec is for Stage 1 only. It does not implement Stage 2 Maze or Stage 3 tarpit behavior.

## Canonical Escalation Levels

Use stable IDs so policy, logs, and metrics can reference a single vocabulary.

1. `L0_ALLOW_CLEAN`
   - Normal pass-through.
2. `L1_ALLOW_TAGGED`
   - Allow response, but mark request/session as suspicious for telemetry.
3. `L2_MONITOR`
   - Elevated instrumentation only; no friction.
4. `L3_SHAPE`
   - Low-cost shaping controls (for Stage 1 scope: tighter rate posture and response-surface reduction; no challenge yet).
5. `L4_VERIFY_JS`
   - JS verification gate.
6. `L5_NOT_A_BOT`
   - Low-friction human check (planned path; not yet implemented).
7. `L6_CHALLENGE_STRONG`
   - Strong challenge path (current puzzle challenge and PoW-backed JS flows).
8. `L7_DECEPTION_EXPLICIT`
   - Explicit maze routing (`/maze`, `/trap`, botness/geo maze actions).
9. `L8_DECEPTION_COVERT`
   - Covert decoy injection into normal responses (planned MZ-4).
10. `L9_COST_IMPOSITION`
    - Bounded tarpit/drip behavior (planned Stage 3).
11. `L10_DENY_TEMP`
    - Temporary deny/ban with TTL.
12. `L11_DENY_HARD`
    - Long/indefinite deny posture (policy-controlled, high confidence only).

## Current-Implementation Mapping

This maps current behavior to canonical levels so Stage 1 can be introduced without regression.

- Current baseline pass -> `L0_ALLOW_CLEAN`.
- Botness below challenge threshold but with suspicious signals -> `L1_ALLOW_TAGGED` or `L2_MONITOR` (new classification/logging only in Stage 1).
- JS gate (`js_verification`) -> `L4_VERIFY_JS`.
- Puzzle challenge (`render_challenge`) -> `L6_CHALLENGE_STRONG`.
- Maze route (`geo_policy_maze`, `botness_gate_maze`, direct maze paths) -> `L7_DECEPTION_EXPLICIT`.
- Immediate TTL bans (honeypot/rate/browser/cdp/maze threshold) -> `L10_DENY_TEMP`.
- `L3`, `L5`, `L8`, `L9`, `L11` remain planned until their corresponding work items land.

## Signal Taxonomy (Canonical IDs)

Signal IDs are separate from action levels.

### Request-context signals

- `S_CTX_IP_TRUSTED`: forwarded-header trust validated.
- `S_CTX_PATH_CLASS`: request path class (admin, health, static, app, maze, challenge).
- `S_CTX_UA`: user-agent parsed/outdated classification.

### Pressure and abuse signals

- `S_RATE_USAGE_MEDIUM`: near limit pressure.
- `S_RATE_USAGE_HIGH`: high pressure.
- `S_RATE_LIMIT_HIT`: hard rate enforcement hit.
- `S_HONEYPOT_HIT`: honeypot path hit.

### Geography and policy signals

- `S_GEO_RISK`: geo risk-country scored signal.
- `S_GEO_ROUTE_CHALLENGE`: geo route forces challenge.
- `S_GEO_ROUTE_MAZE`: geo route forces maze.
- `S_GEO_ROUTE_BLOCK`: geo route forces block.

### Browser and automation signals

- `S_JS_REQUIRED_MISSING`: missing JS verification marker.
- `S_BROWSER_OUTDATED`: outdated browser policy hit.
- `S_CDP_REPORT_LOW`: CDP low-tier detection.
- `S_CDP_REPORT_MEDIUM`: CDP medium-tier detection.
- `S_CDP_REPORT_STRONG`: CDP strong-tier detection.

### External fingerprint signals

- `S_FP_EDGE_ADVISORY`: external edge fingerprint advisory event.
- `S_FP_EDGE_STRONG`: external edge fingerprint strong event.
- `S_FP_EDGE_AUTHORITATIVE_BAN`: external strong fingerprint caused authoritative auto-ban.

### Deception signals (current and planned)

- `S_MAZE_TRAVERSAL`: maze traversal observed.
- `S_MAZE_THRESHOLD`: maze threshold reached.
- `S_DECOY_INTERACTION`: covert decoy interaction (planned).
- `S_TARPIT_PERSISTENCE`: persistent tarpit interaction (planned).

### Request-sequence signals (Stage 1 primitives)

- `S_SEQ_OP_MISSING`: expected operation envelope/token is absent.
- `S_SEQ_OP_INVALID`: operation envelope/token fails signature or format validation.
- `S_SEQ_OP_EXPIRED`: operation envelope/token is expired.
- `S_SEQ_OP_REPLAY`: operation ID/token reuse detected.
- `S_SEQ_BINDING_MISMATCH`: operation binding mismatch (`ip_bucket`, `ua_bucket`, or `path_class`).
- `S_SEQ_ORDER_VIOLATION`: request step ordering violated.
- `S_SEQ_WINDOW_EXCEEDED`: ordering/step-gap window exceeded.
- `S_SEQ_TIMING_TOO_FAST`: inter-step timing is implausibly fast.
- `S_SEQ_TIMING_TOO_REGULAR`: cadence is overly regular/automated.
- `S_SEQ_TIMING_TOO_SLOW`: inter-step timing exceeds allowed flow window.

## Detection Taxonomy Additions (Stage 1 Sequence IDs)

Use stable detection IDs paired with sequence signal primitives:

- `D_SEQ_OP_MISSING`
- `D_SEQ_OP_INVALID`
- `D_SEQ_OP_EXPIRED`
- `D_SEQ_OP_REPLAY`
- `D_SEQ_BINDING_MISMATCH`
- `D_SEQ_ORDER_VIOLATION`
- `D_SEQ_WINDOW_EXCEEDED`
- `D_SEQ_TIMING_TOO_FAST`
- `D_SEQ_TIMING_TOO_REGULAR`
- `D_SEQ_TIMING_TOO_SLOW`

## Signed Operation Envelope Primitives (Stage 1)

Challenge/verification tokens should carry these signed envelope fields:

- `operation_id`
- `flow_id`
- `step_id`
- `issued_at`
- `expires_at`
- `token_version`

Current Stage 1 implementation wires this into signed challenge seed paths:

- Puzzle challenge seeds (`flow_id=challenge_puzzle`, `step_id=puzzle_submit`)
- JS PoW verification seeds (`flow_id=js_verification`, `step_id=pow_verify`)

Envelope integrity is validated before puzzle/PoW scoring is evaluated.

### Binding Integrity Primitives (Stage 1)

Signed challenge tokens also carry request-binding primitives:

- `ip_bucket`
- `ua_bucket`
- `path_class`

Current Stage 1 implementation enforces binding checks on puzzle submit and PoW verify paths.
Binding mismatches emit canonical sequence mismatch taxonomy telemetry:

- signal: `S_SEQ_BINDING_MISMATCH`
- detection: `D_SEQ_BINDING_MISMATCH`

### Ordering Window Primitives (Stage 1)

Signed challenge tokens now carry `step_index` and are validated against an expected flow-step tuple:

- Puzzle submit: `flow_id=challenge_puzzle`, `step_id=puzzle_submit`, `step_index=2`
- JS PoW verify: `flow_id=js_verification`, `step_id=pow_verify`, `step_index=2`

Each flow enforces a bounded max-step window in addition to token expiry:

- puzzle submit max step window: `300s`
- JS PoW verify max step window: `300s`

Violations map to canonical sequence telemetry:

- order mismatch (`flow_id`/`step_id`/`step_index` mismatch) -> `S_SEQ_ORDER_VIOLATION` / `D_SEQ_ORDER_VIOLATION`
- stale step window (`min(expires_at, issued_at + max_step_window) < now`) -> `S_SEQ_WINDOW_EXCEEDED` / `D_SEQ_WINDOW_EXCEEDED`

## Transition Rules

Transitions are monotonic within a request unless an explicit de-escalation rule applies in later requests/windows.

### Deterministic hard transitions (must apply)

1. If `S_HONEYPOT_HIT` -> `L10_DENY_TEMP`.
2. If `S_RATE_LIMIT_HIT` -> `L10_DENY_TEMP`.
3. If `S_BROWSER_OUTDATED` -> `L10_DENY_TEMP`.
4. If existing active ban -> `L10_DENY_TEMP`.
5. If GEO route is block -> `L10_DENY_TEMP`.
6. If `S_CDP_REPORT_STRONG` and `cdp_auto_ban=true` -> `L10_DENY_TEMP`.
7. If `S_FP_EDGE_AUTHORITATIVE_BAN` (authoritative mode) -> `L10_DENY_TEMP`.

### Tiered transitions (policy thresholds)

1. Start at `L0_ALLOW_CLEAN`.
2. If suspicious but below friction threshold -> `L1_ALLOW_TAGGED` or `L2_MONITOR`.
3. If shaping threshold reached -> `L3_SHAPE`.
4. If JS gate required -> `L4_VERIFY_JS`.
5. If challenge threshold reached -> `L6_CHALLENGE_STRONG` (or `L5_NOT_A_BOT` when implemented).
6. If maze threshold reached and maze enabled -> `L7_DECEPTION_EXPLICIT`.
7. If covert-decoy policy is enabled and trigger conditions met -> `L8_DECEPTION_COVERT` (planned).
8. If tarpit policy is enabled and high-confidence persistence is observed -> `L9_COST_IMPOSITION` (planned).
9. Repeated high-confidence abuse over policy window may move `L10_DENY_TEMP` -> `L11_DENY_HARD` (planned, guardrailed).

### Precedence rules

Highest-severity eligible level wins per request:

`L11 > L10 > L9 > L8 > L7 > L6 > L5 > L4 > L3 > L2 > L1 > L0`

### De-escalation rules (windowed)

- De-escalation is never immediate inside one request.
- On subsequent windows, reduce one level at a time when no triggering signal recurs.
- Hard deny (`L11`) requires explicit cooldown/ops policy, not automatic same-window decay.

## Signal Collection Map (Current Runtime)

The following maps where signals are collected today.

1. Early context/trust collection (`src/lib.rs`)
   - `extract_client_ip`, `forwarded_ip_trusted`, HTTPS checks.
   - candidate signals: `S_CTX_IP_TRUSTED`, `S_CTX_PATH_CLASS`.
2. Fingerprint report ingestion (`src/lib.rs` + provider registry)
   - route through selected `FingerprintSignalProvider::report_path()`.
   - internal path: `/cdp-report` (`src/providers/internal.rs` -> `src/signals/cdp/mod.rs`).
   - external path: `/fingerprint-report` (`src/providers/external.rs`, Akamai normalization and fallback).
3. Honeypot/rate/ban hard checks (`src/runtime/policy_pipeline.rs`)
   - `maybe_handle_honeypot` -> `S_HONEYPOT_HIT`.
   - `maybe_handle_rate_limit` -> `S_RATE_LIMIT_HIT`.
   - `maybe_handle_existing_ban`.
4. Browser policy check (`src/lib.rs`)
   - `browser::is_outdated_browser` -> `S_BROWSER_OUTDATED`.
5. GEO signal + route (`src/lib.rs`, `src/signals/geo/mod.rs`, `src/runtime/policy_pipeline.rs`)
   - `assess_geo_request` (country extraction + route + scored risk).
   - route actions: block/challenge/maze.
6. Botness signal collection (`src/lib.rs`, `collect_botness_contributions`)
   - JS needed signal (`S_JS_REQUIRED_MISSING`).
   - GEO risk scored signal (`S_GEO_RISK`).
   - Rate pressure signals (`S_RATE_USAGE_MEDIUM`, `S_RATE_USAGE_HIGH`).
7. Botness-driven action selection (`src/runtime/policy_pipeline.rs`)
   - maze threshold -> `L7_DECEPTION_EXPLICIT`.
   - challenge threshold -> `L6_CHALLENGE_STRONG`.
8. JS challenge path (`src/runtime/policy_pipeline.rs`, `src/signals/js_verification.rs`)
   - JS challenge served (`L4_VERIFY_JS`).
   - challenge script posts automation reports to fingerprint endpoint.
9. CDP/fingerprint report processing
   - `src/signals/cdp/mod.rs`: report validation, tiering (`Low/Medium/Strong`), optional auto-ban.
   - `src/providers/external.rs`: Akamai edge normalization, advisory/authoritative behavior, optional auto-ban.
10. Maze tracking (`src/lib.rs`, `serve_maze_with_tracking`)
   - maze hits, threshold checks, optional `maze_crawler` ban.

## Stage 1 Deliverables (Item 1 Implementation Target)

1. Introduce canonical level enum and stable string IDs.
2. Add policy resolver that maps current decisions into canonical levels without changing behavior.
3. Emit level IDs in event outcomes/metrics labels.
4. Add canonical signal IDs in logs/telemetry (using current signal extraction points).
5. Add tests for precedence and transition determinism.

## Non-Goals For Stage 1

- No behavior change to challenge/maze/ban semantics.
- No covert decoy implementation (`L8`) yet.
- No tarpit implementation (`L9`) yet.
- No hard-deny automation (`L11`) yet.

## Open Questions

1. Should `L3_SHAPE` include immediate response-body minimization in Stage 1, or remain telemetry-only until static-resource bypass is complete?
2. Should authoritative edge strong outcomes map directly to `L10_DENY_TEMP` only, or allow direct `L11_DENY_HARD` under explicit enterprise policy?
3. What cooldown windows should govern de-escalation once detection ID taxonomy lands?
