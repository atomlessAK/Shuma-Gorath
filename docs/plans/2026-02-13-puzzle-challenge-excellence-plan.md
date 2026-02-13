# Puzzle Challenge Excellence Plan

Date: 2026-02-13
Status: Proposed

## Context

Puzzle challenge is implemented (`/challenge/puzzle`) with signed seed, deterministic generation, and replay prevention through challenge-use tracking. It is strong relative to current project stage but needs future-proofing against model-assisted solvers and behavioral replay.

## Goals

- Keep puzzle challenge robust against automated solving improvements.
- Maintain accessibility and predictable UX.
- Preserve stateless verification and bounded server cost.

## Non-goals

- Building a fully dynamic heavy puzzle engine that materially increases host CPU cost.
- Obscure puzzles that harm legitimate completion rates.

## State-of-the-art Signals

1. Static challenge families become learnable; variant diversity is required.
2. Usability strongly affects real-user completion and security effectiveness.
3. Challenge systems should combine content difficulty with interaction integrity checks.

## Internal vs Akamai Ownership

- `self_hosted_minimal`:
  - Internal puzzle engine remains primary.
- `enterprise_akamai`:
  - Akamai risk can trigger or skip puzzle presentation.
  - Puzzle generation and verification remain internal for consistency and explainability.

## Proposed Architecture

### A. Variant expansion

- Add multiple puzzle families and transform sets.
- Rotate variant families with versioned metadata.

### B. Solve integrity

- Tie puzzle submissions to signed nonce and strict expiry.
- Add optional lightweight interaction plausibility checks.

### C. Adaptive routing

- Trigger puzzle only for uncertainty bands where lite challenge is inconclusive.
- Escalate repeatedly failed puzzle sessions to maze/tarpit/block policy.

### D. Accessibility parity

- Introduce equivalent-strength accessible challenge modality.
- Keep token semantics (expiry, single-use, signature) identical across modalities.

## Rollout Strategy

1. Instrument solve outcomes and latency per variant.
2. Introduce variant family rotation in advisory mode.
3. Add escalation rules for repeated failures.
4. Validate accessibility completion rates before widening enforcement.

## Structured Implementation TODOs

1. PZ-1: Add puzzle variant family abstraction and versioning.
2. PZ-2: Add variant rotation controls and metadata logging.
3. PZ-3: Add submission nonce binding and stricter replay guarantees.
4. PZ-4: Add optional solve-integrity telemetry features.
5. PZ-5: Add adaptive routing from challenge-lite outcomes.
6. PZ-6: Add repeated-failure escalation rules.
7. PZ-7: Add accessibility-equivalent modality with equal verification strength.
8. PZ-8: Add per-variant solve-rate and latency metrics.
9. PZ-9: Add adversarial solver regression tests.
10. PZ-10: Publish operator tuning and rollback guidance.

## Source References

- https://doi.org/10.1145/1455770.1455838
- https://doi.org/10.1145/2556288.2557322
- https://doi.org/10.3390/electronics14224403
- https://doi.org/10.3390/app13074602
- `docs/challenge-verification.md`
