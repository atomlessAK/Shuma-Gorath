# PoW Excellence Plan

Date: 2026-02-13
Status: Proposed

## Context

PoW is currently available as an optional JS interstitial step (`/pow` and `/pow/verify`) with fixed difficulty and TTL controls. Verification is stateless and IP-bucket bound, but difficulty is not yet risk-adaptive.

## Goals

- Shift marginal compute cost toward abusive automation.
- Keep human latency impact minimal by default.
- Prevent replay, outsourcing, and stale-token abuse.
- Keep predictable host resource consumption.

## Non-goals

- Global mandatory PoW for all users.
- Cryptocurrency-style long-running proof computations.

## State-of-the-art Signals

1. Client puzzles are effective when difficulty adapts to abuse pressure.
2. Stateless signed challenge payloads scale better than server-heavy puzzle state.
3. PoW should be one part of a risk-tiered challenge stack, not the default gate.

## Internal vs Akamai Ownership

- `self_hosted_minimal`:
  - Internal PoW issuance/verification as optional step-up.
- `enterprise_akamai`:
  - Akamai challenge confidence can gate when PoW is required.
  - Shuma keeps PoW policy, parameters, and fallback behavior.

## Proposed Architecture

### A. Adaptive difficulty engine

- Set difficulty from risk tier and recent abuse pressure.
- Add hard caps for max solve cost and max allowed wall-clock.

### B. Token and replay hardening

- Keep signed short-lived seed payloads.
- Add one-time token IDs for high-risk tiers.
- Enforce strict expiry and attempt budgets per bucket.

### C. Solver and verifier efficiency

- Keep verifier constant-time and bounded-memory.
- Add optional Web Worker solve path to reduce UI blocking.

### D. Escalation logic

- Failed PoW does not always block immediately.
- Route outcomes by policy: retry -> challenge-lite/puzzle -> maze/tarpit/block.

## Cost-allocation Targets

| Cost component | Target bearer | Approach |
| --- | --- | --- |
| Proof search CPU | Bot primary | Browser-side solve with adaptive difficulty |
| Verification CPU | Host bounded | Constant-time hash checks and strict limits |
| Retry overhead | Bot primary | Attempt caps and cooldown windows |
| State memory | Host bounded | Short TTL state only for elevated tiers |

## Rollout Strategy

1. Instrument solve-time and pass/fail telemetry.
2. Enable adaptive difficulty in advisory mode.
3. Gradually enforce by risk tier.
4. Tune limits to keep human impact within SLO.

## Structured Implementation TODOs

1. POW-1: Add adaptive difficulty policy based on risk and recent abuse.
2. POW-2: Add one-time challenge IDs for high-risk replay resistance.
3. POW-3: Add attempt caps and per-bucket cooldowns.
4. POW-4: Add optional Web Worker solve implementation.
5. POW-5: Add verifier-side latency and failure reason metrics.
6. POW-6: Add escalation matrix for PoW failure outcomes.
7. POW-7: Add integration tests for expiry, mismatch, replay, and stale nonce.
8. POW-8: Add config guardrails for max difficulty and TTL bounds.
9. POW-9: Add enterprise Akamai-aware trigger policy hooks.
10. POW-10: Document rollout thresholds and rollback conditions.

## Source References

- https://www.microsoft.com/en-us/research/publication/pricing-via-processing-or-combatting-junk-mail/
- https://www.ndss-symposium.org/ndss1999/cryptographic-defense-against-connection-depletion-attacks/
- https://nakamotoinstitute.org/library/hashcash/
- https://doi.org/10.1016/j.simpa.2022.100335
- https://techdocs.akamai.com/cloud-security/docs/handle-adversarial-bots
