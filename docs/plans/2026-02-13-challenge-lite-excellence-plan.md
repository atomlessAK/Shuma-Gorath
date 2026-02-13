# Challenge Lite Excellence Plan

Date: 2026-02-13
Status: Proposed

## Context

Challenge Lite (`/challenge/not-a-bot-checkbox`) has a detailed spec but is not implemented yet. This is the intended medium-friction checkpoint between passive signals and heavier puzzle/maze escalation.

## Goals

- Add a low-friction, high-signal verification step for uncertain traffic.
- Keep scoring deterministic and explainable.
- Resist replay and scripted checkbox-only bypass attempts.

## Non-goals

- Third-party CAPTCHA dependency.
- Persistent behavioral profiling across unrelated sessions.

## State-of-the-art Signals

1. Challenge success should be scored, not treated as a single boolean.
2. Event-order and timing plausibility can separate scripted from real interactions.
3. Strong anti-replay controls are required for low-friction checks.

## Internal vs Akamai Ownership

- `self_hosted_minimal`:
  - Internal challenge-lite endpoint and scoring engine.
- `enterprise_akamai`:
  - Akamai bot risk can influence challenge-lite trigger thresholds.
  - Shuma remains authoritative for challenge-lite scoring and step-up routing.

## Proposed Architecture

### A. Signed nonce lifecycle

- Implement short-lived, single-use signed nonce model.
- Bind nonce to IP bucket and issue timestamp.

### B. Compact telemetry scoring

- Accept only bounded, typed telemetry fields.
- Compute server-side `0..10` score with hard-fail paths.

### C. Outcome routing

- `pass` -> short-lived continuity token.
- `escalate_puzzle` -> stronger challenge.
- `maze_or_block` -> cost-imposition or deny path by policy.

### D. Abuse controls

- Per-bucket attempt caps.
- Replay cache with TTL.
- Generic failure responses to limit oracle abuse.

## Rollout Strategy

1. Implement endpoint and nonce validation first.
2. Run scoring in shadow mode and measure separability.
3. Enable outcomes with conservative thresholds.
4. Tune weights and thresholds by observed false-positive rates.

## Structured Implementation TODOs

1. CL-1: Implement GET/POST challenge-lite endpoints.
2. CL-2: Implement signed nonce issuance and single-use tracking.
3. CL-3: Add strict telemetry schema validation and range checks.
4. CL-4: Implement score model with hard-fail conditions.
5. CL-5: Add pass/escalate/maze-or-block routing outputs.
6. CL-6: Add short-lived continuity token on pass.
7. CL-7: Add per-bucket attempt caps and cooldowns.
8. CL-8: Add metrics (`served`, `pass`, `escalate`, `fail`, `replay`, latency).
9. CL-9: Add full lifecycle and replay regression tests.
10. CL-10: Add operator docs and threshold tuning runbook.

## Enterprise Offering Snapshot (Akamai and Cloudflare)

- Akamai:
  - Bot Manager uses risk scoring with configurable response segments that include challenge enforcement at the edge.
  - This is useful as upstream triage for when Shuma should trigger challenge-lite.
- Cloudflare:
  - Challenge Pages provide managed challenge flows for suspicious requests.
  - Turnstile provides explicit token attestation with mandatory server-side verification, useful as optional external corroboration.
- Planning implication:
  - Keep challenge-lite scoring and nonce semantics internal for explainability and deterministic policy behavior.
  - In enterprise profiles, use Akamai/Cloudflare challenge outcomes as threshold modifiers and escalation signals, not as authoritative replacements.

## Source References

- `todos/challenge-lite-spec.md`
- https://doi.org/10.1007/3-540-39200-9_18
- https://doi.org/10.1126/science.1160379
- https://doi.org/10.1145/2556288.2557322
- https://doi.org/10.3390/app13074602
- https://www.akamai.com/products/bot-manager
- https://techdocs.akamai.com/cloud-security/docs/handle-adversarial-bots
- https://developers.cloudflare.com/cloudflare-challenges/challenge-types/challenge-pages/
- https://developers.cloudflare.com/turnstile/get-started/server-side-validation/
