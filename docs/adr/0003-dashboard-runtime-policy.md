# 0003 Dashboard Runtime Policy (Svelte Excellence)

- Status: Accepted
- Date: 2026-02-18
- Extends: `docs/adr/0002-dashboard-sveltekit-cutover.md`

## Context

The SvelteKit cutover is complete and pre-launch constraints are explicit:

1. No backward DOM-ID compatibility layer is required.
2. No multi-instance dashboard-runtime guarantees are required yet.
3. Dashboard runtime cleanup should favor smaller, explicit modules and behavior-first tests.

## Decision

Adopt the following dashboard runtime policy:

1. Keep one canonical Svelte runtime path and delete superseded wrappers/glue.
2. Enforce single-instance runtime assumptions in local/client architecture.
3. Prefer behavior/outcome contracts (session, tab flow, config persistence, monitoring rendering) over legacy structural contracts.
4. Require architecture gates in tests for:
   - dead module detection,
   - coordinator size budget,
   - banned legacy runtime globals.
5. Keep runtime path/base handling SvelteKit-base-aware (no hard-coded template asset/login routes).

## Consequences

### Positive
1. Faster iteration with less migration-era code drag.
2. Lower maintenance risk from dead wrapper abstractions.
3. Clear policy boundaries for future contributors and agents.

### Trade-offs
1. No compatibility promises for removed wrapper paths.
2. If multi-instance behavior becomes in-scope, new ADR work is required.

## Security impact

1. No trust-boundary weakening; auth/session checks remain server-side.
2. Reduced legacy glue lowers accidental bypass/regression surface.

## Operational impact

1. Makefile verification remains canonical (`make test`, `make build`, `make setup`, `make verify`).
2. Test gates now block dead modules and oversized coordinator growth.

## Rollback

1. Revert offending runtime-policy commits.
2. Re-run `make test` and `make build` before deploy.
