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

- Keep defense as frictionless as possible for humans and tolerated bots.
- Make malicious bot behavior progressively more expensive.
- Prefer asymmetric designs where attacker cost rises faster than defender cost.
- Prioritize resource efficiency (bandwidth, CPU, memory, energy).
- Keep architecture modular and platform-agnostic.

## Required workflow for non-trivial changes

1. Read relevant docs and touched modules before editing.
2. When acting on TODO items, scan the full TODO backlog first (`todos/todo.md` and `todos/security-review.md`) to identify intersecting items and avoid duplicate or conflicting work.
3. Make small, reviewable changes.
4. Add/update tests for behavior changes.
5. Update docs for behavior/config/ops changes.
6. Run verification through `Makefile` targets only:
   - `make test` as the umbrella verification path (unit + integration + dashboard e2e),
   - `make test-unit`, `make test-integration`, and `make test-dashboard-e2e` for focused reruns,
   - `make build` for release build verification,
   - `make setup`/`make verify` when environment/bootstrap behavior is touched.
   Direct ad-hoc tool invocations (for example `cargo test`, `cargo build`, `spin up`) are not the canonical verification path for normal contributor/agent workflow.
   For `make test`, integration and dashboard e2e tests are mandatory and must not be skipped: start Spin first with `make dev` (separate terminal/session), then run `make test`.
7. Before reporting completion, confirm relevant CI status (or state explicitly that CI is pending/unverified).
8. Commit/push in atomic slices by default:
   - one logical change per commit,
   - avoid mixing unrelated refactors and feature/bug work in the same commit,
   - run relevant Makefile verification before each commit,
   - push after each validated atomic commit unless the user explicitly asks for batching.
9. Document security, operational, and resource implications.

## Security and abuse posture

- Default to secure behavior and explicit hardening guidance.
- Do not weaken auth, trust-boundary checks, or monitoring visibility.
- Prefer low-cost passive signals before expensive interactive friction for likely humans.

## Architecture and boundaries

- Respect module boundaries documented in `docs/module-boundaries.md`.
- Use ADRs (`docs/adr/0000-template.md`) for cross-cutting architecture/security/ops decisions.
- Keep compatibility shims temporary and remove when migrations complete.
- Use descriptive Rust module/file naming: prefer clear, responsibility-revealing `snake_case` names (for example `request_validation.rs`, `browser_user_agent.rs`) over vague names.
- Prefer explicit module files (`foo.rs`) over opaque `mod.rs` for new work when practical; keep directory + filename understandable without opening the file.

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
