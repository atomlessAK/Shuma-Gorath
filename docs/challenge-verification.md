# 🐙 Challenge Verification

This document describes the challenge system that is currently implemented.

## 🐙 Current Puzzle

- Grid puzzle with one worked example (`Before` -> `After`) and one user attempt
- 4x4 grid with two active tones (black/pink)
- Active cells per puzzle are randomized (7-9)
- User solves by selecting the 1st and 2nd transforms from the legend
- Transform set is ordered and configurable by count:
  - `shift up`, `shift down`, `shift left`, `shift right`
  - `90° clockwise`, `90° anticlockwise`
  - `mirror horizontal`, `mirror vertical`

## 🐙 Challenge Routes

- `GET /challenge/puzzle`
  - Serves the challenge page when `test_mode=true` in runtime config
- `POST /challenge/puzzle`
  - Submits and verifies the answer

Challenge rendering is also used by runtime routing when policy/botness logic decides to step up to challenge.

## 🐙 Seed & Verification Model

Each challenge includes a signed seed token carrying:

- `seed_id`
- `issued_at`
- `expires_at`
- `ip_bucket`
- puzzle parameters (`grid_size`, `active_cells`, `transforms`, `training_count`, `seed`)

Verification checks:

1. Form body size and shape validation
2. Seed token signature validation (HMAC)
3. Seed expiry check
4. IP bucket binding check
5. Single-use enforcement via KV key `challenge_used:<seed_id>`
6. Exact output match against deterministically generated solution

Secrets:

- Seed signing uses `SHUMA_CHALLENGE_SECRET` when set
- Fallback secret is `SHUMA_JS_SECRET`

## 🐙 Response Behavior

- Correct answer: `200` with success page
- Incorrect answer: `403 Incorrect.` + `Request new challenge.` link
- Expired or replayed seed: `403 Expired` + `Request new challenge.` link
- Invalid/tampered token: `403 Forbidden. Please request a new challenge.` + link

## 🐙 Security Properties (Current)

- Edge-served generation and verification
- Short-lived seeds (5 minute expiry)
- Single-use seed replay protection
- Request/IP bucket binding
- No debug transform disclosure in page output

## 🐙 Config Knobs

- `SHUMA_CHALLENGE_ENABLED` (enable/disable challenge serving at challenge-tier routing; disabled falls back to maze or block)
- `SHUMA_CHALLENGE_TRANSFORM_COUNT` (clamped to `4..8`)
- `SHUMA_CHALLENGE_RISK_THRESHOLD` (challenge routing threshold in botness flow)
- `SHUMA_CHALLENGE_CONFIG_MUTABLE` (admin mutability control)

## 🐙 Metrics

Challenge metrics currently emitted include:

- `bot_defence_challenges_total`
- `bot_defence_challenge_served_total`
- `bot_defence_challenge_solved_total`
- `bot_defence_challenge_incorrect_total`
- `bot_defence_challenge_expired_replay_total`
