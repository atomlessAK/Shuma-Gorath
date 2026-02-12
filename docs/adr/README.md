# Architecture Decision Records (ADR)

Use ADRs to capture significant architectural or cross-cutting decisions.

## When an ADR is required

Create an ADR when a change:

- Introduces or changes module boundaries.
- Adds/removes provider interfaces or pluggability behavior.
- Changes security posture or trust boundaries.
- Changes deployment/operational model in a non-trivial way.
- Adds a new high-cost defense mechanism with trade-offs.
- Breaks or intentionally alters established behavior.

## Naming and location

- Location: `docs/adr/`
- File format: `NNNN-short-title.md` (example: `0001-rate-limiter-backend.md`)
- Start from template: `docs/adr/0000-template.md`

## Status lifecycle

- `Proposed`
- `Accepted`
- `Superseded`
- `Deprecated`

If superseded, link the replacing ADR.

## Writing rules

- Keep it concise.
- Focus on context, decision, and consequences.
- Include alternatives considered.
- Include security, operational, and resource implications.
- Link to relevant PRs/issues.
