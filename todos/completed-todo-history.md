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
