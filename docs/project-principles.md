# Shuma-Gorath Project Principles

## Purpose

Shuma-Gorath exists to provide layered, practical bot defense that teams can deploy quickly, operate safely, and evolve without lock-in, while staying as frictionless as possible for legitimate humans and increasingly costly for malicious bots.

## Core Goals

1. Deliver full-featured bot defense with multiple control layers.
2. Keep defenses as invisible and frictionless as possible for legitimate visitors.
3. Make malicious bot traffic progressively more expensive to operate.
4. Keep local setup and developer workflows simple and fast.
5. Keep deployment simple and platform-agnostic.
6. Keep testing straightforward, reliable, and part of normal development.
7. Keep documentation clear, complete, and maintained as code changes.
8. Keep security as a default, not an optional add-on.
9. Keep configuration explicit, safe, and easy to understand.
10. Keep monitoring and operational visibility first-class.
11. Keep architecture modular, extensible, and provider-swappable where practical.
12. Keep feature design informed by current, credible techniques.
13. Minimize bandwidth, compute, and energy for desired protection outcomes, and shift unavoidable costs toward bot operators where possible.

## Engineering Principles (MUST/SHOULD)

### P1. Human-Invisible, Bot-Costly Defense

- MUST minimize added friction for legitimate human visitors.
- MUST escalate defenses by confidence so attacker cost rises with suspicious behavior.
- MUST prefer asymmetric designs where attacker resource cost grows faster than defender cost.
- SHOULD prefer passive/covert detection before interactive challenges for likely humans.
- SHOULD track human-friction and attacker-cost indicators in observability.

### P2. Layered Defense

- MUST apply defense-in-depth (signals, policy, enforcement, challenge, deception/tarpit where applicable).
- MUST keep signal collection and enforcement actions as distinct responsibilities, even when implemented in the same module.
- MUST model hybrid defenses (for example rate controls) with explicit signal and enforcement paths.
- MUST preserve safe defaults.
- SHOULD let operators tune policy by confidence and blast radius.

### P3. Simplicity of Setup, Dev, and Deploy

- MUST provide one canonical workflow for setup, run, and test (`Makefile` targets).
- MUST run setup/build/test verification through `Makefile` targets (`make setup`, `make build`, `make test`/`make test-unit`) so shared workflows fail fast when broken.
- MUST treat direct tool invocations (`cargo`, `spin`, script entrypoints) as implementation details behind `make`, not as the default contributor/agent interface.
- MUST avoid hidden prerequisites.
- SHOULD keep dev/prod parity high.

### P4. Security by Default

- MUST fail closed for authz/authn mistakes.
- MUST keep safety-critical controls non-disableable by composability toggles (for example admin auth and trusted health restrictions).
- MUST document operational hardening required in production.
- MUST log security-relevant failures with enough detail for diagnosis.
- SHOULD favor least privilege in runtime, CI, and admin controls.

### P5. Testability and Verification

- MUST include tests for behavior changes and regressions.
- MUST keep CI as the minimum quality bar for merge.
- SHOULD keep tests colocated with modules when possible and add integration tests for cross-module behavior.

### P6. Clarity and Documentation

- MUST update docs when behavior, config, or operations change.
- MUST keep docs discoverable from the docs index and README.
- SHOULD include rationale for non-obvious decisions.

### P7. Platform-Agnostic Core

- MUST keep core policy logic decoupled from platform adapter details.
- MUST support self-hosted and enterprise deployment personas through profile-gated adapters/state backends, not persona-specific policy forks.
- MUST deliver and harden Shuma-native capability paths first; enterprise/provider-specific integrations should be additive and must not block internal baseline maturity for the same capability.
- SHOULD keep provider interfaces explicit and swappable.
- SHOULD avoid tying core behavior to a single runtime unless required for performance or safety.

### P8. Modularity and Extensibility

- MUST keep clear module boundaries and dependency direction.
- MUST route changes that touch provider-managed capabilities through provider interfaces/registry seams instead of new direct orchestration-path module calls.
- MUST make defense modules composable with explicit mode behavior (`off`, `signal`, `enforce`, `both`) where the capability supports partial operation.
- MUST ensure disabled/unavailable modules are represented explicitly in botness/signal flows (no silent zeroing).
- MUST treat cross-cutting architectural choices as ADRs.
- SHOULD provide compatibility shims only as temporary migration aids.

### P9. Observability First

- MUST expose useful metrics and logs for detection, actions, and failures.
- MUST make operational status inspectable without code changes.
- SHOULD keep monitoring guidance current with features.

### P10. Resource Efficiency

- MUST evaluate cost/benefit before adding heavier defenses.
- MUST prefer low-cost checks before expensive checks when risk allows.
- SHOULD shift unavoidable computation and bandwidth costs toward suspicious clients where safe and practical.
- SHOULD track and prevent avoidable increases in bandwidth/CPU/memory/energy usage.

### P11. Stability and Change Discipline

- MUST define lifecycle expectations for new config keys and APIs.
- MUST treat `config/defaults.env` as the canonical source of truth for `SHUMA_*` defaults and update it first when adding/changing variables.
- MUST keep variable wiring in sync across setup/seed/runtime workflows so `make setup` and `make config-seed` establish a correct baseline without manual repair.
- MUST preserve environment profile intent:
  - dev may apply explicit helper overrides for local manual config/monitoring/tuning,
  - tests must restore env/config changes they introduce and not leak mutated state,
  - production defaults must start from the most secure posture (no debug/unsafe defaults enabled by default).
- MUST provide clear migration notes when breaking behavior is unavoidable.
- SHOULD prefer additive change and compatibility windows for operationally sensitive paths.

## Decision Rubric (for new features)

For significant feature work, document:

1. User problem and abuse model.
2. Human-visitor impact (friction, latency, challenge frequency).
3. Safety/security impact.
4. Adversary economics and cost placement (who pays and how much).
5. Signal/enforcement coupling impact (signal contributions, botness effects, mode behavior when disabled).
6. Operational impact (deploy, config, monitor, rollback).
7. Performance/resource impact (bandwidth, CPU, memory, energy).
8. Modularity/provider impact.
9. Test and documentation impact.

If trade-offs are non-trivial, record an ADR under `docs/adr/`.

## How These Principles Are Enforced

- PR checklist: `.github/pull_request_template.md`
- Contribution rules: `CONTRIBUTING.md`
- Decision records: `docs/adr/`
- CI presence checks for governance files: `.github/workflows/ci.yml`

## External References That Informed This Document

- OpenTelemetry specification principles (user-driven, stable, simple, consistent): https://opentelemetry.io/docs/specs/otel/specification-principles/
- Go 1 compatibility expectations: https://go.dev/doc/go1compat
- Kubernetes deprecation and lifecycle discipline: https://kubernetes.io/docs/reference/using-api/deprecation-policy/
- Rust editions compatibility model: https://doc.rust-lang.org/book/appendix-05-editions.html
- Django design philosophies (loose coupling, explicitness, consistency): https://docs.djangoproject.com/en/5.2/misc/design-philosophies/
- Twelve-Factor operational guidance (config, logs, parity): https://12factor.net/
- Python Zen (explicitness/readability pragmatism): https://peps.python.org/pep-0020/
