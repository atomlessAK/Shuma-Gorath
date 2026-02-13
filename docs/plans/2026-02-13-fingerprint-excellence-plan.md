# Fingerprint Excellence Plan

Date: 2026-02-13
Status: Proposed

## Context

Fingerprinting currently relies on a limited set of in-app signals and an external provider stub path. This leaves high-value edge transport signals underused and limits resistance to evasive bots that keep changing browser traits over time.

## Goals

- Build a normalized, provenance-aware fingerprint signal model.
- Detect cross-attribute inconsistencies and temporal drift.
- Keep false positives low by combining weak signals instead of over-trusting one source.
- Support Akamai-first edge signal ingestion without losing self-hosted portability.

## Non-goals

- Universal multi-vendor feature parity across all edge providers.
- Replacing Shuma botness policy orchestration with external policy engines.

## State-of-the-art Signals

1. Fingerprint stability and entropy are measurable but drift over time.
2. Detector surfaces are fingerprintable; static detector behavior is eventually evaded.
3. Inconsistency detection (rather than static fingerprint matching) is effective against evasive automation.
4. Path and timing provenance improve classifier quality when fused with browser/network signals.

## Internal vs Akamai Ownership

- `self_hosted_minimal`:
  - Internal signals as primary.
  - No hard dependency on external transport telemetry.
- `enterprise_akamai`:
  - Akamai transport/bot telemetry as primary source for network-layer attributes.
  - Shuma keeps normalization, weighting, correlation, and final routing decisions.

## Proposed Architecture

### A. Normalized fingerprint schema

- Define canonical attributes with provenance and confidence:
  - browser/runtime,
  - header/client-hint consistency,
  - transport identity (JA3/JA4-class inputs),
  - sequence/timing features.
- Include `source=internal|akamai` and `availability=active|disabled|unavailable`.

### B. Consistency engine

- Add explicit consistency checks across attributes.
- Score mismatch classes rather than simple boolean fails.
- Separate hard-fail patterns from soft-suspicion patterns.

### C. Temporal and session coherence

- Track short-window coherence for suspect traffic.
- Detect impossible fingerprint transitions within bounded time.
- Keep memory and retention bounded with TTL windows.

### D. Provider integration

- Replace external fingerprint stub with Akamai-first mapping adapter.
- Keep strict fallback to internal signal paths when external data is absent.

### E. Safety and explainability

- Log reasons with stable detection IDs.
- Keep operator-visible attribution for each score contribution.

## Rollout Strategy

1. Instrument schema and provenance without changing enforcement.
2. Enable consistency scoring in advisory mode.
3. Add policy thresholds for challenge/maze routing.
4. Enable authoritative edge precedence only after drift and false-positive baselines are stable.

## Structured Implementation TODOs

1. FP-1: Finalize normalized fingerprint schema and versioning.
2. FP-2: Implement Akamai edge outcome mapping into normalized fields.
3. FP-3: Add consistency rules for UA/client-hint/transport mismatches.
4. FP-4: Add temporal coherence windows and bounded state retention.
5. FP-5: Add stable detection IDs for mismatch classes.
6. FP-6: Extend botness weighting with provenance-aware confidence.
7. FP-7: Add dashboard/operator views for fingerprint attribution.
8. FP-8: Add replay and evasive-bot simulation tests.
9. FP-9: Add advisory vs authoritative integration tests.
10. FP-10: Publish rollback criteria and runbook for edge-authoritative mode.

## Source References

- https://link.springer.com/chapter/10.1007/978-3-642-14527-8_1
- https://doi.org/10.1109/SP.2018.00008
- https://doi.org/10.1007/978-3-030-29962-0_28
- https://arxiv.org/abs/2406.07647
- https://cybersecurity.springeropen.com/articles/10.1186/s42400-019-0023-1
- https://techdocs.akamai.com/cloud-security/docs/handle-adversarial-bots
