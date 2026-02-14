# Maze/Tarpit Research Synthesis Gate (MZ-R1/MZ-R2/MZ-R3)

Date: 2026-02-14
Status: Proposed for Stage 2 gate acceptance (`MZ-R0`)

## Scope

This document consolidates the highest-impact research tranche required by Stage 2:

- `R-FP-10` PathMarker (path + timing provenance for crawler discrimination)
- `R-RL-02` low-rate TCP-targeted DoS (shrew/pulse behavior)
- `R-RL-04` defenses for low-rate application-server DoS
- `R-RL-08` low-overhead low-rate DoS detection at transport/application layers
- `R-SSH-01` multistage honeypot fingerprinting

It translates those findings into:

- concrete maze/tarpit design requirements (`MZ-R1`),
- ownership mapping for `self_hosted_minimal` vs `enterprise_akamai` (`MZ-R2`),
- enforceable guardrails and acceptance criteria before coding Stage 2 slices (`MZ-R3`).

## MZ-R1: Research Findings and Implications

### R-FP-10 PathMarker (2019)

Key findings:

- URL markers carry parent-page provenance and user identity, enabling request-path attribution.
- Combined path-shape and timing features improve crawler discrimination, including distributed crawler link sharing.
- Marker binding also suppresses crawler efficiency by forcing duplicate work when links are not reusable across identities.

Shuma implications:

- Treat maze traversal as a signed provenance chain, not independent links.
- Include parent/previous-node binding in traversal tokens.
- Keep path-order and timing checks first-class detection inputs (`D_SEQ_*` / `S_SEQ_*`) during deception flows.
- Design traversal so shared/replayed links collapse attacker efficiency.

### R-RL-02 Low-Rate TCP-Targeted DoS (SIGCOMM 2003)

Key findings:

- Periodic low-rate bursts timed near TCP retransmission timeout can severely degrade victim throughput while evading simple rate-based detectors.
- Attack effectiveness depends on protocol timing predictability.
- Timeout randomization may reduce severity but does not eliminate the attack class.

Shuma implications:

- Do not depend on traffic volume/rate alone for bot and tarpit decisions.
- Add cadence/regularity and ordering windows as durable primitives in deception decisions.
- Avoid deterministic, globally predictable emission schedules for expensive response modes.

### R-RL-04 Defense Techniques for Low-Rate DoS Against Application Servers (2010)

Key findings:

- Effective defenses focus on request-acceptance control and queue protection.
- Two core themes: prevent queue-slot monopolization and randomize server behavior to reduce attacker timing precision.
- Tradeoffs must be measured against legitimate-user impact and operational overhead.

Shuma implications:

- Introduce a single shared budget governor for maze and tarpit with explicit concurrency and per-bucket limits.
- Reserve deterministic fallback when budgets saturate to prevent self-DoS.
- Apply bounded randomization where it reduces timing predictability without harming humans.

### R-RL-08 Detection of Low-Rate DoS at Transport/Application Layers (2021)

Key findings:

- Low-overhead detection can be viable with very small feature sets (for example inter-arrival/timing/entropy style features), avoiding heavy per-flow packet tracking.
- Two-stage detection (coarse + refinement) can support near-real-time operation.

Shuma implications:

- Prefer cheap, bounded per-request signal primitives over heavyweight deep inspection.
- Keep Stage 2 signal checks constant-time where possible (token verify + bounded replay lookup + threshold checks).
- Use staged escalation (`instrument -> advisory -> enforce`) to tune false positives before hard enforcement.

### R-SSH-01 Multistage Honeypot Fingerprinting (2023)

Key findings:

- Honeypots are practically fingerprintable at scale when behavior/configuration is static or default-heavy.
- Multistage probing can identify decoys with low false-positive probability.
- Countermeasure direction: reduce static artifacts and increase realism/diversity over time.

Shuma implications:

- Static maze pages and default-only structures are fingerprint liabilities.
- Stage 2 must ship rotating entropy + polymorphic variants as baseline, not optional polish.
- Detection confidence should combine multiple independent checks before irreversible deny actions.

## MZ-R2: Persona Ownership Mapping

### `self_hosted_minimal` (internal-first mandatory)

Shuma-owned and required:

- rotating signed maze entropy (`MZ-1`),
- signed traversal token envelope + replay window (`MZ-2`),
- shared maze/tarpit budget governor + deterministic fallback (`MZ-7` / `TP-C2`),
- client-checkpoint policy and no-JS fallback semantics (`MZ-5`),
- polymorphic rendering + variant selection (`MZ-3`),
- deception telemetry and botness feedback wiring (`MZ-9`).

Not required in this persona:

- external provider APIs for maze generation or tarpit execution,
- distributed global counters as a prerequisite for baseline correctness.

### `enterprise_akamai` (augment, do not replace)

Akamai/edge can contribute:

- trusted upstream suspicion/fingerprint outcomes as additional routing signals,
- broader perimeter filtering before Shuma maze/tarpit activation,
- optional distributed-state backends for cross-instance budget coherence.

Shuma remains system of record for:

- maze/tarpit token semantics,
- fallback policy matrix,
- escalation composition and local explainability.

## MZ-R3: Enforceable Stage 2 Guardrails

These are pre-implementation constraints for `MZ-1`, `MZ-2`, `MZ-7`, and dependent slices.

### Guardrail A: Cost and Resource Bounds (must be hard caps)

- Global concurrent maze/tarpit responses must be capped.
- Per-IP-bucket concurrency/spend must be capped.
- Per-response byte and duration caps must be enforced.
- Budget exhaustion must trigger deterministic fallback (`challenge` then `block`, or `maze` then `block`) via one shared matrix.

Initial target defaults for implementation proposals:

- `max_concurrent_global`: `128`
- `max_concurrent_per_ip_bucket`: `4`
- `max_response_bytes`: `65536`
- `max_response_duration_ms`: `15000`

### Guardrail B: Anti-Fingerprint Minimum Bar

- No globally stable suspicious-traffic maze page per URL.
- Variant selection must include a rotating entropy input and a versioned selector.
- At least three independent variant dimensions are required: layout, lexical corpus, link-graph shape.

### Guardrail C: Traversal Integrity and Replay Safety

- Every traversal step must carry a signed token with `issued_at`, `expires_at`, `depth`, `branch_budget`, and previous-node binding.
- Replay cache must be bounded (TTL + size policy), with explicit behavior on overflow.
- Expired, replayed, or binding-mismatched tokens must map to canonical taxonomy signals/detections.

Initial target defaults for implementation proposals:

- `token_ttl_seconds`: `90`
- `token_max_depth`: `8`
- `token_branch_budget`: `3`
- `replay_ttl_seconds`: `600`

### Guardrail D: Checkpoint Cadence and Step-Ahead Bound

- Suspicious-tier client expansion must checkpoint every 3 nodes or every 1500 ms, whichever is first.
- Server must allow at most 4 unverified steps ahead.
- Missing/late/invalid checkpoints must trigger deterministic fallback, not unbounded server continuation.

### Guardrail E: Rollout Abort Thresholds

Stage 2 enforcement rollout must automatically pause or roll back if any hold condition is sustained for 10 minutes:

- maze/tarpit budget saturation above `2%` of eligible suspicious requests,
- protected-route p95 latency regression above `20%` versus baseline,
- non-2xx/3xx response-rate increase above `0.5%` absolute on protected routes,
- challenge abandonment or operator-defined human-success metrics degrade beyond pre-set bounds.

## Stage 2 Gate Acceptance Checklist (`MZ-R0`)

Before starting `MZ-1` implementation, confirm all items below:

1. Research synthesis accepted by maintainers (this document).
2. Ownership mapping approved for both personas without changing internal-first policy.
3. Guardrail defaults accepted or replaced with explicit alternatives.
4. Rollout abort thresholds documented in operator runbook targets.
5. Stage 2 TODO order remains `MZ-R0 -> MZ-R1 -> MZ-R2 -> MZ-R3 -> MZ-1 ...`.

## Source References

- PathMarker (Cybersecurity, 2019): https://cybersecurity.springeropen.com/articles/10.1186/s42400-019-0023-1
- Low-Rate TCP-Targeted DoS Attacks (SIGCOMM 2003): https://doi.org/10.1145/863955.863966
- Defense techniques for low-rate DoS attacks against application servers (Computer Networks, 2010): https://doi.org/10.1016/j.comnet.2010.05.002
- On the Detection of Low-Rate Denial of Service Attacks at Transport and Application Layers (Electronics, 2021): https://doi.org/10.3390/electronics10172105
- Gotta catch 'em all: A Multistage Framework for Honeypot Fingerprinting (Digital Threats, 2023): https://doi.org/10.1145/3584976
