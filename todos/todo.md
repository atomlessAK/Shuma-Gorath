# TODO Roadmap

Last updated: 2026-02-12

This is the active work queue.
`todos/security-review.md` tracks security finding validity and closure status.

## Direction Snapshot (for next implementation stages)
- [ ] Evolve maze behavior toward Cloudflare-style selective covert decoys for suspicious traffic while keeping explicit `/maze` and `/trap` endpoints as fallback and test surface.
- [ ] Build Shuma-native bounded slow-drip tarpit behavior in Rust/Spin; treat external projects (for example Finch/Sarracenia/caddy-defender) as design references, not runtime dependencies.
- [ ] Strengthen fingerprinting by ingesting trusted upstream edge signals (JA3/JA4 and similar) and integrating them into scoring/policy.
- [ ] Refactor to clearer in-repo modules first; defer multi-repo splits until module boundaries and interfaces are stable.

## Modularization Sprint (Active)
- [x] Define sprint guardrails: refactor-only, no behavior changes, no new dependencies, tests must pass before each checkoff.
- [x] M1 Extract inline test-mode block from `src/lib.rs` into dedicated `src/test_mode.rs`.
- [x] M2 Add focused unit tests for extracted test-mode behavior (cover bypass, block, and allow outcomes).
- [x] M3 Keep `src/lib.rs` behavior identical by routing existing test-mode flow through the new module.
- [x] M4 Run verification (`cargo test` and integration smoke path) and record result.
- [x] M5 Plan and execute next extraction slice from `src/lib.rs` (routing/decision helpers) with similarly scoped checklist items.
- [x] M5.1 Extract early endpoint routing (`/health`, `/admin`, `/metrics`, `/robots.txt`, challenge endpoints) into a dedicated router helper/module without changing semantics.
- [x] M5.2 Extract KV outage/open-close gate into a dedicated helper to isolate fail-open/fail-closed behavior.
- [x] M5.3 Extract post-config enforcement pipeline ordering into named helpers (honeypot, rate, ban, geo policy, botness, JS).
- [x] M5.4 Add regression tests for routing order and short-circuit precedence after extraction.
- [x] M6 Dashboard decomposition track (`dashboard/dashboard.js` split into domain modules under `dashboard/modules/`).
- [x] M6.1 Extract charts/timeseries logic into `dashboard/modules/charts.js` and wire via a stable API surface.
- [x] M6.2 Extract status panel state/rendering into `dashboard/modules/status.js` and remove status-specific globals from root script.
- [x] M6.3 Extract config/tuning save handlers into `dashboard/modules/config-controls.js`.
- [x] M6.4 Extract admin actions/session endpoint wiring into `dashboard/modules/admin-session.js`.
- [x] M6.5 Add frontend smoke checks for key interactions (login/session restore, chart refresh, config save buttons enabled/disabled state).
- [x] M7 Maze domain decomposition track (split `src/maze.rs` into `src/maze/` submodules: generation, routing hooks, telemetry, templates).
- [x] M7.1 Convert `src/maze.rs` into `src/maze/mod.rs` with identical public API (`is_maze_path`, `handle_maze_request`, `generate_maze_page`, `MazeConfig`).
- [x] M7.2 Extract deterministic generation primitives (`SeededRng`, path seeding, content generators) into focused submodules.
- [x] M7.3 Extract HTML rendering/template assembly into a dedicated maze template module.
- [x] M7.4 Isolate maze request/path helpers so later telemetry/routing-hook extraction from `src/lib.rs` has a stable seam.
- [x] M7.5 Keep/extend maze tests after extraction and run verification (`cargo test`, integration smoke).
- [x] M8 Challenge domain decomposition track (split `src/challenge.rs` into `src/challenge/` submodules: token/crypto, puzzle generation, HTTP handlers, validation/anti-replay).
- [x] M8.1 Convert `src/challenge.rs` into `src/challenge/mod.rs` while preserving public API used by `src/lib.rs` and tests.
- [x] M8.2 Extract seed token/HMAC logic into a dedicated `src/challenge/token.rs` module.
- [x] M8.3 Extract puzzle generation/transform/validation logic into `src/challenge/puzzle.rs`.
- [x] M8.4 Extract rendering and submit/anti-replay flow into focused HTTP modules (`src/challenge/render.rs`, `src/challenge/submit.rs`).
- [x] M8.5 Run verification (`cargo test`) to confirm no behavior change.
- [x] M9 Directory structure prep for future repo boundaries (core policy, maze+tarpit, challenge, dashboard adapter) with explicit interface contracts.
- [x] M9.1 Add explicit Rust boundary contracts for challenge/maze/admin in `src/boundaries/contracts.rs`.
- [x] M9.2 Add default adapter implementations in `src/boundaries/adapters.rs` and route `src/lib.rs` through `src/boundaries/`.
- [x] M9.3 Document boundary rules and target split direction in `docs/module-boundaries.md`.

## P0 Immediate (ops and abuse containment)
- [ ] Enforce `SHUMA_ADMIN_IP_ALLOWLIST` in every production environment.
- [ ] Configure CDN/WAF rate limits for `POST /admin/login` and all `/admin/*` in every deployment (Cloudflare and Akamai guidance already documented).
- [ ] Rotate `SHUMA_API_KEY` using `make gen-admin-api-key` and set a regular rotation cadence.
- [ ] Add deployment runbook checks for admin exposure, allowlist status, and login rate-limit posture.
- [ ] Add stronger admin controls for production tuning: split read/write privileges and keep audit visibility for write actions. (`src/auth.rs`, `src/admin.rs`, dashboard, docs)

## P1 Distributed State and Limiter Correctness
- [ ] Design and implement atomic distributed rate limiting (Redis `INCR`/Lua) for main traffic and admin auth, aligned with edge-state sync work. (`src/rate.rs`, `src/auth.rs`, `spin.toml`)
- [ ] Define outage posture for distributed limiter (`fail-open` vs `fail-closed`) and add monitoring/alerts for limiter backend health. (architecture, ops, `docs/deployment.md`)
- [ ] Design strategy for syncing bans/unbans across global edge instances. (architecture, ops)
- [ ] Add monitoring for limiter fallback usage, sync lag, and distributed state drift.

## P1 Staged Adaptive Defence (maze to slow-drip)

### Stage 1: Policy and signal prerequisites
- [ ] Add stable detection ID taxonomy and policy matching (`allow`, `monitor`, `challenge`, `maze`, `block`).
- [ ] Add static-resource bypass defaults to avoid expensive bot checks on obvious static assets.
- [ ] Add request-sequence signal primitives (operation IDs, ordering windows, timing thresholds).
- [ ] Add AI-bot policy controls as first-class admin config (separate from robots-only controls).

### Stage 2: Maze evolution (Cloudflare-inspired, Shuma-native)
- [ ] Inject covert decoy links into eligible HTML responses for medium-confidence suspicious traffic.
- [ ] Keep decoys invisible to normal users and compliant crawlers; avoid UX/SEO regressions.
- [ ] Increase maze entropy (template diversity, fake static assets, path diversity) to reduce fingerprintability.
- [ ] Feed maze interaction behavior back into botness scoring and detection IDs.

### Stage 3: Bounded slow-drip tarpit
- [ ] Implement `maze_plus_drip` mode with configurable byte rate and hard timeout.
- [ ] Enforce strict tarpit budgets (global concurrent streams and per-IP-bucket caps).
- [ ] Add deterministic fallback action when tarpit budget is exhausted (`maze` or `block`).
- [ ] Add tarpit metrics/admin visibility for activation, saturation, duration, bytes sent, and escalation outcomes.

### Stage 4: Escalation and distributed hardening
- [ ] Escalate persistent tarpit clients to ban/block with guardrails to minimize false positives.
- [ ] Integrate tarpit budgets/counters with distributed state work for multi-instance consistency.

## P1 Fingerprint Strengthening
- [ ] Add trusted-header ingestion for transport fingerprints supplied by CDN/proxy.
- [ ] Normalize fingerprint signals with provenance/confidence metadata for rule evaluation.
- [ ] Add mismatch heuristics (for example UA/client-hint versus transport fingerprint anomalies).
- [ ] Add fingerprint-centric admin visibility for investigations and tuning.
- [ ] Run a Finch compatibility spike as an optional upstream sidecar experiment and document tradeoffs for Shuma (no direct dependency in core runtime).

## P1 IP Range Policy Controls
- [ ] Add CIDR/IP-range policy evaluation to block, challenge, maze, or otherwise handle requests from configured ranges.
- [ ] Ship managed built-in CIDR sets for major AI service traffic (for example OpenAI, DeepSeek, GitHub Copilot) with explicit update/versioning process.
- [ ] Add operator-defined custom CIDR lists in config/admin with strict validation and clear precedence against managed sets.
- [ ] Extend response-action execution to support: `403_forbidden`, custom-message response, connection drop, `308` redirect (custom URL), `rate_limit`, `honeypot`, `maze`, and `tarpit`.
- [ ] Document operational guidance for IP-range policy safety (false-positive controls, dry-run/test mode, observability, rollback).

## P2 Challenge Roadmap
- [ ] Implement Challenge Lite (`/challenge/not-a-bot-checkbox`) per `todos/challenge-lite-spec.md` with signed short-lived single-use nonce and IP-bucket binding.
- [ ] Implement Challenge Lite telemetry capture/validation and scoring model (`0..10`) with server-side threshold routing (`pass`, `escalate_puzzle`, `maze_or_block`).
- [ ] Add Challenge Lite verification marker/token issuance after pass and enforce it in routing flow.
- [ ] Add Challenge Lite admin visibility/config controls for thresholds, TTL, and attempt caps (read-only defaults plus optional mutability controls).
- [ ] Add Challenge Lite metrics and dashboard exposure (`served`, `pass`, `escalate`, `fail`, `replay`, latency).
- [ ] Add unit/integration/e2e coverage for Challenge Lite lifecycle and replay/abuse paths.
- [ ] Add an accessibility-equivalent challenge modality with equivalent verification strength (expiry, single-use, signature, IP-bucket checks).
- [ ] Add post-success human-verification token issuance and enforcement for protected flows.
- [ ] Add challenge operational metrics for abandonment/latency (for example median solve time and incomplete challenge rate).

## P2 GEO Defence Maturity
- [ ] Add ASN/network dimensions in GEO policy logic (not just country list). (`src/geo.rs`, `src/config.rs`, `src/admin.rs`)
- [ ] Add GEO/ASN observability and alerting (metrics, dashboard panels, docs). (`src/metrics.rs`, dashboard, docs)

## P2 Modularization and Future Repository Boundaries
- [ ] Restructure source into clearer domain modules (policy engine, maze/tarpit, challenges, fingerprint signals, admin adapters).
- [ ] Extract policy decision flow from HTTP plumbing to enable isolated testing and future reuse.
- [ ] Define module interface contracts and dependency direction (core domain first, adapters second).
- [ ] Write objective criteria for future repo splits (API stability, release cadence, ownership, operational coupling).

## P2 Repository and Architecture Hardening (Structure + Pluggability)

### H1 Artifact and workspace hygiene
- [x] Keep all generated build artifacts out of `src/` (including WASM binaries) and move them to a dedicated artifacts path (for example `dist/wasm/`).
- [x] Update `spin.toml`, Makefile targets, and bootstrap scripts to consume the new artifacts path without changing runtime behavior.
- [x] Keep Playwright and test outputs ephemeral (`playwright-report/`, `test-results/`) and confirm ignore rules remain correct after any directory changes.
- [x] Add a short doc section describing expected generated directories and what should never be committed.

### H2 Test layout modernization (Rust idiomatic split)
- [x] Define and document project test conventions:
  unit tests colocated with module code,
  integration/behavior tests in `tests/`.
- [x] Create a shared test support module (request builders, env guards, common fixtures) to reduce duplication across current `src/*_tests.rs`.
- [x] Incrementally migrate top-level `src/*_tests.rs` files into colocated module tests and/or `tests/` integration suites (no behavior changes).
- [x] Keep test discovery and CI commands stable (`cargo test`, Make targets) throughout migration.
- [x] Add/adjust regression tests to ensure routing and enforcement order remain stable while tests move (runtime-backed early routes should be covered in integration-level tests, not native unit tests).
- [x] H2 slice A completed: moved ban/CDP/GEO/request-router/test-mode/whitelist test files to module-local paths and removed corresponding top-level test module wiring in `src/lib.rs`.
- [x] H2 slice A completed: added shared unit-test helpers in `src/test_support.rs` and adopted them in env-sensitive suites.
- [x] H2 slice A verification: `cargo test` passes after migration with no behavior changes.
- [x] H2 slice B completed: migrated config tests from `src/config_tests.rs` to module-local `src/config/tests.rs`, including shared env-lock adoption.
- [x] H2 slice C completed: migrated challenge tests from `src/challenge_tests.rs` to module-local `src/challenge/tests.rs`.
- [x] H2 slice B/C verification: `cargo test` passes with module-local config/challenge suites.
- [x] H2 slice D completed: migrated remaining crate-level test files (`risk`, `security`, `logging`) into structured `src/lib_tests/` modules and removed legacy top-level `src/*_tests.rs`.
- [x] H2 slice D verification: `cargo test` passes with stable test discovery after `src/lib_tests/` adoption.

### H3 Domain directory deepening (beyond first modularization pass)
- [x] Move orchestration helpers (`request_router`, `kv_gate`, `policy_pipeline`) into a cohesive runtime/policy directory with clear ownership boundaries.
- [x] Group admin/auth/config concerns into a cohesive adapter/domain boundary layout with minimal cross-module leakage.
- [x] Group signal and enforcement modules by domain (for example risk signals, enforcement actions, challenge/maze) and reduce root-level file sprawl.
- [x] Add thin compatibility re-exports during moves so refactors remain reviewable and low-risk.
- [x] Remove temporary compatibility shims once imports are fully migrated.
- [x] H3.1 slice completed: moved request orchestration modules into `src/runtime/` (`runtime/request_router.rs`, `runtime/kv_gate.rs`, `runtime/policy_pipeline.rs`) and rewired `src/lib.rs` call sites without behavior changes.
- [x] H3.2 slice completed: moved admin/auth into `src/admin/` (`admin/mod.rs`, `admin/api.rs`, `admin/auth.rs`) and moved config into `src/config/mod.rs`, then rewired module imports with no behavior change.
- [x] H3.3/H3.4 slice completed: regrouped signal modules under `src/signals/` and enforcement modules under `src/enforcement/`, then added crate-level compatibility re-exports in `src/lib.rs` to keep call sites stable during the move.
- [x] H3.5 slice completed: migrated remaining call sites to `src/signals/*` and `src/enforcement/*` paths and removed temporary compatibility re-exports from `src/lib.rs`.

### H3.6 Composable Defence + Signal Foundation (internal-first)
- [x] Define and document the defence taxonomy with an explicit inventory of `signal`, `barrier`, and `hybrid` modules (for example `rate` as hybrid); include ownership and dependency direction. (`docs/module-boundaries.md`, Defence Taxonomy section)
- [x] Introduce a canonical per-request signal contract (for example `BotSignal` + `SignalAccumulator`) that every signal/hybrid module writes to.
- [x] Add explicit signal availability semantics (`active`, `disabled`, `unavailable`) so botness logic never treats missing modules as silent zero.
- [ ] Split hybrid modules into distinct paths:
  rate telemetry signal contribution for scoring,
  hard rate-limit enforcement barrier for immediate protection.
- [ ] Add composability modes for eligible modules (`off`, `signal`, `enforce`, `both`) while keeping safety-critical controls non-disableable.
- [ ] Define clear behavior for each mode in config/admin surfaces and runtime flow (including invalid combinations and defaults).
- [ ] Refactor botness scoring to consume normalized accumulator output rather than direct module internals.
- [ ] Preserve existing behavior as default mode mapping (no behavior change when operators do not opt in to new modes).
- [ ] Add unit and integration regression tests for mode matrix behavior and ordering invariants (especially hybrid modules and early-route interactions).
- [ ] Add observability for mode and signal-state visibility (metrics/log fields indicating enabled/disabled/unavailable contributors).
- [ ] Update docs (`configuration`, `features`, `observability`, `module-boundaries`) to explain composability semantics and tuning implications.
- [ ] Keep implementations internal-only for now; defer external provider registry/factory work until signal contract and mode semantics stabilize.
- [x] H3.6.1 slice completed: added explicit defence taxonomy + inventory (`signal`, `barrier`, `hybrid`) with ownership and dependency direction in `docs/module-boundaries.md`.
- [x] H3.6.2 slice completed: introduced `BotSignal`/`SignalAccumulator` in `src/signals/botness.rs` and rewired JS, GEO, and rate-pressure botness scoring paths in `src/lib.rs` to emit normalized signal contributions with no behavior change.
- [x] H3.6.3 slice completed: added explicit signal availability states (`active`, `disabled`, `unavailable`) across JS/GEO/rate signal emitters and botness assessment flow, with regression tests for non-silent disabled/unavailable handling.

### H4 Pluggable provider architecture (internal by default, external-capable)
- [ ] Define provider traits for swappable capabilities:
  rate limiting,
  ban store/sync,
  challenge engine,
  maze/tarpit serving,
  fingerprint signal source.
- [ ] Add a provider registry/factory that selects implementations from config (compile-time/runtime config, no behavior change by default).
- [ ] Implement `Internal*` providers matching current behavior as the default path.
- [ ] Add explicit `External*` adapter stubs/contracts (for example Redis limiter, upstream fingerprint feed) with clear unsupported-path handling.
- [ ] Add contract tests that every provider implementation must pass to guarantee semantic parity.
- [ ] Add observability tags/metrics identifying active provider implementation per capability.
- [ ] Document provider selection, rollout, and rollback procedures in deployment docs.

### H5 Execution and rollout discipline
- [ ] Execute this hardening work as small, test-backed slices (one boundary family at a time) to avoid broad regressions.
- [ ] Require each structural slice to pass full verification (`cargo test`, integration smoke, dashboard smoke where relevant) before merge.
- [ ] Track and enforce “no net behavior change” for refactor-only slices unless explicitly scoped otherwise.
- [ ] Define a cutover checklist for enabling any external provider in non-dev environments (staging soak, SLOs, rollback trigger).

## P3 Platform and Configuration Clarity
- [ ] Design runtime-agnostic architecture that keeps core detection logic portable while preserving Fermyon-first performance paths.
- [ ] Define platform scope boundaries to avoid overreach by leaning on upstream bot managers (for example Akamai) for features better handled there.
- [ ] Add non-secret runtime config export for deploy handoff (exclude secrets) so dashboard-tuned settings can be applied in immutable redeploys.
- [ ] Evaluate renaming `SHUMA_CHALLENGE_RISK_THRESHOLD` to `SHUMA_BOTNESS_CHALLENGE_THRESHOLD` to reflect botness semantics.
- [ ] Standardize terminology across code/UI/docs so `honeypot` and `maze` are used consistently instead of interchangeably.
- [ ] Initialize Ban IP pane duration controls from the current Admin Manual Ban default duration so Ban IP and Ban Durations panes stay consistent.
- [ ] Document setup-time config bootstrapping clearly: how `make setup` creates/populates local env, how env-only vars are sourced, and how KV defaults are seeded and later overridden.
- [ ] Long-term option: integrate upstream identity/proxy auth (OIDC/SAML) for dashboard/admin instead of app-level key login.

## Recurring Quality Gates
- [ ] Keep unit, integration, e2e, and CI flows passing; clean up defunct tests quickly.
- [ ] Identify and prioritize missing tests for new defence stages before implementation.
- [ ] Reassess data retention policy as event and metrics volume grows.
