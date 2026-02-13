# 🐙 Bot Defence Layering (Shuma-Gorath + Managed Edge Defences)

## 🐙 Purpose

This document explains how to position Shuma-Gorath in a layered defence stack.

Shuma-Gorath is application-focused bot defence logic you control directly.
Managed providers (for example Akamai Bot Manager) provide broad edge-scale detection and mitigation.
Used in concert, and sensibly configured, they limit human friction while increasing attacker cost.

## 🐙 Core Positioning

- Shuma-Gorath is the app-specific control plane:
  - business-logic aware traps and routing,
  - configurable challenge/maze/tarpit behavior,
  - transparent, auditable policy and telemetry,
  - fast iteration for site-specific abuse patterns.
- Managed edge bot defences are the global control plane:
  - large-scale edge telemetry,
  - reputation and behavioral models,
  - mature bot-category policy controls,
  - globally distributed enforcement.

## 🐙 Product Stance (2026-02-13)

- Default posture: self-hosted minimal infrastructure must be first-class and complete.
  Every core defence remains internally available, toggleable, and tweakable.
- Enterprise integration focus: prioritize Akamai-compatible ingestion and control surfaces.
- Runtime posture: keep Fermyon/Spin as a primary deployment path.
- Decision authority:
  - advisory mode (default): managed-edge outcomes are signals for Shuma policy.
  - authoritative mode (optional): managed-edge outcomes can short-circuit selected local paths.
- Boundary rule:
  externalize state and upstream signals where edge providers have superior reach,
  keep Shuma-owned policy composition (botness scoring, escalation routing, module mode composition) internal.

## 🐙 Architectural Ownership Policy (2026-02-13)

This policy is the default for `enterprise_akamai` planning and implementation. Any exception should be documented in a plan/ADR before implementation.

### Leaning Akamai (especially in `enterprise_akamai`)

- Fingerprint signal ingestion (edge transport/bot telemetry is strongest there).
- Rate limiting first-pass enforcement at the edge (with Shuma app-context fallback/control).
- IP range policy enforcement for managed network blocks/reputation sets.
- GEO attribution/enforcement when edge geo confidence is higher.

### Leaning Internal (Shuma-native)

- Puzzle challenge engine.
- Challenge Lite scoring/verification flow.
- PoW issuance/verification logic.
- CDP detection execution/scoring.
- HTTP tarpit behavior.
- SSH tarpit (as a separate Shuma-managed component).
- Maze/deception internals (already planned as internal-first).

### Hybrid (Akamai informs, Shuma decides)

- JS verification trigger policy.
- CDP corroboration.
- Challenge escalation routing.
- Overall botness/policy orchestration.

## 🐙 Where Shuma-Gorath Adds Unique Value (Not Just Duplication)

When Akamai Bot Manager (or equivalent) is already in place, Shuma-Gorath should focus on controls that are highly application-specific and hard to get from generic edge policy alone:

- App-workflow abuse controls:
  - route- and action-specific guardrails tied to your own business semantics,
  - policy decisions based on app context and challenge outcomes.
- Deception tailored to your app:
  - custom honeypot paths and maze/tarpit flows that match your URL and behavior patterns.
- Operator-owned enforcement composition:
  - explicit, auditable routing between challenge, maze, tarpit, and block paths under your own policy model.
- Fast app-team iteration:
  - change behavior in your own code/config/admin flow without waiting for vendor model/policy cycles.
- Transparent, self-hosted observability:
  - event and policy telemetry aligned to your own incident response and tuning workflow.

## 🐙 How They Work Together

Typical layered order:

1. Managed edge bot defence evaluates and filters obvious automation first.
2. Requests that pass edge controls are evaluated by Shuma-Gorath for app-specific abuse.
3. Origin only receives traffic that passed both layers.

This model keeps high-volume commodity bot traffic away from app paths while preserving local control for nuanced abuse.

## 🐙 Capability Split (Practical Focus)

| Focus Area | Managed Edge Bot Defence (e.g., Akamai Bot Manager) | Shuma-Gorath Focus |
|---|---|---|
| Global bot classing and reputation | Primary | Consume outcome, avoid re-implementing globally |
| App-specific workflow abuse controls | Limited | Primary |
| Custom deception (honeypot/maze/tarpit) | Limited/variable | Primary |
| Policy-code ownership and extensibility | Limited | Primary |
| Per-app rapid tuning loop | Shared | Primary in app layer |

## 🐙 Platform Scope Boundaries (Akamai vs Shuma)

Use this section to avoid overreach and duplicated controls.

| Capability Family | System of Record | Why | Shuma Role | Explicit Non-Goal |
|---|---|---|---|---|
| Global reputation, bot category models, and internet-scale classifier training | Akamai (managed edge) | Edge has cross-tenant/global telemetry and model lifecycle tooling | Ingest outputs as advisory/authoritative signals where configured | Rebuilding global bot classifier training inside Shuma |
| Transport/network identity signals (for example JA3/JA4-like fingerprints, ASN reputation, network provenance) | Akamai (managed edge) | Edge sees handshake/network context origin apps usually do not | Normalize and consume trusted upstream signals in policy | Duplicating edge-only transport fingerprint collection logic in app runtime |
| Volumetric pre-filtering and broad perimeter suppression | Akamai (managed edge) | Best placed before app resource spend | Assume pre-filtered traffic and apply app-aware controls | Treating Shuma as a DDoS/perimeter replacement |
| App workflow abuse controls (route/action semantics, business logic checks) | Shuma | Requires application context and product semantics | Primary policy and enforcement ownership | Delegating all app-abuse policy design to generic edge controls |
| Deception controls (honeypot/maze/tarpit) | Shuma | Needs app-specific URL/content shaping and local tuning | Primary design and operation | Waiting for edge vendor parity before shipping app-specific deception |
| Challenge escalation composition (when to challenge/maze/block) | Shuma policy engine | Requires local thresholds, observability, and rollback ownership | Keep policy composition internal even with external signals | Outsourcing core policy/routing composition authority |
| Security/ops observability for app decisions | Shuma + platform observability | Operators need explainable app-level outcomes and rollback evidence | Emit provider/mode/signal-state telemetry and event context | Relying only on edge dashboards for app-level incident response |

### Boundary Decision Rule

Prefer implementation in Shuma only when at least one is true:

- the control depends on application-specific semantics unavailable at the edge,
- self-hosted minimal mode must support the feature without managed-edge dependency,
- operators need direct local policy ownership and explainability not provided by upstream controls.

Prefer implementation at the edge when at least one is true:

- the capability relies on authoritative network/transport vantage unavailable to origin runtime,
- global state/reputation quality materially exceeds what local app-only data can provide,
- placing the control at edge materially reduces origin resource spend with acceptable explainability.

## 🐙 Shuma-Gorath Feature Fit in a Layered Model

Shuma-Gorath remains valuable even behind a strong edge bot product:

- Honeypot and trap paths tuned to your routes.
- Risk scoring from in-app context (headers, routes, challenge outcomes).
- Configurable routing to challenge, maze, tarpit, or block actions.
- Self-hosted observability and event logs aligned to your ops model.

## 🐙 Deployment Patterns

### Pattern A: Edge-managed first, Shuma-Gorath second (recommended for enterprise)

```
Internet
  ↓
Managed Edge Bot Defence
  ↓
Shuma-Gorath (Spin component)
  ↓
Origin
```

### Pattern B: Shuma-Gorath standalone

```
Internet
  ↓
Shuma-Gorath (Spin component)
  ↓
Origin
```

Pattern B is viable for smaller estates or pre-launch stages.
Pattern A is generally stronger for high-risk/high-volume environments.

## 🐙 Practical Operating Guidance

- Keep high-confidence commodity bot blocking at the edge.
- Use Shuma-Gorath for app-specific workflow protections and deception flows the edge layer does not natively model.
- Avoid duplicate friction: if the edge layer already challenged, keep Shuma defaults conservative.
- Monitor challenge rates and false positives at both layers.
- Treat Shuma as the place for business-context logic, not as a clone of global bot classification.
- Use advisory integration as the default enterprise posture; enable authoritative overrides only when needed and observable.

## 🐙 Current and Future Threat Model Alignment

Shuma-Gorath roadmap should continue focusing on:

- low-friction human-passive signals,
- progressive cost imposition for suspicious automation,
- modular barriers and signal composition,
- resource-efficient controls with asymmetric attacker cost.

## 🐙 Notes on Edge Runtime

- Akamai Bot Manager is an edge-delivered bot mitigation product.
- Shuma-Gorath can run wherever Spin is deployed.
- That includes edge deployment models when Spin workloads are deployed on edge-capable platforms.
- Planned distributed edge-state work (for example ban/rate sync) narrows operational overlap further by improving cross-edge consistency in Shuma-managed controls.
