# Rate Limiting Excellence Plan

Date: 2026-02-13
Status: Proposed

## Context

Current rate limiting is fixed-window KV-based and non-atomic across distributed instances. This is acceptable for basic deployments but insufficient for high-scale multi-instance correctness and coordinated abuse control.

## Goals

- Provide atomic, distributed rate limiting semantics.
- Separate signal contribution from hard enforcement cleanly.
- Keep deterministic fallback under backend degradation.

## Non-goals

- Overly complex global control loops that sacrifice predictability.
- Replacing all local controls with external-only enforcement.

## State-of-the-art Signals

1. Distributed limiters need atomic operations and explicit outage posture.
2. Low-rate attack patterns can evade naive threshold-only controls.
3. Multi-window or token-bucket strategies outperform single-window-only approaches.

## Internal vs Akamai Ownership

- `self_hosted_minimal`:
  - Internal limiter remains default.
- `enterprise_akamai`:
  - Edge limiter can be authoritative for first-pass control.
  - Shuma enforces app-context limits and fallback behavior.

## Proposed Architecture

### A. Distributed limiter backend

- Add atomic counter backend support (for example Redis INCR/Lua semantics).
- Keep local KV fallback with explicit degraded behavior.

### B. Policy model

- Use combined windows and burst controls.
- Keep separate paths for admin auth vs general traffic.

### C. Signal + enforcement split

- Maintain rate-pressure as scoring signal.
- Enforce hard limits as barrier action with explicit detection IDs.

### D. Outage and failover posture

- Define `fail-open` vs `fail-closed` per route class.
- Expose limiter health and fallback usage metrics.

## Rollout Strategy

1. Introduce distributed backend in shadow-read mode.
2. Validate parity against internal limiter counters.
3. Enable hard enforcement per route class.
4. Enable enterprise edge-authoritative interplay after soak.

## Structured Implementation TODOs

1. RL-1: Implement atomic distributed limiter adapter.
2. RL-2: Add multi-window/burst policy primitives.
3. RL-3: Split admin auth limiter and main traffic limiter policies.
4. RL-4: Add explicit outage posture config by route class.
5. RL-5: Add fallback-to-internal behavior and tagging.
6. RL-6: Add limiter health and drift metrics.
7. RL-7: Add advisory vs authoritative edge precedence tests.
8. RL-8: Add low-rate attack simulation regression tests.
9. RL-9: Add operator runbook for threshold tuning and rollback.
10. RL-10: Document distributed-state SLOs and alerts.

## Enterprise Offering Snapshot (Akamai and Cloudflare)

- Akamai:
  - Application Security APIs expose first-class rate-policy resources (`/appsec/v1/rate-policies`) for centralized edge-side enforcement.
  - App and API Protector Hybrid has added app-layer rate limiting support, reinforcing enterprise-first edge control patterns.
- Cloudflare:
  - Rate Limiting Rules support multiple counting characteristics and enterprise controls, including bot-score-aware expressions and JA3/JA4 characteristics.
  - Cloudflare documents that counters are not globally shared between all PoPs, which matters for strict global correctness assumptions.
- Planning implication:
  - Keep Shuma distributed limiter work (atomic backend + outage posture) as the correctness anchor for app-context decisions.
  - In enterprise profiles, use edge rate limiting as first-pass shedding while preserving internal fallback and drift monitoring.

## Source References

- https://www.microsoft.com/en-us/research/publication/cloud-control-with-distributed-rate-limiting/
- https://doi.org/10.1145/863955.863966
- https://doi.org/10.1016/j.comnet.2010.05.002
- https://doi.org/10.3390/electronics10172105
- https://techdocs.akamai.com/cloud-security/docs/handle-adversarial-bots
- https://techdocs.akamai.com/application-security/reference/get-rate-policies
- https://techdocs.akamai.com/application-security/reference/post-rate-policy
- https://techdocs.akamai.com/app-api-protector/changelog/release-2024-11-27
- https://developers.cloudflare.com/waf/rate-limiting-rules/
- https://developers.cloudflare.com/waf/rate-limiting-rules/parameters/#counting-characteristics
- https://developers.cloudflare.com/waf/rate-limiting-rules/request-rate/
