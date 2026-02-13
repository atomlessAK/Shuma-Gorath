# HTTP Tarpit Excellence Plan

Date: 2026-02-13
Status: Proposed

## Context

HTTP tarpit behavior is planned (`maze_plus_drip`) but not yet implemented as a bounded production feature. Provider contracts include a tarpit seam (`maybe_handle_tarpit`) but runtime behavior remains effectively maze-only today.

## Goals

- Add bounded slow-drip cost imposition for high-confidence abusive clients.
- Keep host-side resource usage strictly budgeted.
- Ensure deterministic fallback when tarpit budgets saturate.

## Non-goals

- Unbounded long-lived streams that degrade host availability.
- Tarpitting uncertain or likely-human traffic.

## State-of-the-art Signals

1. Cost-imposition controls work best when tightly bounded and risk-gated.
2. Low-rate attack patterns require both scheduling and budget visibility.
3. Escalation policies need explicit guardrails to avoid collateral damage.

## Internal vs Akamai Ownership

- `self_hosted_minimal`:
  - Shuma-native tarpit remains internal-first.
- `enterprise_akamai`:
  - Edge signals may trigger tarpit eligibility.
  - No stable external tarpit API target today; keep execution internal.

## Proposed Architecture

### A. Tarpit modes

- `off`, `drip`, `drip_plus_escalate` modes.
- Risk-tier gating to prevent accidental broad activation.

### B. Budget controls

- Global concurrent tarpit cap.
- Per-IP/UA bucket cap.
- Max bytes and max duration per response.

### C. Deterministic fallback

- On budget exhaustion: fallback by policy (`challenge` then `block`, or `maze` then `block`).
- Emit explicit fallback reason codes.

### D. Escalation policy

- Persistent tarpit traversals can escalate to temporary ban with guardrails.
- Add revalidation windows before long-duration enforcement.

### E. Maze/Tarpit shared primitives (must be reused)

- Reuse the same signed deception token envelope defined for maze traversal (`MZ-2` scope).
- Reuse a single budget governor for concurrency, byte/time caps, and per-bucket spend (`MZ-7` scope).
- Reuse one deterministic fallback policy matrix across both modes.
- Reuse one observability taxonomy so maze and tarpit metrics are directly comparable.

Non-duplication rule:

- `TP-1` through `TP-4` must consume shared primitives used by maze work; do not implement tarpit-only token, budget, or fallback subsystems.

## Cost-allocation Targets

| Cost component | Target bearer | Approach |
| --- | --- | --- |
| Connection hold time | Bot primary | Slow-drip response pacing |
| Bandwidth overhead | Bot primary | Extended traversal and controlled drip |
| Scheduler cost | Host bounded | Hard concurrency caps and timeouts |
| Memory/state | Host bounded | TTL-limited lightweight tracking |

## Rollout Strategy

1. Build tarpit core with strict default-off posture.
2. Enable for tiny high-confidence cohorts.
3. Tune caps and fallback thresholds with saturation telemetry.
4. Add escalation only after false-positive confidence is strong.

## Structured Implementation TODOs

1. TP-C1: Reuse shared deception token and budget primitives from maze scope (`MZ-2`, `MZ-7`) before tarpit mode implementation.
2. TP-C2: Confirm shared fallback policy matrix and shared observability schema are used by both modes.
3. TP-1: Implement `maze_plus_drip` tarpit mode and response pacing using shared primitives.
4. TP-2: Add global and per-bucket concurrency budgets via shared budget governor.
5. TP-3: Add max byte and max duration caps via shared budget governor.
6. TP-4: Add deterministic fallback action policy via shared fallback matrix.
7. TP-5: Add tarpit activation and saturation metrics.
8. TP-6: Add escalation policy for repeated tarpit clients.
9. TP-7: Add guardrails for false-positive minimization.
10. TP-8: Add integration tests for budget exhaustion and fallback paths.
11. TP-9: Add observability dashboards for tarpit cost attribution.
12. TP-10: Document operational runbook and emergency disable path.

## Enterprise Offering Snapshot (Akamai and Cloudflare)

- Akamai:
  - Akamai's adversarial-bot handling guidance includes tarpit-style response actions in edge mitigation strategy.
  - This creates a viable enterprise signal/trigger source, but the behavior model is provider-specific and not a portable contract.
- Cloudflare:
  - Cloudflare provides strong challenge and rate-limiting primitives, but no equivalent first-class "tarpit mode" control surface for app-layer deception pacing.
- Planning implication:
  - Keep tarpit execution internal so response pacing, budgets, and fallback semantics stay deterministic across personas.
  - In enterprise mode, treat edge outcomes as eligibility inputs for internal tarpit activation.

## Source References

- https://doi.org/10.1016/j.cose.2008.07.004
- https://doi.org/10.1016/j.comnet.2010.05.002
- https://doi.org/10.3390/electronics10172105
- https://techdocs.akamai.com/cloud-security/docs/handle-adversarial-bots
- https://developers.cloudflare.com/cloudflare-challenges/challenge-types/challenge-pages/
- https://developers.cloudflare.com/waf/rate-limiting-rules/
