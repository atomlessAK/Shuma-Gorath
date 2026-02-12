# AGENTS.md

This file provides instructions for coding agents working in this repository.

## Scope and precedence

1. Follow explicit user instructions first.
2. Follow this file next.
3. Follow canonical project policy docs:
   - `docs/project-principles.md`
   - `CONTRIBUTING.md`
   - `docs/adr/README.md`
4. If instructions conflict, preserve security, correctness, and principle alignment.

## Core operating goals

- Keep defense as frictionless as possible for legitimate humans.
- Make malicious bot behavior progressively more expensive.
- Prefer asymmetric designs where attacker cost rises faster than defender cost.
- Prioritize resource efficiency (bandwidth, CPU, memory, energy).
- Keep architecture modular and platform-agnostic.

## Required workflow for non-trivial changes

1. Read relevant docs and touched modules before editing.
2. Make small, reviewable changes.
3. Add/update tests for behavior changes.
4. Update docs for behavior/config/ops changes.
5. Run verification through `Makefile` targets only:
   - `make test-unit` / `make test` for Rust verification,
   - `make build` for release build verification,
   - `make setup`/`make verify` when environment/bootstrap behavior is touched.
   Direct ad-hoc tool invocations (for example `cargo test`, `cargo build`, `spin up`) are not the canonical verification path for normal contributor/agent workflow.
6. Document security, operational, and resource implications.

## Security and abuse posture

- Default to secure behavior and explicit hardening guidance.
- Do not weaken auth, trust-boundary checks, or monitoring visibility.
- Prefer low-cost passive signals before expensive interactive friction for likely humans.

## Architecture and boundaries

- Respect module boundaries documented in `docs/module-boundaries.md`.
- Use ADRs (`docs/adr/0000-template.md`) for cross-cutting architecture/security/ops decisions.
- Keep compatibility shims temporary and remove when migrations complete.

## Pull request expectations

Ensure PR descriptions address:

- human visitor impact (friction/latency/challenge rate),
- adversary cost asymmetry and cost placement,
- resource impact,
- monitoring impact,
- rollback plan for risky changes.

Use `.github/pull_request_template.md`.

## Notes for agent tooling

- `AGENTS.md` is a convention, not a universal standard across all tools.
- If your tooling does not auto-read this file, treat `CONTRIBUTING.md` and `docs/project-principles.md` as mandatory equivalents.
