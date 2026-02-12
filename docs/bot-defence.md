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
