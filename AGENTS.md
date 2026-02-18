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
2. Check `docs/plans/` for active or recent design docs relevant to the area you are changing, and align implementation with those plans unless the user explicitly overrides them.
3. When acting on TODO items, achieve full situational awareness before implementation:
   - scan the full TODO backlog first (`todos/todo.md` and `todos/security-review.md`) to identify intersecting items,
   - identify and read relevant plan documents in `docs/plans/`,
   - synchronize TODO execution with those plans so work does not duplicate, contradict, or drift from planned architecture.
4. Make small, reviewable changes.
5. Add/update tests for behavior changes.
6. Update docs for behavior/config/ops changes.
7. Run verification through `Makefile` targets only:
   - `make test` as the umbrella verification path (unit + integration + dashboard e2e),
   - `make test-unit`, `make test-integration`, and `make test-dashboard-e2e` for focused reruns,
   - `make build` for release build verification,
   - `make setup`/`make verify` when environment/bootstrap behavior is touched.
   Makefile targets are the single source of truth for setup/build/test/deploy workflows.
   Direct ad-hoc tool invocations (for example `cargo test`, `cargo build`, `corepack pnpm run ...`, `playwright test`, `node e2e/...`, `spin up`) are not the canonical path for normal contributor/agent workflow.
   If a required workflow is missing from `Makefile`, add/update the target first, then run it via `make` (do not bypass and "fix later").
   Keep docs/PR notes/user guidance aligned to `make` commands so contributors follow one documented path.
   For `make test`, integration and dashboard e2e tests are mandatory and must not be skipped: start Spin first with `make dev` (separate terminal/session), then run `make test`.
   Exception: if a change is documentation-only (`*.md` and no behavior/config/runtime code changes), do not run tests; document that verification was intentionally skipped because the slice is docs-only.
8. Before reporting completion, confirm relevant CI status (or state explicitly that CI is pending/unverified).
9. Commit/push in atomic slices by default:
   - one logical change per commit,
   - avoid mixing unrelated refactors and feature/bug work in the same commit,
   - run relevant Makefile verification before each commit,
   - push after each validated atomic commit unless the user explicitly asks for batching.
10. Document security, operational, and resource implications.
11. When a full TODO section is completed, move its completed checklist items from `todos/todo.md` to `todos/completed-todo-history.md` and preserve the original section title as a heading in the archive entry.
12. For any new `SHUMA_*` variable, follow the single-source-of-truth lifecycle:
   - define/update canonical default in `config/defaults.env` first and classify it as env-only or KV-tunable,
   - wire seeding/bootstrap paths so `make config-seed`/`make setup` produce a correct local baseline (at minimum update `scripts/config_seed.sh`, `scripts/bootstrap/setup.sh`, and `Makefile` env wiring/help as applicable),
   - keep dev-only overrides intentional for local manual config/monitoring/tuning workflows (do not silently broaden them),
   - ensure tests leave no strange state behind (restore env mutations and reset runtime config they toggle),
   - ensure production-start defaults remain secure-by-default (no debug/unsafe defaults enabled by default).

## Security and abuse posture

- Default to secure behavior and explicit hardening guidance.
- Do not weaken auth, trust-boundary checks, or monitoring visibility.
- Prefer low-cost passive signals before expensive interactive friction for likely humans.

## Architecture and boundaries

- Respect module boundaries documented in `docs/module-boundaries.md`.
- If internal feature work touches a capability covered by `src/providers/contracts.rs`, route the change through the provider interface and registry path rather than adding new direct module calls.
- Use ADRs (`docs/adr/0000-template.md`) for cross-cutting architecture/security/ops decisions.
- Keep compatibility shims temporary and remove when migrations complete.
- Use descriptive Rust module/file naming: prefer clear, responsibility-revealing `snake_case` names (for example `request_validation.rs`, `browser_user_agent.rs`) over vague names.
- Prefer explicit module files (`foo.rs`) over opaque `mod.rs` for new work when practical; keep directory + filename understandable without opening the file.
- Keep `.env.local` entries in unquoted `KEY=value` form for consistency. The setup script normalizes quoted values, so avoid introducing new quoted scalars unless a value truly needs shell quoting semantics.

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
