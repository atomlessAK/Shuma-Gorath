# Contributing to Shuma-Gorath

## Ground Rules

- Follow the project principles in `docs/project-principles.md`.
- Keep changes small, reviewable, and test-backed.
- Prefer refactor-only slices unless behavior change is explicitly intended.
- Avoid new dependencies unless clearly justified.

## Required for Every Change

1. Add/update tests where behavior is affected.
2. Run verification through `Makefile` targets (do not use direct ad-hoc commands as the canonical path):
   - `make test` for the umbrella verification gate (unit + integration + dashboard e2e),
   - `make test-unit`, `make test-integration`, `make test-dashboard-e2e` for focused reruns,
   - `make build` for release build verification,
   - `make setup`/`make verify` when setup/bootstrap tooling changes.
   Notes:
   - `make test` requires Spin running (`make dev`) and fails if integration/e2e cannot run.
3. Update docs for behavior/config/ops changes.
4. Note security and operational implications.
5. Consider human-visitor friction impact (challenge frequency, latency, UX impact).
6. Consider resource efficiency and cost-placement impact (bandwidth/CPU/memory/energy, and whether cost is shifted toward malicious bots).

## Commit and Push Discipline

- Prefer atomic commits: one logical change per commit.
- Do not mix unrelated work (for example refactors + new behavior + docs cleanups) in a single commit.
- Run the relevant `make` verification target(s) before each commit.
- Push after each validated atomic commit unless batching is explicitly requested.

## Rust Naming and Layout Convention

- Use descriptive `snake_case` module and file names that make responsibility obvious from path + filename.
- Avoid vague root-level names (for example `input_validation.rs` at `src/`) when domain context is required; prefer domain-scoped placement and naming.
- Prefer explicit module files (`foo.rs`) instead of `foo/mod.rs` for new code when practical.
- Keep naming semantics consistent across domains (`renders.rs` for renderers, `http.rs` for HTTP handlers, `tests.rs` for colocated unit tests).
- Treat renames as behavior-preserving refactors: update imports/paths and run full `make test`.

## When an ADR Is Required

Create/update an ADR in `docs/adr/` when you change architecture or cross-cutting behavior:

- Module boundaries or dependency direction.
- Provider interfaces/pluggability model.
- Security model or trust boundaries.
- Deployment model or runtime assumptions.
- High-cost defenses and their trade-offs.
- Intentional breaking behavior.

Use `docs/adr/0000-template.md`.

## Pull Requests

- Use the PR checklist in `.github/pull_request_template.md`.
- Keep PR description concrete: what changed, why, risk, and verification.
- Link related issues/ADRs.

## Documentation Expectations

- Keep docs discoverable from `README.md` and `docs/index.md`.
- Prefer clear operator guidance over implied behavior.
- If something is security-sensitive, include deployment hardening notes.
