# Challenge Lite (Checkbox) Spec

## Purpose

Provide a low-friction step-up challenge for medium-uncertainty traffic, before escalating to the full puzzle challenge.

This is a scored verification checkpoint, not a boolean checkbox gate.

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
3. Challenge Lite (new) for medium-uncertainty traffic
4. Puzzle challenge for stronger verification when Lite is inconclusive
5. Allow

## Endpoints

- `GET /challenge/not-a-bot-checkbox`
  - Returns the checkbox challenge page with signed nonce payload.
- `POST /challenge/not-a-bot-checkbox`
  - Validates nonce + telemetry summary.
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
4. Single-use key (`challenge_lite_used:<id>`) in KV
5. Attempt cap per IP bucket/window

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

Server should ignore extra fields and reject invalid ranges.

## Scoring Model (Initial)

Compute a `lite_score` with weighted signals:

- Event integrity (`events_order_valid`) => high weight
- Timing plausibility (`down_up_ms`, `interaction_elapsed_ms`) => medium weight
- Motion plausibility (`pointer_*`) => medium weight
- Context features from existing pipeline (`rate_pressure`, `geo_risk`, prior failures) => medium/high weight

Keep score bounded to `0..10`.

## Outcome Thresholds (Initial)

- `pass` if `lite_score >= 7` and no hard-fail signals
- `escalate_puzzle` if `4 <= lite_score < 7`
- `maze_or_block` if `lite_score < 4` or hard-fail signal present

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

- `challenge_lite_served_total`
- `challenge_lite_pass_total`
- `challenge_lite_escalate_total`
- `challenge_lite_fail_total`
- `challenge_lite_replay_total`

Add latency histogram:

- `challenge_lite_solve_ms`

## Security & Privacy Notes

- Treat telemetry as anti-abuse signal only; avoid persistent user profiling.
- Never expose scoring internals in client responses.
- Use generic failure messages.
- Keep debug detail behind explicit dev-only flags.

## Testing Requirements

- Unit tests for nonce validation, scoring, thresholds, and hard-fail paths
- Integration tests for full GET/POST lifecycle and replay handling
- Dashboard/admin tests for new metrics visibility
