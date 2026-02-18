# Completed TODO History

Moved from active TODO files on 2026-02-14.

## todos/todo.md

- [x] Define sprint guardrails: refactor-only, no behavior changes, no new dependencies, tests must pass before each checkoff.
- [x] M1 Extract inline test-mode block from `src/lib.rs` into dedicated test-mode module (`src/runtime/test_mode/mod.rs`).
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
- [x] M8.2 Extract seed token/HMAC logic into a dedicated `src/challenge/puzzle/token.rs` module.
- [x] M8.3 Extract puzzle generation/transform/validation logic into `src/challenge/puzzle/mod.rs`.
- [x] M8.4 Extract rendering and submit/anti-replay flow into focused HTTP modules (`src/challenge/puzzle/renders.rs`, `src/challenge/puzzle/submit.rs`).
- [x] M8.5 Run verification (`cargo test`) to confirm no behavior change.
- [x] M9 Directory structure prep for future repo boundaries (core policy, maze+tarpit, challenge, dashboard adapter) with explicit interface contracts.
- [x] M9.1 Add explicit Rust boundary contracts for challenge/maze/admin in `src/boundaries/contracts.rs`.
- [x] M9.2 Add default adapter implementations in `src/boundaries/adapters.rs` and route `src/lib.rs` through `src/boundaries/`.
- [x] M9.3 Document boundary rules and target split direction in `docs/module-boundaries.md`.

- [x] Enforce `SHUMA_ADMIN_IP_ALLOWLIST` in every production environment.
- [x] Configure CDN/WAF rate limits for `POST /admin/login` and all `/admin/*` in every deployment (Cloudflare and Akamai guidance already documented).
- [x] Rotate `SHUMA_API_KEY` using `make gen-admin-api-key` and set a regular rotation cadence.
- [x] Add deployment runbook checks for admin exposure, allowlist status, and login rate-limit posture.
- [x] Add stronger admin controls for production tuning: split read/write privileges and keep audit visibility for write actions. (`src/auth.rs`, `src/admin.rs`, dashboard, docs)
- [x] P0.1 slice completed: hardened `make deploy-env-validate` to require non-empty/non-overbroad `SHUMA_ADMIN_IP_ALLOWLIST` and expanded deployment runbook checklist coverage for admin exposure, allowlist status, and login rate-limit posture.
- [x] P0.2 slice completed: added deploy-time edge-rate-limit attestation guard (`SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED=true`) so production deploys fail until `/admin/login` and `/admin/*` CDN/WAF rate limits are explicitly confirmed.
- [x] P0.3 slice completed: added optional read-only admin bearer key (`SHUMA_ADMIN_READONLY_API_KEY`), enforced write-access checks on mutating admin routes, hardened `/admin/unban` to POST-only, and logged denied write attempts for audit visibility.
- [x] P0.4 slice completed: added `gen-admin-api-key` alias + deploy-time API key rotation attestation guard (`SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED=true`) and documented a recommended 90-day rotation cadence in deployment/security runbooks.

- [x] (Enterprise/hybrid track; non-blocking for `self_hosted_minimal`) Formalize profile-gated state-plane architecture: shared policy engine across personas, swappable state backends by profile (`self_hosted_minimal` vs `enterprise_akamai`), and no persona-specific policy fork.
- [x] P1.0 slice completed: documented the profile-gated state-plane architecture as ADR `docs/adr/0001-profile-gated-state-plane.md` and synchronized policy/deployment/config docs.
- [x] (Enterprise/hybrid track; non-blocking for `self_hosted_minimal`) Design and implement atomic distributed rate limiting (Redis `INCR`/Lua) for main traffic and admin auth, aligned with edge-state sync work. (`src/rate.rs`, `src/auth.rs`, `spin.toml`)
- [x] (Enterprise/hybrid track; non-blocking for `self_hosted_minimal`) Define outage posture for distributed limiter (`fail-open` vs `fail-closed`) and add monitoring/alerts for limiter backend health. (architecture, ops, `docs/deployment.md`)
- [x] (Enterprise/hybrid track) Add deploy guardrails that block unsafe multi-instance enterprise rollouts when `rate_limiter`/`ban_store` remain local-only, with explicit override attestation for temporary advisory-only exceptions.
- [x] P1.1 slice completed: `make deploy-env-validate` now enforces multi-instance enterprise state posture (`SHUMA_ENTERPRISE_MULTI_INSTANCE`) and blocks authoritative local-only rate/ban state while requiring explicit advisory/off exception attestation (`SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED=true`) for temporary unsynced operation.
- [x] (Enterprise/hybrid track) Add startup/runtime warnings or hard-fail hooks for enterprise multi-instance local-only state posture, aligned with deploy guardrail semantics.
- [x] P1.2 slice completed: runtime config loading now enforces enterprise multi-instance state guardrails (hard-fail on unsafe posture) and `/admin/config` surfaces enterprise guardrail warnings/errors and attestation visibility fields.
- [x] P1.3 slice completed: replaced external `rate_limiter` stub with a Redis-backed distributed adapter (`INCR` + TTL window key), added explicit fallback-to-internal behavior, and enforced `SHUMA_RATE_LIMITER_REDIS_URL` guardrails for enterprise multi-instance posture.
- [x] P1.4 slice completed: replaced external `ban_store` stub with a Redis-backed distributed adapter (JSON ban entries + Redis TTL), routed admin ban/unban/list paths through provider selection, and enforced `SHUMA_BAN_STORE_REDIS_URL` guardrails for enterprise multi-instance posture.
- [x] P1.5 slice completed: routed admin auth failure throttling through the provider-selected rate limiter so external distributed rate-limiter mode covers admin auth (`/admin/login`, `/admin/logout`, unauthorized admin endpoints) with safe internal fallback when runtime config/provider selection is unavailable.
- [x] P1.6 slice completed: added route-class outage posture controls for external rate limiter degradation (`SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN`, `SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH`) and shipped limiter degradation observability (`backend_errors`, `outage_decisions`, `usage_fallback`, `state_drift` metrics) with deployment/config docs.

- [x] S1.0 taxonomy spec drafted: canonical escalation levels, signal IDs, transition precedence, and current signal-collection map documented in `docs/plans/2026-02-14-stage1-policy-signal-taxonomy-spec.md`.
- [x] Add stable detection ID taxonomy and policy matching using canonical escalation/action IDs from `docs/plans/2026-02-14-stage1-policy-signal-taxonomy-spec.md` (`L0_ALLOW_CLEAN` through `L11_DENY_HARD`) and canonical signal IDs (`S_*`) in logs/metrics/admin events.
- [x] S1.1 slice completed: added `src/runtime/policy_taxonomy.rs` canonical IDs + deterministic precedence tests, threaded policy-match telemetry through runtime/CDP/external event paths, and exposed canonical policy/signal metrics (`bot_defence_policy_matches_total`, `bot_defence_policy_signals_total`) plus taxonomy-annotated admin event outcomes.
- [x] Add static-resource bypass defaults to avoid expensive bot checks on obvious static assets.
- [x] S1.2 slice completed: added early static-asset bypass defaults for obvious `GET`/`HEAD` resource paths/extensions (with admin/challenge/control-path exclusions) to skip expensive JS/botness/geo challenge checks, plus unit + routing-order regression coverage.
- [x] Keep all generated build artifacts out of `src/` (including WASM binaries) and move them to a dedicated artifacts path (for example `dist/wasm/`).
- [x] Update `spin.toml`, Makefile targets, and bootstrap scripts to consume the new artifacts path without changing runtime behavior.
- [x] Keep Playwright and test outputs ephemeral (`playwright-report/`, `test-results/`) and confirm ignore rules remain correct after any directory changes.
- [x] Add a short doc section describing expected generated directories and what should never be committed.

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

- [x] Move orchestration helpers (`request_router`, `kv_gate`, `policy_pipeline`) into a cohesive runtime/policy directory with clear ownership boundaries.
- [x] Group admin/auth/config concerns into a cohesive adapter/domain boundary layout with minimal cross-module leakage.
- [x] Group signal and enforcement modules by domain (for example risk signals, enforcement actions, challenge/maze) and reduce root-level file sprawl.
- [x] Add thin compatibility re-exports during moves so refactors remain reviewable and low-risk.
- [x] Remove temporary compatibility shims once imports are fully migrated.
- [x] H3.1 slice completed: moved request orchestration modules into `src/runtime/` (`runtime/request_router.rs`, `runtime/kv_gate.rs`, `runtime/policy_pipeline.rs`) and rewired `src/lib.rs` call sites without behavior changes.
- [x] H3.2 slice completed: moved admin/auth into `src/admin/` (`admin/mod.rs`, `admin/api.rs`, `admin/auth.rs`) and moved config into `src/config/mod.rs`, then rewired module imports with no behavior change.
- [x] H3.3/H3.4 slice completed: regrouped signal modules under `src/signals/` and enforcement modules under `src/enforcement/`, then added crate-level compatibility re-exports in `src/lib.rs` to keep call sites stable during the move.
- [x] H3.5 slice completed: migrated remaining call sites to `src/signals/*` and `src/enforcement/*` paths and removed temporary compatibility re-exports from `src/lib.rs`.

- [x] Define and document the defence taxonomy with an explicit inventory of `signal`, `barrier`, and `hybrid` modules (for example `rate` as hybrid); include ownership and dependency direction. (`docs/module-boundaries.md`, Defence Taxonomy section)
- [x] Introduce a canonical per-request signal contract (for example `BotSignal` + `SignalAccumulator`) that every signal/hybrid module writes to.
- [x] Add explicit signal availability semantics (`active`, `disabled`, `unavailable`) so botness logic never treats missing modules as silent zero.
- [x] Split hybrid modules into distinct paths:
  rate telemetry signal contribution for scoring,
  hard rate-limit enforcement barrier for immediate protection.
- [x] Add composability modes for eligible modules (`off`, `signal`, `enforce`, `both`) while keeping safety-critical controls non-disableable.
- [x] Define clear behavior for each mode in config/admin surfaces and runtime flow (including invalid combinations and defaults).
- [x] Refactor botness scoring to consume normalized accumulator output rather than direct module internals.
- [x] Lock explicit pre-launch default mode semantics and enforce via tests (`rate=both`, `geo=both`, `js=both`, with JS still gated by `js_required_enforced`).
- [x] Add unit and integration regression tests for mode matrix behavior and ordering invariants (especially hybrid modules and early-route interactions).
- [x] Add observability for mode and signal-state visibility (metrics/log fields indicating enabled/disabled/unavailable contributors).
- [x] Update docs (`configuration`, `features`, `observability`, `module-boundaries`) to explain composability semantics and tuning implications.
- [x] Keep implementations internal-only for now; defer external provider registry/factory work until signal contract and mode semantics stabilize.
- [x] H3.6.1 slice completed: added explicit defence taxonomy + inventory (`signal`, `barrier`, `hybrid`) with ownership and dependency direction in `docs/module-boundaries.md`.
- [x] H3.6.2 slice completed: introduced `BotSignal`/`SignalAccumulator` in `src/signals/botness.rs` and rewired JS, GEO, and rate-pressure botness scoring paths in `src/lib.rs` to emit normalized signal contributions with no behavior change.
- [x] H3.6.3 slice completed: added explicit signal availability states (`active`, `disabled`, `unavailable`) across JS/GEO/rate signal emitters and botness assessment flow, with regression tests for non-silent disabled/unavailable handling.
- [x] H3.6.4 slice completed: split rate hybrid paths into `src/signals/rate_pressure.rs` (telemetry + pressure scoring signals) and `src/enforcement/rate.rs` (hard limit enforcement path), then rewired runtime botness flow accordingly.
- [x] H3.6.5 slice completed: added per-module composability modes (`off`, `signal`, `enforce`, `both`) for JS/GEO/rate with runtime signal/action gating and admin-config validation, preserving default behavior as `both`.
- [x] H3.6.6 slice completed: defined explicit mode semantics in runtime/config/admin surfaces, added effective-mode + warning payloads (`defence_modes_effective`, `defence_mode_warnings`), and validated invalid mode key/value combinations.
- [x] H3.6.7 slice completed: introduced `BotnessSignalContext` and split botness into contribution collection + score finalization (`collect_botness_contributions`, `compute_botness_assessment_from_contributions`) so runtime policy consumes normalized contributions rather than direct scoring internals.
- [x] H3.6.8 slice completed: locked pre-launch default-mode semantics with explicit config tests and added mode-matrix regression coverage for JS/GEO/rate signal paths (including rate hybrid signal behavior), while retaining early-route ordering integration guards.
- [x] H3.6.9 slice completed: added botness signal-state and effective defence-mode observability (`botness_signal_state_total`, `defence_mode_effective_total`) plus richer botness log outcomes (`signal_states`, `modes`) for maze/challenge decisions.
- [x] H3.6.10 slice completed: updated composability/tuning/operator docs (`docs/configuration.md`, `docs/features.md`, `docs/observability.md`, `docs/module-boundaries.md`) with effective-mode semantics and observability guidance.
- [x] H3.6.11 slice completed: kept implementation internal-only (no provider registry/factory introduced) and explicitly deferred external-provider wiring to H4.

- [x] Define provider traits for swappable capabilities:
  rate limiting,
  ban store/sync,
  challenge engine,
  maze/tarpit serving,
  fingerprint signal source.
- [x] Add a provider registry/factory that selects implementations from config (compile-time/runtime config, no behavior change by default).
- [x] Implement `Internal*` providers matching current behavior as the default path.
- [x] Define and document provider externalization matrix by deployment persona:
  `self_hosted_minimal` (default),
  `enterprise_akamai` (target managed-edge integration),
  with advisory-by-default and authoritative-optional edge signal precedence.
- [x] Add explicit `External*` adapter stubs/contracts for high-leverage capabilities first:
  `fingerprint_signal`,
  `rate_limiter`,
  `ban_store`,
  `challenge_engine`,
  with explicit unsupported handling for `maze_tarpit` until a stable external API target exists.
- [x] Add contract tests that every provider implementation must pass to guarantee semantic parity and explicit unavailability behavior (`active`/`disabled`/`unavailable`) for external signal sources.
- [x] Add observability tags/metrics identifying active provider implementation per capability and edge integration mode (`off`/`advisory`/`authoritative`).
- [x] Document provider selection, rollout, and rollback procedures in deployment docs (including Akamai advisory/authoritative guidance and fallback-to-internal behavior).
- [x] H4.1 slice completed: formalized provider capability contracts in `src/providers/contracts.rs` (`RateLimiterProvider`, `BanStoreProvider`, `ChallengeEngineProvider`, `MazeTarpitProvider`, `FingerprintSignalProvider`) with stable enum labels and default-behavior regression tests.
- [x] H4.2 slice completed: added config-backed provider backend selection (`provider_backends` + `SHUMA_PROVIDER_*` defaults), plus `src/providers/registry.rs` factory/registry mapping (`internal`/`external`) with default internal selection and no behavior change to request handling paths.
- [x] H4.3 slice completed: implemented `Internal*` provider adapters in `src/providers/internal.rs` and routed core request/policy flow through registry-selected provider interfaces in `src/lib.rs` and `src/runtime/policy_pipeline.rs` (default behavior preserved under `internal` backends).
- [x] H4.4.1 slice completed: added `edge_integration_mode` posture (`off`/`advisory`/`authoritative`) to config/defaults and threaded it through runtime decision metadata plus metrics export (`bot_defence_edge_integration_mode_total`) without changing enforcement precedence.
- [x] H4.4.2 slice completed: added explicit `external` provider adapters in `src/providers/external.rs`; `fingerprint_signal` now routes to an external stub contract, while `challenge_engine` and `maze_tarpit` remain explicit unsupported adapter paths with safe fallback semantics.
- [x] H4.4.3 slice completed: added provider implementation observability with capability/backend/implementation metrics (`bot_defence_provider_implementation_effective_total`) and runtime event-tag provider summaries (`providers=...`) wired from registry-selected implementations.
- [x] H4.4.4 slice completed: added fingerprint provider contract availability semantics (`active`/`disabled`/`unavailable`) across internal/external adapters plus registry tests enforcing explicit unavailability behavior when external fingerprint is selected but not configured.
- [x] H4.4.5 slice completed: documented deployment personas plus provider selection matrix and added Akamai-focused advisory/authoritative rollout + rollback runbook with explicit fallback-to-internal procedure in `docs/configuration.md`, `docs/deployment.md`, and `docs/observability.md`.
- [x] H4.5 plan follow-up (`docs/plans/2026-02-13-provider-externalization-design.md` step 3): replace external fingerprint stub with an Akamai-first adapter that maps edge/Bot Manager outcomes into normalized fingerprint signals.
- [x] H4.5 slice completed: external `fingerprint_signal` now uses an Akamai-first adapter (`/fingerprint-report`) that normalizes edge/Bot Manager-style outcomes into CDP-tier-compatible signals, retains explicit fallback to the internal CDP handler for non-Akamai/legacy payloads, and exports implementation label `external_akamai_with_internal_fallback`.
- [x] H4.6 plan follow-up (`docs/plans/2026-02-13-provider-externalization-design.md` step 4): implemented external `rate_limiter` and `ban_store` adapters with distributed state/sync semantics and retired unsupported-stub behavior for those capabilities.
- [x] H4.6.1 slice completed: external `rate_limiter` now uses Redis-backed distributed counting (`INCR` + TTL) with explicit internal fallback and provider implementation labeling (`external_redis_with_internal_fallback`).
- [x] H4.6.2 slice completed: external `ban_store` now uses Redis-backed distributed ban state (JSON + TTL) with explicit internal fallback, provider implementation labeling (`external_redis_with_internal_fallback`), and admin ban/unban/list provider routing.
- [x] H4.7 plan follow-up (`docs/plans/2026-02-13-provider-externalization-design.md` step 5): add integration tests for advisory vs authoritative mode precedence and explicit downgrade-to-internal behavior when external providers are unavailable.
- [x] H4.7 slice completed: admin config now supports validated `provider_backends` + `edge_integration_mode` updates, external fingerprint precedence is mode-aware (`off` ignore, `advisory` non-authoritative, `authoritative` strong-edge auto-ban), and integration coverage was added for advisory-vs-authoritative behavior plus external rate-limiter downgrade-to-internal fallback.
- [x] H4.7.1 UX follow-up completed: added an Admin Dashboard Config control for `edge_integration_mode` (`off`/`advisory`/`authoritative`) with save/dirty-state wiring and dashboard e2e smoke coverage so operators can stage and verify H4.7 precedence behavior without manual env edits.

- [x] Define a cutover checklist for enabling any external provider in non-dev environments (staging soak, SLOs, rollback trigger).

- [x] Define platform scope boundaries to avoid overreach by leaning on upstream bot managers (for example Akamai) for features better handled there.
- [x] Add non-secret runtime config export for deploy handoff (exclude secrets) so dashboard-tuned settings can be applied in immutable redeploys.
- [x] P3.1 slice completed: documented Akamai-vs-Shuma platform scope ownership boundaries, non-goals, and decision rules in `docs/bot-defence.md` to keep edge-vs-app responsibilities explicit.
- [x] S1.3.a slice completed: defined canonical request-sequence signal IDs (`S_SEQ_*`) and matching detection IDs (`D_SEQ_*`) in `src/runtime/policy_taxonomy.rs` and documented them in `docs/plans/2026-02-14-stage1-policy-signal-taxonomy-spec.md`.
- [x] S1.3.b slice completed: added signed operation-envelope primitives (`operation_id`, `flow_id`, `step_id`, `issued_at`, `expires_at`, `token_version`) for puzzle/PoW challenge seeds with shared integrity validation in `src/challenge/operation_envelope.rs`, enforced token parse-time validation before scoring paths, and added regression coverage.
- [x] S1.3.c slice completed: added binding/integrity primitives for signed challenge/PoW tokens (`ip_bucket`, `ua_bucket`, `path_class`) with shared request-binding validation, enforced mismatch handling on puzzle/PoW verification paths, and emitted canonical sequence mismatch taxonomy telemetry (`D_SEQ_BINDING_MISMATCH`, `S_SEQ_BINDING_MISMATCH`) instead of silent fallback.
- [x] S1.3.d slice completed: added ordering-window primitives (`step_index`, expected flow/step/index validation, and bounded step windows) for challenge submit and PoW verify paths, mapped order/window failures to canonical taxonomy transitions (`D_SEQ_ORDER_VIOLATION`, `S_SEQ_ORDER_VIOLATION`, `D_SEQ_WINDOW_EXCEEDED`, `S_SEQ_WINDOW_EXCEEDED`), and added deterministic coverage in challenge, envelope, and policy-taxonomy tests.
- [x] S1.3.e slice completed: added timing-threshold primitives (`min_step_latency`, `max_step_latency`, `max_flow_age`, cadence regularity windows/spread TTL) and wired enforcement to challenge submit + PoW verify flows with canonical timing taxonomy transitions (`D/S_SEQ_TIMING_TOO_FAST`, `D/S_SEQ_TIMING_TOO_REGULAR`, `D/S_SEQ_TIMING_TOO_SLOW`).
- [x] S1.3.f slice completed: added replay primitives for operation-level first-seen/duplicate/expired tracking with bounded TTL stores and mapped duplicate/expired operation reuse to canonical replay/expired transitions (`D/S_SEQ_OP_REPLAY`, `D/S_SEQ_OP_EXPIRED`) across challenge and PoW verification.
- [x] S1.3.g slice completed: threaded sequence transitions into policy telemetry (`bot_defence_policy_matches_total`, `bot_defence_policy_signals_total`) and taxonomy-annotated admin event outcomes for challenge submit and PoW sequence violation paths.
- [x] S1.3.h slice completed: added deterministic sequence correctness coverage for challenge/PoW/envelope flows, including valid progression, operation replay, stale expiry, reorder, binding mismatch, too-fast submissions, and too-regular cadence.
- [x] Stage 1 umbrella completion: request-sequence primitives are now end-to-end across taxonomy IDs, signed operation envelopes, binding, ordering windows, timing/replay primitives, telemetry wiring, and regression coverage.
- [x] AI-policy controls slice completed: added first-class admin config keys (`ai_policy_block_training`, `ai_policy_block_search`, `ai_policy_allow_search_engines`) and dashboard controls separate from robots-serving controls while preserving legacy robots-field compatibility.

### todos/todo.md (Stage 2 completion)

- [x] MZ-S1: Keep Stage 2 completion criteria internal-first (no external-provider dependency).
- [x] MZ-S2: Execute Stage 2 delivery order as `MZ-R0 -> MZ-R1 -> MZ-R2 -> MZ-R3 -> MZ-1 -> MZ-2 -> MZ-7 -> MZ-5 -> MZ-3 -> MZ-4 -> MZ-8 -> MZ-9 -> MZ-10 -> MZ-6`.
- [x] MZ-R0: Research-first hold gate accepted from `docs/research/2026-02-14-maze-tarpit-research-synthesis.md`.
- [x] MZ-1 through MZ-10 completed (entropy rotation, signed traversal + replay, budgets, client checkpoint flow, polymorphic rendering, pluggable seed providers/refresh/metadata-only extraction, covert non-maze decoys, crawler simulation harness, botness + observability wiring, rollout/rollback runbook guidance, optional adaptive micro-PoW).


## todos/security-review.md

- [x] Event-log append race fixed (`85bff68`).
- [x] Panic on invalid bool env parsing fixed (`69603c5`).
- [x] Health endpoint spoofing risk hardened with strict trust gate plus optional secret (`163e0dc`).
- [x] Admin login brute-force gap fixed in-app (`add999d`), with deployment-layer guidance added (`40e120c`).
- [x] Unsanitized ban reason storage fixed with sanitization/truncation and dashboard escaping (`4b65e49`).
- [x] Per-request runtime config KV reads fixed with in-memory TTL cache (`09e0017`, docs `88155ab`).
- [x] Browser version parsing robustness improved for edge cases (`b44eeca`).
- [x] "Missing SameSite cookie" report assessed as false positive in current implementation.
- [x] Silent KV error suppression significantly reduced by logging critical write/delete failures (`393e0b1`); low-impact cases remain opportunistic cleanup.

## Additional completions (2026-02-14)

### todos/todo.md

- [x] R-FP-10 Review Li et al., "PathMarker: protecting web contents against inside crawlers" (Cybersecurity 2019) and map path/timing marker concepts to Shuma detection IDs.
- [x] R-RL-02 Review Kuzmanovic/Knightly, "Low-Rate TCP-Targeted DoS Attacks" (SIGCOMM 2003) and map low-rate adversary behaviors to Shuma tarpit/limiter heuristics.
- [x] R-RL-04 Review Veroff et al., "Defense techniques for low-rate DoS attacks against application servers" (Computer Networks 2010) and identify bounded-randomization strategies usable in Shuma tarpit controls.
- [x] R-RL-08 Review Vedula et al., "On the Detection of Low-Rate Denial of Service Attacks at Transport and Application Layers" (Electronics 2021) and map detector candidates to Shuma observability/tuning.
- [x] R-SSH-01 Review Vasilomanolakis et al., "Gotta catch 'em all: A Multistage Framework for Honeypot Fingerprinting" (Digital Threats 2023) and derive anti-fingerprint requirements for SSH tarpit realism.
- [x] MZ-R1: Complete and summarize the highest-impact Maze/Tarpit research items (`R-FP-10`, `R-RL-02`, `R-RL-04`, `R-RL-08`, `R-SSH-01`) with concrete anti-fingerprinting and bounded-cost implications.
- [x] MZ-R2: Map research outcomes to `self_hosted_minimal` vs `enterprise_akamai` ownership and explicitly define what remains internal-first for Stage 2.
- [x] MZ-R3: Convert research findings into enforceable implementation guardrails (budget caps, replay windows, fallback policy, rollout abort thresholds) and update Stage 2 acceptance criteria before coding.

## Additional completions (2026-02-15)

### todos/todo.md (Stage 2.5 completion)

- [x] MZ-X0.R through MZ-X10.R completed via Stage 2.5 research synthesis memo in `/docs/research/2026-02-15-stage2.5-maze-efficiency-and-asymmetry.md`.
- [x] MZ-X0.I completed: Web Worker-first client expansion now uses compact signed seed bootstrap with deterministic fallback behavior when worker/proof cannot complete.
- [x] MZ-X1.I + MZ-X5.I completed: exact path commitment, chain marker checks, sibling edge-operation uniqueness, replay enforcement, and branch-budget-aware progressive issuance checks.
- [x] MZ-X2.I + MZ-X9.I completed: compact maze shell with external versioned shared assets and adaptive styling tiers (full/lite/machine, optional no-CSS deep tier).
- [x] MZ-X3.I completed: hidden links are no longer shipped in bootstrap payload; links are issued progressively via proof/checkpoint-gated `/maze/issue-links`.
- [x] MZ-X4.I + MZ-X6.I completed: proactive pre-render budget/degrade controls and bounded host-write behavior were implemented to reduce per-hop synthesis pressure.
- [x] MZ-X7.I completed: deterministic maze asymmetry benchmark harness + CI gate added (`make test-maze-benchmark`, included in `make test`) with regression-threshold enforcement.
- [x] MZ-X8.I completed: deep-tier micro-PoW and link expansion compute moved off main thread with constrained-device safeguards.
- [x] MZ-X10.I completed: high-confidence violation accumulation now triggers deterministic early fallback before expensive maze serving continues.

## Additional completions (2026-02-15, section-preserving archive)

### todos/todo.md

#### P3 Dashboard Architecture Modernization (Tabbed SPA, Frameworkless-First)
- [x] DSH-4 completed: shared dashboard API client layer added with typed request/response adapters and centralized API error handling (`dashboard/modules/api-client.js`).
- [x] DSH-5 completed: shared dashboard state primitives added with explicit invalidation scopes and tab-local derived state (`dashboard/modules/dashboard-state.js`).
- [x] DSH-6 completed: CDN chart dependency removed; local pinned chart runtime vendored under `dashboard/assets/vendor/chart-lite-1.0.0.min.js` with provenance note in docs.
- [x] DSH-7 completed: active-tab scoped polling added with deterministic suspend/resume and bounded timer count.
- [x] DSH-8 completed: tab accessibility/keyboard behavior strengthened (ARIA visibility semantics, focus management, selected-state behavior).
- [x] DSH-9 completed: progressive `// @ts-check` typing enabled across dashboard modules and orchestration.
- [x] DSH-10 completed: per-tab loading/empty/error states implemented for silent-failure resistance.
- [x] DSH-11 completed: Playwright e2e coverage expanded for tabbed routing, keyboard navigation, and tab error-state surfacing.
- [x] DSH-12 completed: dashboard module unit-style tests added for API adapters, state invalidation, and tab normalization (`e2e/dashboard.modules.unit.test.js`).
- [x] DSH-13 completed: public docs updated (`README.md`, `docs/dashboard.md`, `docs/testing.md`) for tab model and dashboard test workflow.
- [x] DSH-14 completed: migration/rollback notes added to public dashboard docs.
- [x] DSH-G1 closure: framework-adoption gate did not trip after DSH-1..DSH-14; Lit pilot deferred.

## Additional completions (2026-02-15, section-preserving archive)

### todos/todo.md

#### Fingerprinting, JS Verification, and CDP-Adjacent Detection
- [x] R-FP-10 Review Li et al., "PathMarker: protecting web contents against inside crawlers" (Cybersecurity 2019) and map path/timing marker concepts to Shuma detection IDs. https://cybersecurity.springeropen.com/articles/10.1186/s42400-019-0023-1 (summarized in `docs/research/2026-02-14-maze-tarpit-research-synthesis.md`)

#### Rate Limiting, Tarpit, and Cost-Imposition
- [x] R-RL-02 Review Kuzmanovic/Knightly, "Low-Rate TCP-Targeted DoS Attacks" (SIGCOMM 2003) and map low-rate adversary behaviors to Shuma tarpit/limiter heuristics. https://doi.org/10.1145/863955.863966 (summarized in `docs/research/2026-02-14-maze-tarpit-research-synthesis.md`)
- [x] R-RL-04 Review Veroff et al., "Defense techniques for low-rate DoS attacks against application servers" (Computer Networks 2010) and identify bounded-randomization strategies usable in Shuma tarpit controls. https://doi.org/10.1016/j.comnet.2010.05.002 (summarized in `docs/research/2026-02-14-maze-tarpit-research-synthesis.md`)
- [x] R-RL-08 Review Vedula et al., "On the Detection of Low-Rate Denial of Service Attacks at Transport and Application Layers" (Electronics 2021) and map detector candidates to Shuma observability/tuning. https://doi.org/10.3390/electronics10172105 (summarized in `docs/research/2026-02-14-maze-tarpit-research-synthesis.md`)

#### SSH Tarpit and Honeypot Evasion Resistance
- [x] R-SSH-01 Review Vasilomanolakis et al., "Gotta catch 'em all: A Multistage Framework for Honeypot Fingerprinting" (Digital Threats 2023) and derive anti-fingerprint requirements for SSH tarpit realism. https://doi.org/10.1145/3584976 (summarized in `docs/research/2026-02-14-maze-tarpit-research-synthesis.md`)

#### Stage 1: Policy and signal prerequisites
- [x] Add request-sequence signal primitives end-to-end (canonical `S_SEQ_*`/`D_SEQ_*` taxonomy IDs, signed operation envelope fields, binding checks, ordering windows, timing thresholds, replay detection, telemetry wiring, and deterministic/integration coverage).
- [x] S1.3.e Add timing-threshold primitives (min-step-latency, max-step-latency, cadence-regularity threshold, max-flow-age) with conservative defaults tuned for low human false positives.
- [x] S1.3.f Add replay primitives (first-seen/duplicate/expired operation tracking with bounded TTL stores) and map duplicate/reused operations into canonical replay signals.
- [x] S1.3.g Thread sequence signals into botness/policy telemetry (`bot_defence_policy_signals_total`, taxonomy-annotated admin outcomes) and define escalation semantics for advisory vs enforce paths.
- [x] S1.3.h Add deterministic tests for sequence correctness (valid progression, reorder, replay, stale window, too-fast/too-regular cadence, binding mismatch) plus integration coverage for JS/PoW/challenge flows.
- [x] Add AI-bot policy controls as first-class admin config (separate from robots-only controls).

#### Stage 2: Maze excellence execution (Cloudflare-inspired, Shuma-native)
- [x] MZ-S1: Keep Stage 2 completion criteria internal-first (no external-provider dependency).
- [x] MZ-S2: Execute Stage 2 delivery order as `MZ-R0 -> MZ-R1 -> MZ-R2 -> MZ-R3 -> MZ-1 -> MZ-2 -> MZ-7 -> MZ-5 -> MZ-3 -> MZ-4 -> MZ-8 -> MZ-9 -> MZ-10 -> MZ-6`.
- [x] MZ-R0: Research-first hold gate. Do not start Stage 2 implementation slices until the Maze/Tarpit research tranche is synthesized and accepted. (accepted research baseline in `docs/research/2026-02-14-maze-tarpit-research-synthesis.md`)
- [x] MZ-R1: Complete and summarize the highest-impact Maze/Tarpit research items (`R-FP-10`, `R-RL-02`, `R-RL-04`, `R-RL-08`, `R-SSH-01`) with concrete anti-fingerprinting and bounded-cost implications. (`docs/research/2026-02-14-maze-tarpit-research-synthesis.md`)
- [x] MZ-R2: Map research outcomes to `self_hosted_minimal` vs `enterprise_akamai` ownership and explicitly define what remains internal-first for Stage 2. (`docs/research/2026-02-14-maze-tarpit-research-synthesis.md`)
- [x] MZ-R3: Convert research findings into enforceable implementation guardrails (budget caps, replay windows, fallback policy, rollout abort thresholds) and update Stage 2 acceptance criteria before coding. (`docs/research/2026-02-14-maze-tarpit-research-synthesis.md`)
- [x] MZ-1: Replace path-only deterministic seeding with rotating signed entropy for suspicious traffic; keep short TTL deterministic windows for cacheability/debugging.
- [x] MZ-2: Add signed traversal-link tokens with TTL, depth scope, branch budget, and replay protection.
- [x] MZ-7: Enforce maze cost budgets (global concurrency, per-bucket spend, response byte/time caps) with deterministic fallback behavior.
- [x] MZ-5: Make client-side expansion foundational for suspicious maze tiers (Web Worker branch generation + signed server verification) with explicit checkpoint cadence (every 3 nodes or 1500 ms), bounded step-ahead allowance, and no-JS fallback rules.
- [x] MZ-3: Add polymorphic maze rendering (layout/content/link-graph variant families with versioned selection).
- [x] MZ-3.1: Implement pluggable maze content-seed providers (internal default corpus + operator-provided source adapters).
- [x] MZ-3.2: Add manual/scheduled seed refresh for provider-fed corpora with robots/compliance guardrails, caching, and rate limits.
- [x] MZ-3.3: Enforce metadata/keyword-first extraction (avoid article-body copying) to reduce legal risk, bandwidth, and fingerprintability.
- [x] MZ-4: Inject covert decoys into eligible non-maze HTML responses for medium-confidence suspicious traffic while preserving UX/SEO safety.
- [x] MZ-8: Add a crawler simulation harness covering replay, deterministic fingerprinting attempts, JS/no-JS cohorts, and bypass attempts.
- [x] MZ-9: Feed maze traversal behavior into botness scoring/detection IDs and add observability for entropy/token/proof/cost/budget signals.
- [x] MZ-10: Roll out by phase (`instrument -> advisory -> enforce`) with explicit rollback triggers and operator runbook checks.
- [x] MZ-6: Add optional adaptive micro-PoW for deeper traversal tiers.

#### Stage 2 follow-up: Operator-safe Maze Preview
- [x] MZ-PV1: Add an admin-auth-only maze preview endpoint (`GET /admin/maze/preview`) so operators can inspect maze rendering before serving it.
- [x] MZ-PV2: Ensure preview output is non-operational by design (no live traversal tokens, no hidden decoy tracking links, no replay/checkpoint/budget side effects, and no maze hit/risk counter mutation).
- [x] MZ-PV3: Isolate preview entropy/signing from live trap flow (`SHUMA_MAZE_PREVIEW_SECRET` with safe fallback) so leaked preview artifacts cannot forge production traversal.
- [x] MZ-PV4: Add dashboard UX affordance in the Maze config pane ("Preview Maze") that opens the admin preview safely and clearly indicates auth/session requirements.
- [x] MZ-PV5: Add deterministic tests for preview safety guarantees (route auth/read-only behavior, no-live-token markers, and no state mutation) and update docs/API references.

#### Stage 2.5 follow-up: Maze excellence shortfall closure (research-first)
- [x] MZ-X0.R Research optimal client-side branch generation architecture (Web Worker-first, compact signed seed bootstrap, verification cadence) using current state-of-the-art anti-bot/anti-fingerprinting references; publish decision memo with host-cost, attacker-cost, and UX tradeoffs.
- [x] MZ-X0.I Implement Web Worker-based branch generation from compact signed seed for suspicious maze tiers, with server verification protocol and deterministic fallback when worker/proof path fails.
- [x] MZ-X1.R Research optimal signed traversal-token semantics (path commitment granularity, operation-id uniqueness, chain integrity, replay windows, branch budget accounting) and select robust envelope design.
- [x] MZ-X1.I Enforce exact per-link path commitment and full chain constraints in runtime token validation (including `branch_budget` and `prev_digest`) with deterministic fallback and compatibility migration.
- [x] MZ-X2.R Research low-bandwidth maze response delivery patterns (static shell + versioned assets, compression, cache partitioning, anti-fingerprint constraints under no-store policy) and choose target payload budget.
- [x] MZ-X2.I Replace per-hop full inline HTML/CSS/JS with a compact shell + reusable static assets where safe, while preserving deception variability and no-index semantics; include explicit hashed asset/version strategy and cache policy acceptance criteria (for example immutable long-cache static assets with controlled cache-busting on deploy).
- [x] MZ-X3.R Research concealed link-delivery strategies that preserve attacker cost asymmetry (progressive on-demand expansion, encrypted/obfuscated manifests, proof-gated link issuance) without obvious giveaway markers.
- [x] MZ-X3.I Stop shipping the full hidden-link set in bootstrap JSON; move to proof/checkpoint-gated progressive link issuance so bandwidth and traversal state are requester-amortized.
- [x] MZ-X4.R Research host-cost minimization strategies for per-hop maze serving (pre-generation pools, fragment caches, bounded KV write coalescing, lazy state persistence) and pick target CPU/write budgets.
- [x] MZ-X4.I Reduce per-hop host synthesis and write cost by implementing selected caching/pre-generation/write-coalescing strategy with hard budget guardrails.
- [x] MZ-X5.R Research operation-id construction and sibling-token uniqueness patterns to prevent cross-link token reuse and branch-collapse artifacts.
- [x] MZ-X5.I Issue unique child tokens per link edge (operation/path-bound), enforce single-edge replay semantics, and add regression tests for sibling traversal correctness.
- [x] MZ-X6.R Research proactive overload controls for deception systems (pre-render admission control, queue/latency-aware throttles, deterministic degrade ladders) to avoid post-render-only cap enforcement.
- [x] MZ-X6.I Add pre-render admission and deterministic degrade controls so byte/time caps are enforced proactively, not only after render cost is incurred.
- [x] MZ-X7.R Research measurable attacker-vs-defender cost models for maze/tarpit systems (CPU, bandwidth, latency, energy) and define project SLO/SLA thresholds and acceptance tests.
- [x] MZ-X7.I Add repeatable benchmark harness + CI gates that report host and attacker-side cost deltas and fail regressions against defined asymmetry targets.
- [x] MZ-X8.R Research client-side compute fairness controls (battery/thermal sensitivity, mobile safeguards, main-thread impact) for deep-tier micro-PoW and JS expansion.
- [x] MZ-X8.I Move deep-tier proof and expansion compute fully off the main thread, add adaptive safeguards for constrained clients, and validate no significant human UX regression.
- [x] MZ-X9.R Research deception-page styling minimalism: quantify anti-fingerprint benefit vs byte/energy cost of CSS, determine when no-CSS is safe, and define tiered styling policy by botness confidence + traversal depth.
- [x] MZ-X9.I Implement adaptive maze styling tiers: minified external shared stylesheet at low/medium suspicion, ultra-minimal style at high suspicion, and optional no-CSS templates at high-confidence deep traversal before ban threshold; tier selection must key on botness score + traversal depth + violation history, and no-CSS variants must remain plausible machine-oriented surfaces (not obviously broken or synthetic giveaway pages).
- [x] MZ-X10.R Research confidence-accumulation escalation models for deception systems (stacked violation semantics, false-positive controls, rollback criteria) to stop expensive maze serving earlier without premature giveaway.
- [x] MZ-X10.I Add pre-ban high-confidence early-escalation matrix (for example replay + binding mismatch + checkpoint/proof failures) that deterministically degrades from maze serving to lower-cost challenge/block actions.

## Additional completions (2026-02-16, section-preserving archive)

### todos/todo.md

#### Stage 2.7 follow-up: Honeypot + Maze stealth excellence (research-first, pre-launch no-compat mode)
- [x] MZ-SR1 Publish a current research synthesis for stealth deception routing and honeypot fingerprinting resistance with explicit source mapping and implementation requirements. (`docs/research/2026-02-16-honeypot-maze-stealth-excellence.md`)
- [x] MZ-S1 Remove explicit `/trap/*` route handling and trap-specific robots bait comments; keep deception routes non-semantic and reduce immediate classifier signal.
- [x] MZ-S2 Introduce an opaque, deployment-specific maze route namespace (secret-derived prefix) and route helper API; remove remaining public `/maze/*` labels from live routing paths.
- [x] MZ-S3 Move maze support endpoints (`checkpoint`, `issue-links`) and versioned maze assets under the same opaque namespace and update worker/bootstrap generation to consume helper paths only.
- [x] MZ-S4 Remove deception-path disclosure from `robots.txt` defaults (no explicit maze/trap path disallow lines or bait comments); keep robots focused on crawler policy communication, not trap advertisement.
- [x] MZ-S5 Update admin preview + dashboard links to use runtime path helpers so preview reflects live namespace while staying non-operational.
- [x] MZ-S6 Add regression tests for route stealth and canonicalization (slash variants, malformed prefixes, old explicit-path rejection) across unit/integration paths.
- [x] MZ-S7 Refresh public docs (`docs/maze.md`, `docs/api.md`, `docs/configuration.md`, `README.md`/`docs/quick-reference.md`) to describe the new opaque routing model and operator expectations.
- [x] MZ-S8 Re-run benchmark and verification gates (`make test`, `make build`) and record resource/behavior deltas for stealth migration.
  Verification notes (2026-02-16): `make test` passed end-to-end (unit + benchmark + integration + dashboard e2e), `make build` passed, and maze benchmark gate reported `pages=6 avg_page_bytes=6638 host_set_ops=46 host_write_bytes=511 attacker_requests=16 issue_links_calls=5 attacker_pow_iterations=3553`.

#### Direction Snapshot (for next implementation stages)
- [x] Evolve maze behavior toward Cloudflare-style selective covert decoys for suspicious traffic with opaque, non-semantic route namespaces (no explicit `/maze` or `/trap` public labels).

#### P3 Dashboard Architecture Modernization (Tabbed SPA, Frameworkless-First)
##### Baseline and decision gate
- [x] DSH-R1 Baseline current dashboard architecture and runtime costs (JS/CSS bytes, startup time, memory, polling cadence, bundle provenance, current e2e coverage) and publish a short decision memo in `docs/plans/`. (`docs/plans/2026-02-15-dashboard-architecture-modernization.md`)
- [x] DSH-R2 Evaluate two implementation tracks against Shuma constraints: (A) frameworkless modular SPA + JSDoc typing, (B) ultra-light framework (Lit) with equivalent tab shell; include explicit tradeoffs for maintenance, DX, runtime weight, and migration risk. (`docs/plans/2026-02-15-dashboard-architecture-modernization.md`)
- [x] DSH-R3 Define framework-adoption gate criteria (for example: unresolved lifecycle complexity, repeated DOM/state bugs, unacceptable change lead time after frameworkless refactor); default to no framework unless gate is tripped. (`docs/plans/2026-02-15-dashboard-architecture-modernization.md`)

##### Tabbed SPA shell and structure (frameworkless path)
- [x] DSH-1 Implement tabbed SPA shell in `dashboard/index.html` + `dashboard/dashboard.js` with canonical tabs: `Monitoring`, `IP Bans`, `Status`, `Config`, `Tuning`.
- [x] DSH-2 Add URL-backed tab routing (`#monitoring`, `#ip-bans`, `#status`, `#config`, `#tuning`) with refresh-safe deep links and history navigation.
- [x] DSH-3 Refactor monolithic dashboard orchestration into tab-scoped controllers/modules with clear lifecycle (`init`, `mount`, `unmount`, `refresh`) and no cross-tab hidden coupling.

#### Fingerprinting, JS Verification, and CDP-Adjacent Detection
- [x] R-FP-01 Review Peter Eckersley, "How Unique Is Your Web Browser?" (PETS 2010) and extract entropy-design implications for Shuma fingerprint signals and replay windows. https://link.springer.com/chapter/10.1007/978-3-642-14527-8_1 (summarized in `docs/research/2026-02-16-fingerprinting-research-synthesis.md`)
- [x] R-FP-02 Review Acar et al., "The Web Never Forgets" (CCS 2014) and derive tracking/fingerprint abuse patterns relevant to bot-detection evasion hardening. https://doi.org/10.1145/2660267.2660347 (summarized in `docs/research/2026-02-16-fingerprinting-research-synthesis.md`)
- [x] R-FP-03 Review Vastel et al., "FP-STALKER" (IEEE S&P 2018) and define time-evolution checks for Shuma fingerprint consistency logic. https://doi.org/10.1109/SP.2018.00008 (summarized in `docs/research/2026-02-16-fingerprinting-research-synthesis.md`)
- [x] R-FP-04 Review Jonker/Krumnow/Vlot, "Fingerprint Surface-Based Detection of Web Bot Detectors" (ESORICS 2019) and identify detector-surface minimization requirements. https://doi.org/10.1007/978-3-030-29962-0_28 (summarized in `docs/research/2026-02-16-fingerprinting-research-synthesis.md`)
- [x] R-FP-05 Review Azad et al., "Web Runner 2049: Evaluating Third-Party Anti-bot Services" and extract anti-evasion architecture lessons for internal-vs-edge integration boundaries. https://pmc.ncbi.nlm.nih.gov/articles/PMC7338186/ (summarized in `docs/research/2026-02-16-fingerprinting-research-synthesis.md`)
- [x] R-FP-06 Review Iliou et al., "Detection of advanced web bots by combining web logs with mouse behavioural biometrics" (DTRAP 2021) and assess feasibility of low-friction behavior features in Shuma. https://doi.org/10.1145/3447815 (summarized in `docs/research/2026-02-16-fingerprinting-research-synthesis.md`)
- [x] R-FP-07 Review Zhao et al., "Toward the flow-centric detection of browser fingerprinting" (Computers & Security 2024) and evaluate flow-level JS signal extraction options. https://doi.org/10.1016/j.cose.2023.103642 (summarized in `docs/research/2026-02-16-fingerprinting-research-synthesis.md`)
- [x] R-FP-08 Review Venugopalan et al., "FP-Inconsistent: Detecting Evasive Bots using Browser Fingerprint Inconsistencies" (2024) and define cross-attribute consistency checks for Shuma scoring. https://arxiv.org/abs/2406.07647 (summarized in `docs/research/2026-02-16-fingerprinting-research-synthesis.md`)
- [x] R-FP-09 Review Bursztein et al., "Picasso: Lightweight Device Class Fingerprinting for Web Clients" (SPSM 2016) and assess replay-resistant challenge-bound fingerprint options. https://doi.org/10.1145/2994459.2994467 (summarized in `docs/research/2026-02-16-fingerprinting-research-synthesis.md`)
- [x] Strengthen fingerprinting by hardening internal baseline signals first, then ingesting trusted upstream edge signals (JA3/JA4 and similar) with provenance checks and explicit internal fallback when edge headers are absent or untrusted.
- [x] Phase 1 completed: normalized fingerprint signals now carry provenance/confidence metadata, family entropy budgeting/caps are enforced, and data-minimization controls (TTL/pseudonymization/export visibility) are wired and documented.
- [x] Phase 2 completed: cross-layer mismatch heuristics (UA/client-hint/transport), temporal coherence detection IDs, and bounded flow-window fingerprint telemetry are active.
- [x] Phase 3 completed: versioned CDP probe-family rotation (`v1`/`v2`/`split`) is active, trusted transport-header ingestion is implemented, persistence-abuse signals are emitted, challenge-bound short-lived marker checks are wired, and low-friction micro-signal checks are added with conservative weighting.
- [x] Phase 4 completed (except Finch spike): fingerprint-focused admin visibility/tuning surfaces are shipped (`/admin/cdp` config + `fingerprint_stats`, dashboard cards), and evasive-regression coverage was added for detector variation, temporal drift, and inconsistency bypass classes.

## Additional completions (2026-02-17, section-preserving archive)

### todos/todo.md

#### P3 Dashboard Architecture Modernization (Frameworkless-First)
- [x] DSH-ARCH-1 Add shared dashboard core utilities: `core/format.js` (escaping + numeric/date helpers + shallow equality) and `core/dom.js` (DOM cache + safe setters + write scheduler), then consume them from feature modules.
- [x] DSH-ARCH-2 Consolidate writable config path inventories into a single `config-schema.js` source and consume it from both Status inventory rendering and Advanced Config template generation.
- [x] DSH-ARCH-3 Replace fragmented per-pane saved-state bags with a single `config-draft-store` baseline (`get/set/isDirty`) used by config dirty-check paths.
- [x] DSH-ARCH-4 Reduce config bind coupling by switching `config-controls.bind(...)` callsites to a typed `context` object and adding normalization coverage in dashboard module unit tests.
- [x] DSH-ARCH-5 Add render-performance guards: skip chart redraws when data/labels are unchanged and batch refresh-driven DOM writes through one scheduler cycle.
- [x] DSH-ARCH-6 Remove uncached hot-path `getElementById` usage from `dashboard.js` and `config-controls.js` by routing lookups through shared DOM cache helpers.

#### P3 Dashboard Native ESM + Functional JS Modernization (No Build Step)
- [x] DSH-ESM-1 Hard cutover selected for pre-launch: migrate dashboard JS to native ESM without dual global-script wiring; decision recorded in `docs/plans/2026-02-17-dashboard-native-esm-hard-cutover.md`.
- [x] DSH-ESM-2 Freeze behavior contracts to preserve during refactor: tab routing/hash behavior, API payload expectations, status/config control semantics, and monitoring render states. (`docs/plans/2026-02-17-dashboard-esm-behavior-contracts.md`)
- [x] DSH-ESM-3 Add/expand regression coverage before migration for all dashboard tabs (`loading`/`empty`/`error`/`data`) and critical config dirty-state/save flows. (`e2e/dashboard.smoke.spec.js`, `e2e/dashboard.modules.unit.test.js`)
- [x] DSH-ESM-4 Introduce a single native module entrypoint (`<script type="module">`) and convert dashboard boot from global-init order to explicit imports.
- [x] DSH-ESM-5 Replace `window.ShumaDashboard*` global module registry wiring with ESM `export`/`import` contracts across dashboard modules.
- [x] DSH-ESM-6 Define and enforce a stable module graph (`core` -> `services` -> `features` -> `main`) with no circular imports. (`docs/plans/2026-02-17-dashboard-esm-module-graph.md` + module-graph guard test)
- [x] DSH-ESM-7 Refactor feature modules to functional boundaries: pure `deriveViewModel(snapshot, options)` and side-effectful `render(viewModel, effects)`; no class-based state.
- [x] DSH-ESM-8 Centralize side effects in dedicated effect adapters (DOM writes, network calls, clipboard, timers) so feature logic remains pure/testable. (`dashboard/modules/services/runtime-effects.js`)
- [x] DSH-ESM-9 Consolidate dashboard state updates around immutable transition functions (`nextState = reduce(prevState, event)`) and remove ad-hoc mutable globals where possible.
- [x] DSH-ESM-10 Standardize function style for new/changed dashboard code: default parameter values, arrow functions for local/pure helpers and callbacks, and explicit named function declarations only where hoisting/readability is clearly beneficial.
- [x] DSH-ESM-11 Remove legacy IIFE wrappers and duplicate helper code paths that were only needed for global-script loading.
- [x] DSH-ESM-12 Add lightweight static guard checks for dashboard JS (for example: fail on new `window.ShumaDashboard*` exports, fail on `class` usage in dashboard modules, fail on duplicate helper definitions across modules).
- [x] DSH-ESM-13 Execute migration in small slices with mandatory full verification per slice via Makefile (`make test` with dev Spin running).
- [x] DSH-ESM-14 Update public and contributor docs (`docs/dashboard.md`, architecture plan, contributor notes) with native ESM conventions, functional patterns, and module-boundary rules.
- [x] DSH-ESM-15 Run a final no-net-behavior-change audit against baseline contracts and capture known intentional deltas (if any) before merge. (`docs/plans/2026-02-17-dashboard-esm-no-net-behavior-audit.md`)

## Additional completions (2026-02-17, section-preserving archive)

### todos/todo.md

#### P3 Dashboard Modernization Follow-up (Functional + ESM Refinement)
- [x] DSH-FUP-1 Replace repeated config save-handler boilerplate in `dashboard/modules/config-controls.js` with shared functional helpers (save-state transitions, status message helpers, and error-path normalization) while preserving exact button labels/dirty-check timing.
- [x] DSH-FUP-2 Consolidate repeated `check*ConfigChanged` patterns in `dashboard/dashboard.js` into a schema-driven dirty-check registry + generic evaluator to reduce copy-paste state logic and event binding drift.
- [x] DSH-FUP-3 Replace repeated `configUiState` wrapper functions in `dashboard/dashboard.js` with a dispatch/invoke helper so config snapshot refresh is declarative and less error-prone.
- [x] DSH-FUP-4 Refactor tab refresh orchestration into a tab-handler map (including shared config-backed tabs) instead of if/else branching for `status`/`config`/`tuning`.
- [x] DSH-FUP-5 Add a shared status-panel patch helper to coalesce `statusPanel.update(...)` + `statusPanel.render()` across dashboard modules and remove duplicate render-trigger code paths.
- [x] DSH-FUP-6 Move monitoring loading placeholder reset logic out of `dashboard/dashboard.js` and into `dashboard/modules/monitoring-view.js` so monitoring rendering state is feature-owned.
- [x] DSH-FUP-7 Reduce `configControls.bind(...)` coupling by replacing the broad callback bag with a focused domain API object (typed by shape and covered by module tests).
- [x] DSH-FUP-8 Replace inline style mutations for test-mode visual state with semantic classes/CSS tokens and add coverage to prevent style regressions.
- [x] DSH-FUP-9 Expand dashboard save-flow test coverage for robots serving, AI policy, GEO scoring/routing, CDP config, and botness config to catch regressions that unit adapter tests miss.

## Additional completions (2026-02-17, section-preserving archive)

### todos/todo.md

#### P3 Dashboard Functional Excellence Remediation (Post-Review)
- [x] DSH-FEX-1 Remove global `window.fetch` monkey patching from admin session flow and move CSRF/session write handling into explicit request paths (idempotent, no global side effects).
- [x] DSH-FEX-2 Harden dashboard boot with safe DOM-binding guards for optional/missing elements so markup drift cannot crash initialization.
- [x] DSH-FEX-3 Refactor status rendering to instance-based state (`create(...)`) rather than module-level mutable singleton state.
- [x] DSH-FEX-4 Decompose `config-controls.bind(...)` orchestration into declarative save-handler wiring primitives to reduce mixed concerns and imperative branching.
- [x] DSH-FEX-5 Improve DOM cache semantics to avoid stale/null permanence (re-resolve disconnected or previously missing nodes) with focused unit coverage.
- [x] DSH-FEX-6 Reduce config-control coupling by replacing the monolithic `domainApi` callback bag with smaller capability namespaces and compatibility tests.
- [x] DSH-FEX-7 Add regression coverage for: session-auth write CSRF behavior, missing-control boot resilience, and status instance isolation.

## Additional completions (2026-02-18, section-preserving archive)

### todos/todo.md

#### P0 Dashboard SvelteKit Full Cutover (All Tabs, Excellence Architecture)
- [x] DSH-SVLT-R0 Record architecture decision for SvelteKit full cutover and supersede the prior framework migration direction (`docs/adr/0002-dashboard-sveltekit-cutover.md`).
- [x] DSH-SVLT-R1 Preserve route and behavior contracts (`/dashboard/index.html`, `/dashboard/login.html`, hash-tab UX) during migration.
- [x] DSH-SVLT-R2 Keep deployment static-only (adapter-static + Spin fileserver), with no Node server in production runtime.
- [x] DSH-SVLT-PLAT1 Add SvelteKit app scaffolding under `dashboard/` with static adapter output to `dist/dashboard`.
- [x] DSH-SVLT-PLAT2 Wire `spin.toml` dashboard static source to `dist/dashboard`.
- [x] DSH-SVLT-PLAT3 Add canonical dashboard build integration to `make dev`, `make run`, and `make build`.
- [x] DSH-SVLT-UI1 Move dashboard/login page shells into Svelte routes while preserving exact design and DOM IDs.
- [x] DSH-SVLT-LIFE1 Introduce explicit Svelte route lifecycle bridges that mount legacy dashboard/login runtimes.
- [x] DSH-SVLT-LIFE2 Keep local chart runtime vendored and loaded from static assets under the SvelteKit base path.
- [x] DSH-SVLT-NEXT1 Replace legacy runtime bridge with Svelte-native store/actions for tab lifecycle, polling, and session/auth state.
- [x] DSH-SVLT-NEXT1.1 Add centralized dashboard store module (`state`, `actions`, `selectors`) for active tab, auth/session, tab status (loading/error/empty), snapshots, and stale flags.
- [x] DSH-SVLT-NEXT1.2 Add explicit effect adapters for network, timers, history/hash writes, and page-visibility events; forbid direct effect calls from UI components.
- [x] DSH-SVLT-NEXT1.3 Replace hash/tab behavior from legacy coordinator with Svelte-owned tab action pipeline (`activateTab`, keyboard nav, hash sync, reload persistence).
- [x] DSH-SVLT-NEXT1.4 Add Svelte-owned polling scheduler with per-tab cadence (`30s/45s/60s`) and visibility pause/resume semantics matching current behavior.
- [x] DSH-SVLT-NEXT1.5 Add Svelte-owned auth/session bootstrap (`/admin/session` check, login redirect, logout action, csrf token propagation).
- [x] DSH-SVLT-NEXT1.6 Move config dirty-state tracking from legacy runtime into store-level draft baselines and section-local derived selectors.
- [x] DSH-SVLT-NEXT1.7 Gate legacy bridge boot behind a migration toggle and switch default path to Svelte-native store/actions once parity tests pass.
- [x] DSH-SVLT-NEXT2 Split monitoring/ip-bans/status/config/tuning into dedicated Svelte component trees with declarative rendering.
- [x] DSH-SVLT-NEXT2.1 Create shared Svelte UI primitives for tab state messages, stat cards, table wrappers, and empty/loading/error blocks.
- [x] DSH-SVLT-NEXT2.2 Implement Monitoring component tree (cards, charts, events table, monitoring summaries, Prometheus helper) using declarative rendering only.
- [x] DSH-SVLT-NEXT2.3 Implement IP Bans component tree (ban table, quick-unban interactions, row-detail expansion) with store-driven actions.
- [x] DSH-SVLT-NEXT2.4 Implement Status component tree (status cards + runtime variable inventory tables) with shared schema-driven metadata.
- [x] DSH-SVLT-NEXT2.5 Implement Config component tree split by concern (maze, robots/ai policy, geo, honeypot, browser policy, bypass lists, challenge/pow, cdp, edge mode, advanced JSON).
- [x] DSH-SVLT-NEXT2.6 Implement Tuning component tree (botness thresholds/weights/status blocks) with the same save/dirty architecture as Config.
- [x] DSH-SVLT-NEXT2.7 Migrate chart lifecycle management into Svelte-friendly adapters (`onMount`/`onDestroy`, no global chart instance leaks).
- [x] DSH-SVLT-NEXT2.8 Complete no-net-behavior parity pass against current smoke contracts for all five tabs before deleting legacy path.
- [x] DSH-SVLT-NEXT3 Remove legacy shell source files once Svelte-native component parity is complete.
- [x] DSH-SVLT-NEXT3.1 Remove shell fragment injection path (`src/lib/shell/*.html` + `{@html ...}`) after Svelte-native component parity is complete.
- [x] DSH-SVLT-NEXT3.2 Remove bridge modules (`src/lib/bridges/*.js`) and legacy runtime boot globals once no longer referenced.
- [x] DSH-SVLT-NEXT3.3 Remove or archive superseded legacy dashboard entry shell dependencies (`dashboard/index.html`, `dashboard/login.html`) from active runtime path.
- [x] DSH-SVLT-NEXT3.4 Remove unused legacy orchestration modules from active dependency graph and keep only reusable domain adapters.
- [x] DSH-SVLT-NEXT3.5 Add static guardrails preventing reintroduction of bridge-era anti-patterns (`{@html}` shell injection, route-level legacy runtime imports).
- [x] DSH-SVLT-TEST1 Add targeted tests for Svelte route bridge lifecycle (single-mount guarantees, duplicate listener prevention, teardown behavior).
- [x] DSH-SVLT-TEST1.1 Add unit tests for single-mount guarantees when route is revisited (no duplicate listeners/timers/intervals).
- [x] DSH-SVLT-TEST1.2 Add unit tests for teardown behavior on route unmount (listener cleanup, polling stop, chart cleanup).
- [x] DSH-SVLT-TEST1.3 Add unit tests for auth/session bootstrap transitions (`authenticated`, `unauthenticated`, `expired`) in Svelte-native path.
- [x] DSH-SVLT-TEST1.4 Add unit tests for hash-route/tab keyboard behavior in Svelte-native tab actions.
- [x] DSH-SVLT-TEST2 Expand Playwright assertions for generated SvelteKit asset/runtime loading under `/dashboard` base path.
- [x] DSH-SVLT-TEST2.1 Add Playwright assertions that dashboard static assets resolve under `/dashboard/_app/*` and `/dashboard/assets/*` without 4xx/5xx.
- [x] DSH-SVLT-TEST2.2 Add Playwright assertion that `/dashboard/login.html` stays functional after direct navigation and refresh.
- [x] DSH-SVLT-TEST2.3 Add Playwright assertion that `/dashboard` redirect contract remains `308 -> /dashboard/index.html`.
- [x] DSH-SVLT-TEST2.4 Add Playwright runtime-failure guardrails for missing module/stylesheet/script requests in generated SvelteKit output.
- [x] DSH-SVLT-DOC1 Update dashboard docs to reflect SvelteKit runtime, file layout, and rollback procedure.

## Additional completions (2026-02-18, section-preserving archive)

### todos/todo.md

#### P1 Dashboard SvelteKit Post-Cutover Excellence
- [x] DSH-SVLT-EX1 Remove remaining import-time DOM/event bindings in `dashboard/dashboard.js`; move all bindings to mount-scoped setup with deterministic teardown so route remounts remain safe.
- [x] DSH-SVLT-EX2 Continue extracting orchestration out of `dashboard/dashboard.js` into `dashboard/src/lib/runtime/*`, leaving `dashboard/modules/*` as pure domain adapters.
- [x] DSH-SVLT-EX3 Resolve current Svelte a11y warnings in dashboard tab semantics (`tablist`/`tabpanel`) while preserving keyboard/hash contracts and smoke coverage.
- [x] DSH-SVLT-EX4 Add `AbortController`-based request cancellation/dedupe for tab switches and polling to prevent stale render overwrites and wasted refresh work.
- [x] DSH-SVLT-EX5 Add explicit dashboard runtime performance telemetry (fetch latency, render timing, polling skip/resume counters) and document operator thresholds.
- [x] DSH-SVLT-EX6 Add route-remount e2e coverage (navigate away/back) and assert that ban/unban, save flows, polling, and keyboard tab navigation still function.
- [x] DSH-SVLT-EX7 Replace the temporary query-param legacy toggle with an explicit config-driven runtime switch and rollout/rollback docs.

#### P1 Dashboard SvelteKit Excellence Round 3 (Native Hardening + Perf Budgets)
- [x] DSH-SVLT-EX13 Remove native-mode dependency on `mountDashboardRuntime` by extracting remaining refresh/session/tab adapter calls out of `dashboard/dashboard.js` into Svelte runtime modules; native mode should not require legacy app mount flags.
- [x] DSH-SVLT-EX14 Replace runtime chart script injection in `src/routes/+page.svelte` (`ensureScript`) with a deterministic static load path (preload/import strategy) to reduce mount-time variability and simplify lifecycle cleanup.
- [x] DSH-SVLT-EX15 Collapse Monitoring auto-refresh fan-out further by consuming a consolidated Monitoring summary contract (aligned with `MON-TEL-4`) so the native polling path does not require multiple endpoint reads per cycle.
- [x] DSH-SVLT-EX16 Add dashboard performance gates to CI/Make flow: bundle-size ceilings for `/dashboard/_app` assets and polling request-budget assertions for native remount/steady-state flows.
- [x] DSH-SVLT-EX17 Reduce repeated full-table DOM churn on Monitoring refresh by adding bounded row diff/patch updates (or virtualization where needed) for high-volume event/CDP tables.

## Additional completions (2026-02-18, section-preserving archive)

### todos/todo.md

#### P1 Dashboard SvelteKit Excellence Round 2 (Architecture + Performance)
- [x] DSH-SVLT-EX8 Continue shrinking the `dashboard/dashboard.js` hotspot by extracting config-dirty orchestration and save-check wiring into `dashboard/src/lib/runtime/*` with typed capability contracts.
- [x] DSH-SVLT-EX9 Reduce native Monitoring-tab auto-refresh fan-out by removing redundant request paths and documenting the bounded request budget per refresh cycle.
- [x] DSH-SVLT-EX10 Upgrade runtime telemetry aggregation from unbounded lifetime averages to bounded rolling windows (for example last `N` samples + p95) with deterministic reset semantics.
- [x] DSH-SVLT-EX11 Add repeated remount stress coverage (multiple navigate-away/back loops) that asserts no timer/listener/request duplication over time.
- [x] DSH-SVLT-EX12 Remove remaining direct DOM/window reads from action pipelines (redirect path, focus target lookup) by routing them through effect adapters for stricter testability.

#### P1 Dashboard SvelteKit Excellence Round 4 (Native Decoupling + Perf Hardening)
- [x] DSH-SVLT-EX18 Remove `dashboard/dashboard.js` from the native runtime refresh path by moving remaining tab-refresh/session orchestration into `dashboard/src/lib/runtime/*` modules with explicit typed contracts.
- [x] DSH-SVLT-EX19 Implement and consume a consolidated Monitoring data contract for manual/native refresh cycles (close `MON-TEL-4` alignment) so Monitoring detail updates avoid multi-endpoint fan-out.
- [x] DSH-SVLT-EX20 Replace global chart runtime script dependency with a module-scoped chart adapter lifecycle (lazy import + singleton guard + teardown) to minimize global side effects.
- [x] DSH-SVLT-EX21 Add no-flicker Monitoring auto-refresh coverage (no placeholder reset on auto cycles, bounded table patch churn assertions) in dashboard smoke + module tests.
- [x] DSH-SVLT-EX22 Add native remount/refresh soak performance gate (bounded fetch/render p95 + stable polling cadence across repeated mount loops) and wire into Make/CI reporting.

#### P0 Branch Handoff (dashboard-sveltekit-port -> main)
- [x] HND-SVLT-1 Resume from branch `codex/dashboard-sveltekit-port` at commit `979fa2f` (with `c7291e5` included immediately before it in branch history).
  - Completed on branch `codex/dashboard-sveltekit-port`; current tip was `86b42bf` (contains remount fan-out test stabilization).
- [x] HND-SVLT-2 In an unrestricted shell, run canonical verification only through Makefile paths:
  - terminal A: `make dev`
  - terminal B: `make test`
  - required outcome: Rust unit + maze benchmark + integration + dashboard e2e all green.
  - Completed on 2026-02-18 after commit `86b42bf`; `make test` passed end-to-end (including dashboard e2e).
- [x] HND-SVLT-3 If verification is green, open/update PR from `codex/dashboard-sveltekit-port` into `main` and include:
  - SvelteKit migration summary (hard cutover with no archived legacy fallback assets),
  - Makefile-only workflow enforcement updates (`AGENTS.md`, `CONTRIBUTING.md`, `Makefile`),
  - dashboard runtime/perf guardrails (`e2e` remount fan-out + bundle budget gate).
  - Completed on 2026-02-18: PR opened as `https://github.com/atomless/Shuma-Gorath/pull/1` with required handoff summary.
  - DNS troubleshooting outcome in Codex runtime: resolved (`curl -I https://api.github.com` returned `HTTP/2 200`; `gh api rate_limit` succeeded).
- [x] HND-SVLT-4 Merge to `main` after CI is green; then continue Round 4 items (`DSH-SVLT-EX18..EX22`) on a fresh `codex/*` branch.
  - Completed on 2026-02-18: work merged into `main`; Round 4 implementation and canonical verification (`make verify`, `make test`, `make build`) completed cleanly from `main`.
