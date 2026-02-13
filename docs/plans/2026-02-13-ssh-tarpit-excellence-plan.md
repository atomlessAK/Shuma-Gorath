# SSH Tarpit Excellence Plan

Date: 2026-02-13
Status: Proposed

## Context

SSH tarpit/honeypot capability is not implemented in the current Shuma HTTP runtime. If added, it should be treated as a separate infrastructure component with explicit boundaries, not mixed into core HTTP request handling.

## Goals

- Impose cost on automated SSH abuse traffic.
- Capture high-value threat intelligence without exposing production SSH services.
- Keep operational blast radius low via strict isolation.

## Non-goals

- Embedding SSH protocol handling directly inside the primary web defence runtime.
- Building a full general-purpose honeynet platform.

## State-of-the-art Signals

1. Honeypots are fingerprintable when protocol behavior is unrealistic.
2. Adaptive and hybrid interaction tiers improve intelligence quality.
3. Anti-fingerprinting realism is required to avoid rapid attacker filtering.

## Internal vs Akamai Ownership

- `self_hosted_minimal`:
  - Optional standalone SSH tarpit component managed by operator.
- `enterprise_akamai`:
  - Akamai remains relevant for HTTP layer, not native SSH tarpit execution.
  - Shuma can ingest SSH risk outputs as external signals for cross-channel policy.

## Proposed Architecture

### A. Deployment boundary

- Run SSH tarpit as isolated service with separate keys/network policy.
- Export only structured events into Shuma policy pipeline.

### B. Interaction tiers

- Tier 1: lightweight protocol-delay and banner decoy.
- Tier 2: constrained interactive decoy session.
- Tier 3: deep interaction reserved for high-value intelligence environments.

### C. Anti-fingerprinting realism

- Vary protocol timing and implementation signatures within safe bounds.
- Avoid static, obviously synthetic handshake patterns.

### D. Safety controls

- Strict egress restrictions and command sandboxing.
- Data minimization and retention limits for captured artifacts.

## Rollout Strategy

1. Define isolated deployment model and event schema.
2. Launch passive intelligence mode first.
3. Add selective tarpit tiers for repeat automated abuse.
4. Integrate cross-channel enrichment into Shuma scoring.

## Structured Implementation TODOs

1. SSH-1: Define standalone SSH tarpit service boundary and interfaces.
2. SSH-2: Define event schema for Shuma ingestion.
3. SSH-3: Implement tiered interaction modes with safety caps.
4. SSH-4: Add protocol realism variation controls.
5. SSH-5: Add anti-fingerprinting regression checks.
6. SSH-6: Add strict sandbox/egress controls and audit trails.
7. SSH-7: Add retention and privacy guardrails for captured data.
8. SSH-8: Add Shuma-side correlation of SSH signals with HTTP abuse signals.
9. SSH-9: Add deployment runbook for isolated hosting.
10. SSH-10: Add incident rollback and emergency disable procedures.

## Source References

- https://doi.org/10.1145/3584976
- https://doi.org/10.1109/SoutheastCon51012.2023.10115143
- https://doi.org/10.3390/app12105224
- https://doi.org/10.23919/TMA66427.2025.11097018
