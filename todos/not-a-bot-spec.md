# Not-a-Bot Checkbox Spec

## Purpose

Provide a low-friction step-up challenge for medium-uncertainty traffic, before escalating to the stronger puzzle challenge.

This is a scored verification checkpoint, not a boolean checkbox gate.
Default behavior should remain adaptive: low certainty should pass without interactive friction, medium certainty should receive not-a-bot, higher certainty should escalate to puzzle/maze.
For very-low certainty traffic, Shuma should stay in managed/invisible mode (passive signals + JS verification/PoW only when required) without forcing this interactive step.

## Scope

- Edge-only implementation (no third-party CAPTCHA provider)
- Same-origin only
- Single-use, short-lived nonce
- Deterministic server-side verification + routing

Out of scope:

- Cross-site reputation graph
- Third-party identity providers

## Routing Position

Proposed order:

1. Hard blocks / bans / explicit policy routes
2. Maze route for high-confidence automation
3. Not-a-Bot checkbox for medium-uncertainty traffic
4. Puzzle challenge for stronger verification when Lite is inconclusive
5. Allow

## Endpoints

- `GET /challenge/not-a-bot-checkbox`
  - Returns the Not-a-Bot checkbox page with signed nonce payload.
- `POST /challenge/not-a-bot-checkbox`
  - Validates nonce + telemetry summary.
  - Triggered by a single activation of the checkbox-like control (no extra Continue button).
  - Returns routing outcome (`pass`, `escalate_puzzle`, `maze_or_block`).

## Nonce Model

Signed payload fields:

- `id` (random challenge id)
- `issued_at`
- `expires_at` (target: 60-120s)
- `ip_bucket`
- `sig` (HMAC)

Server checks:

1. Signature validity
2. Expiry
3. IP bucket match
4. Single-use key (`not_a_bot_used:<id>`) in KV
5. Attempt cap per IP bucket/window

Operational controls:

- `not_a_bot_risk_threshold` (`1..10`, must remain below puzzle threshold)
- `not_a_bot_nonce_ttl_seconds` (short TTL; bounded range)
- `not_a_bot_marker_ttl_seconds` (short continuity marker TTL)
- `not_a_bot_attempt_limit_per_window`
- `not_a_bot_attempt_window_seconds`
- `not_a_bot_score_pass_min` and `not_a_bot_score_escalate_min`

## Client Telemetry Contract

Submit compact numeric features (not full event streams):

- `has_pointer` (bool)
- `pointer_move_count` (u16)
- `pointer_path_length` (f32)
- `pointer_direction_changes` (u16)
- `down_up_ms` (u32)
- `focus_changes` (u8)
- `visibility_changes` (u8)
- `interaction_elapsed_ms` (u32)
- `keyboard_used` (bool)
- `touch_used` (bool)
- `events_order_valid` (bool)
- `activation_method` (`pointer|touch|keyboard|unknown`)
- `activation_trusted` (bool)
- `activation_count` (u8)
- `control_focused` (bool)

Server should ignore extra fields and reject invalid ranges.

Accessibility parity requirement:

- Keyboard-only and touch-only interactions must be pass-capable with equivalent verification strength (no mandatory pointer-motion dependency).
- Accessibility interaction patterns are never negative signals by themselves.

## Scoring Model (Initial)

Compute a `not_a_bot_score` with weighted signals:

- Event integrity (`events_order_valid`) => high weight
- Timing plausibility (`down_up_ms`, `interaction_elapsed_ms`) => medium weight
- Motion plausibility (`pointer_*`) => medium weight
- Accessibility-equivalent modality plausibility (`keyboard_used`/`touch_used` with timing and order checks) => medium weight
- Activation semantics (`activation_method`, trusted activation, bounded activation count, focused control) => medium weight
- Context features from existing pipeline (`rate_pressure`, `geo_risk`, prior failures) => medium/high weight

Keep score bounded to `0..10`.

## Outcome Thresholds (Initial)

- `pass` if `not_a_bot_score >= 7` and no hard-fail signals
- `escalate_puzzle` if `4 <= not_a_bot_score < 7`
- `maze_or_block` if `not_a_bot_score < 4` or hard-fail signal present

Hard-fail examples:

- nonce invalid/replayed/expired
- impossible event ordering
- repeated rapid failures from same IP bucket

## Token / Session Handling

On `pass`, issue a short-lived verification marker (cookie or token) for route continuity.

- Scope: same-origin
- TTL: short (for example 5-10 minutes)
- Bind to IP bucket (and optionally session id)

## Metrics

Add counters:

- `not_a_bot_served_total`
- `not_a_bot_pass_total`
- `not_a_bot_escalate_total`
- `not_a_bot_fail_total`
- `not_a_bot_replay_total`

Add latency histogram:

- `not_a_bot_solve_ms`

Monitoring parity (dashboard summary):

- outcomes: `served`, `pass`, `escalate`, `fail`, `replay`
- solve-latency buckets (bounded labels)
- estimated abandonment: `served - submitted`

## Security & Privacy Notes

- Treat telemetry as anti-abuse signal only; avoid persistent user profiling.
- Never expose scoring internals in client responses.
- Use generic failure messages.
- Keep debug detail behind explicit dev-only flags.
- Keep telemetry schema compact and fixed to control serialization/storage costs.
- PAT/private-attestation style signals (if added) must be additive evidence only, not stand-alone allow.

## Testing Requirements

- Unit tests for nonce validation, scoring, thresholds, and hard-fail paths
- Integration tests for full GET/POST lifecycle and replay handling
- Dashboard/admin tests for new metrics visibility
