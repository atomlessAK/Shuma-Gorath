# Not-a-Bot Checkbox Excellence Plan

Date: 2026-02-13  
Last revised: 2026-02-19  
Status: Active

## Context

Not-a-bot (`/challenge/not-a-bot-checkbox`) is the intended low-friction checkpoint between passive signals and puzzle/maze escalation. The module existed only as a placeholder.

## Goals

- Add a strong lightweight challenge for medium-uncertainty traffic.
- Keep verifier cost bounded and deterministic under attack.
- Make outcome routing explainable: `pass`, `escalate_puzzle`, `maze_or_block`.
- Preserve accessibility with equivalent-strength keyboard/touch completion semantics.

## Non-goals

- Third-party CAPTCHA dependency in the default path.
- Stateful long-term user profiling.
- Replacing puzzle or maze as high-certainty controls.

## Research-derived constraints

1. Server-side token verification must be short-lived and single-use.
2. Checkbox success cannot be binary authority; it must be scored with corroborating signals.
3. Responses must be oracle-resistant; detailed failure reasons stay internal.
4. Equivalent accessibility path is mandatory for production quality.
5. Monitoring/output dimensions must stay low-cardinality and bounded for cost control.
6. Optional PAT-like signals are additive only and cannot be sole allow authority.
7. Interaction must be single-step (checkbox-like activation progresses immediately; no secondary submit button).
8. Accessibility paths are neutral-to-positive evidence only; never penalize assistive usage patterns directly.

Reference synthesis: `docs/research/2026-02-19-not-a-bot-challenge-research-synthesis.md`.

## Internal vs Akamai ownership

- `self_hosted_minimal`:
  - Internal not-a-bot endpoint, scoring, and replay controls.
- `enterprise_akamai`:
  - Edge risk signals can tune trigger thresholds.
  - Shuma remains authoritative for nonce semantics and outcome routing.

## Proposed architecture

### A. Signed nonce lifecycle

- Short-lived signed payload with operation id, flow step fields, and expiry.
- Bind to IP bucket + UA bucket + expected submit path class.
- Enforce single-use replay marker with bounded TTL.

### B. Compact telemetry contract

- Strict typed/ranged summary fields only.
- Reject malformed payloads; ignore no untrusted dynamic extras.
- No raw high-cardinality event-stream ingestion.

### C. Deterministic scoring and outcomes

- Score normalized to `0..10`.
- Hard-fail conditions bypass weighted score.
- Outcome routing:
  - `pass` -> short-lived continuity marker,
  - `escalate_puzzle` -> stronger challenge,
  - `maze_or_block` -> policy-driven high-cost path.

### D. Abuse controls

- Per-IP-bucket attempt cap in short windows.
- Replay rejection with explicit metric labels.
- Generic external failure responses.

### E. Escalation placement

- Trigger not-a-bot below puzzle threshold.
- Trigger puzzle above not-a-bot threshold.
- Trigger maze above maze threshold or on repeated high-confidence abuse.

### F. Runtime controls + monitoring parity

- Runtime controls for not-a-bot route threshold, nonce/marker TTL, and attempt caps.
- Monitoring parity for not-a-bot:
  - outcomes (`served`, `pass`, `escalate`, `fail`, `replay`),
  - solve-latency buckets,
  - abandonment estimate (`served - submitted`).
- Dashboard exposure without introducing unbounded payload cardinality.

## Ordered implementation sequence

- [x] NAB-1: Implement GET/POST not-a-bot endpoints and signed nonce parse/verify.
- [x] NAB-2: Add strict telemetry schema validation and bounded scoring (`0..10`).
- [x] NAB-3: Implement continuity marker token issuance and verification on pass.
- [x] NAB-4: Add attempt caps, cooldown, replay tracking, and generic failure responses.
- [x] NAB-5: Wire policy routing for lower botness certainty -> not-a-bot -> puzzle escalation.
- [x] NAB-6: Add runtime controls for routing threshold, nonce/marker TTL, and attempt caps.
- [x] NAB-7: Add monitoring + dashboard parity (`served`, `pass`, `escalate`, `fail`, `replay`, solve latency, abandonment estimate).
- [x] NAB-8: Add lifecycle tests for success + all failure classes (unit + integration + dedicated browser e2e).
- [x] NAB-9: Add operator docs and threshold tuning guidance.
- [ ] NAB-10: Evaluate optional PAT-like attestation adapter as additive low-friction signal (non-blocking).
- [x] NAB-11: Align UI with state-of-the-art one-step interaction (`role=checkbox` control, auto-submit on activation, remove separate Continue action).
- [x] NAB-12: Formalize and document very-low-certainty invisible flow mapping (passive signals + JS/PoW path) so not-a-bot remains medium-certainty only.
- [x] NAB-13: Update scoring semantics for activation-modality evidence while preserving equivalent-strength keyboard/touch pass paths.

## Source references

- `todos/not-a-bot-spec.md`
- `docs/research/2026-02-19-not-a-bot-challenge-research-synthesis.md`
- https://www.akamai.com/products/bot-manager
- https://techdocs.akamai.com/cloud-security/docs/handle-adversarial-bots
- https://developers.cloudflare.com/cloudflare-challenges/challenge-types/challenge-pages/
- https://developers.cloudflare.com/turnstile/get-started/server-side-validation/
