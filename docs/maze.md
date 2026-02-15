# 🐙 Maze

The maze is Shuma-Gorath's deception subsystem: a synthetic crawl space designed to absorb suspicious automation while keeping normal human traffic friction low.

## 🐙 Maze Excellence Mission

Maze excellence is about asymmetry:

- increase attacker cost (time, traversal effort, compute, and bandwidth),
- keep defender cost bounded and energy-aware,
- preserve human UX and avoid SEO regressions,
- maintain operator control and explainable policy outcomes.

This is the core stance for `L7_DECEPTION_EXPLICIT` behavior and related deception flows.

## 🐙 Guiding Aims and Principles

1. **Asymmetric cost placement**
   Most incremental cost should land on malicious visitors, not on host infrastructure.
2. **Adaptive depth by confidence**
   Low-suspicion traffic gets minimal or no maze friction; high-suspicion traffic gets deeper deception.
3. **Anti-fingerprinting by design**
   Avoid globally stable decoys by using rotating entropy, signed traversal tokens, and variant families.
4. **Bounded host budgets**
   Enforce hard limits (concurrency, bytes, time, per-bucket spend) with deterministic fallback.
5. **Progressive escalation**
   Maze is one control in a ladder, not a silo; saturated or bypassed maze paths escalate predictably.
6. **Observability first**
   Every major maze decision should remain measurable and explainable in metrics/events.

## 🐙 How Maze Pages Are Populated

Current Stage 2 behavior combines polymorphic rendering, signed traversal state, and bounded-cost controls:

1. **Polymorphic variant families**
   Maze pages render with rotating layout/palette/content variants and emit a stable variant marker (`X-Shuma-Maze-Variant`) for observability.
2. **Signed rotating entropy**
   Variant selection uses HMAC-backed entropy inputs (path, IP bucket, UA bucket, short entropy window, and flow nonce), not path-only deterministic hashes.
3. **Signed traversal links**
   Maze links carry signed `mt` traversal tokens with TTL/depth/branch budget/previous-node binding and replay tracking.
4. **Client-side expansion + checkpoints**
   Suspicious-tier traversal renders visible + hidden link sets and emits checkpoint state via `/maze/checkpoint` to validate ongoing traversal cadence.
5. **No-JS bounded fallback**
   Missing checkpoints allow bounded no-JS progress up to configured depth, then deterministically trigger fallback.
6. **Adaptive deep-tier micro-PoW**
   Optional per-link micro-PoW is required at deeper traversal depths, verified server-side via token-bound nonce checks.
7. **Pluggable seed corpora**
   Content seeding supports internal corpus defaults and operator-fed sources with metadata-first extraction and refresh/rate-limit guardrails.
8. **Strict budget governor**
   Maze runtime enforces global and per-IP-bucket concurrency caps plus response byte/time caps.
9. **Covert decoys in non-maze HTML**
   Medium-suspicion challenge responses can include hidden decoy links (`dc=1`) to detect covert decoy-follow behavior while preserving visible UX.

### Content Sources and Safety Rules

- Use synthetic or operator-approved metadata corpora only.
- Do not mirror user/private application data into maze pages.
- Keep generated content disposable and non-authoritative.
- Preserve explicit robots signaling for trap routes and honeypot paths.
- Prefer metadata/keyword extraction over article-body copying for operator sources.

## 🐙 Rollout and Rollback (MZ-10)

Maze rollout is phase-gated via `maze_rollout_phase`:

- `instrument`
  - violations are measured and scored; enforcement fallback is not applied.
- `advisory`
  - budget violations are enforced; token/checkpoint/proof violations remain advisory.
- `enforce`
  - token, replay, binding, checkpoint, proof, and budget violations all enforce fallback behavior.

Recommended rollout order:

1. Start in `instrument` for baseline telemetry.
2. Move to `advisory` after baseline quality checks.
3. Promote to `enforce` once abort-threshold checks remain healthy.

Rollback triggers (sustained for 10 minutes) should force phase rollback or pause:

- budget saturation above `2%` of eligible suspicious traffic,
- protected-route p95 latency regression above `20%` versus baseline,
- protected-route non-2xx/3xx rate increase above `0.5%` absolute,
- challenge abandonment / human-success degradation beyond operator SLO.

## 🐙 Signal Inputs That Shape Maze Behavior

Maze complexity/routing can consume:

- local signals (rate, geo posture, challenge outcomes, JS/CDP observations),
- traversal signals (ordering windows, timing thresholds, replay checks),
- trusted upstream enterprise signals (for example edge-provided bot outcomes) when configured.

Signal collection informs policy; the maze remains Shuma-controlled policy composition by default.

## 🐙 Configuration

These fields are part of the runtime config (`/admin/config`):

- `maze_enabled` (bool) - Enable or disable the maze.
- `maze_auto_ban` (bool) - Auto-ban after threshold.
- `maze_auto_ban_threshold` (u32) - Number of maze pages before auto-ban.
- `maze_rollout_phase` (`instrument|advisory|enforce`) - stage-gated enforcement mode.
- `maze_token_*`, `maze_replay_ttl_seconds` - token integrity + replay windows.
- `maze_entropy_window_seconds`, `maze_path_entropy_segment_len` - entropy/fingerprint controls.
- `maze_client_expansion_enabled`, `maze_checkpoint_*`, `maze_step_ahead_max`, `maze_no_js_fallback_max_depth` - checkpoint/no-JS policy.
- `maze_micro_pow_*` - optional deep-tier micro-PoW settings.
- `maze_max_concurrent_*`, `maze_max_response_*` - cost-budget controls.
- `maze_seed_provider`, `maze_seed_refresh_*`, `maze_seed_metadata_only` - seed corpus controls.
- `maze_covert_decoys_enabled` - non-maze covert decoy injection toggle.

## 🐙 Admin Endpoint

- `GET /admin/maze` - Returns maze stats for the dashboard.
- `GET /admin/maze/seeds` - Lists operator seed sources and cached corpus snapshot.
- `POST /admin/maze/seeds` - Upserts operator seed sources.
- `POST /admin/maze/seeds/refresh` - Triggers manual operator-corpus refresh.

## 🐙 Metrics

- `bot_defence_maze_hits_total` tracks total maze page hits.
- `bot_defence_maze_token_outcomes_total{outcome=...}` tracks token-validation outcomes.
- `bot_defence_maze_checkpoint_outcomes_total{outcome=...}` tracks checkpoint submission outcomes.
- `bot_defence_maze_budget_outcomes_total{outcome=...}` tracks budget acquisition/saturation/cap outcomes.
- `bot_defence_maze_proof_outcomes_total{outcome=...}` tracks micro-PoW proof requirements/outcomes.
- `bot_defence_maze_entropy_variants_total{variant,provider,metadata_only}` tracks entropy-family/provider use.

## 🐙 Notes

- If you do not want crawler trapping, set `maze_enabled` to `false`.
- Auto-ban uses the `maze_crawler` reason in metrics and events.
- For deeper implementation detail, see `docs/plans/2026-02-13-maze-excellence-plan.md`.
