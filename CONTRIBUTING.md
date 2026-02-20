# Contributing to Shuma-Gorath

## Ground Rules

- Follow the project principles in `docs/project-principles.md`.
- Keep changes small, reviewable, and test-backed.
- Prefer refactor-only slices unless behavior change is explicitly intended.
- Avoid new dependencies unless clearly justified.
- Keep verification lanes healthy (`make test` umbrella plus focused lanes); if a lane becomes defunct/flaky, prioritize fixing or removing it before expanding scope.

## Required for Every Change

1. Add/update tests where behavior is affected.
2. Run verification through `Makefile` targets (do not use direct ad-hoc commands as the canonical path):
   - `make test` for the umbrella verification gate (unit + integration + dashboard e2e),
   - `make test-unit`, `make test-integration`, `make test-dashboard-e2e` for focused reruns,
   - `make build` for release build verification,
   - `make setup`/`make verify` when setup/bootstrap tooling changes.
   Notes:
   - `make test` requires Spin running (`make dev`) and fails if integration/e2e cannot run.
   - Makefile is the single source of truth for setup/build/test/deploy; avoid direct command paths in normal workflow (`cargo ...`, `corepack pnpm run ...`, `playwright ...`, `node scripts/...`, `spin ...`).
   - If you need a workflow that is not exposed in `Makefile`, add/update the Make target first and then execute it via `make`.
   - Keep contributor docs, runbooks, and PR verification notes expressed as `make ...` commands to prevent fragmented workflows.
3. Update docs for behavior/config/ops changes.
4. Note security and operational implications.
5. Consider human-visitor friction impact (challenge frequency, latency, UX impact).
6. Consider resource efficiency and cost-placement impact (bandwidth/CPU/memory/energy, and whether cost is shifted toward malicious bots).
7. If a change touches any provider capability boundary (`rate_limiter`, `ban_store`, `challenge_engine`, `maze_tarpit`, `fingerprint_signal`), implement it through the provider interface/registry flow instead of introducing direct module calls on the hot path.

## Variable Lifecycle (Required for `SHUMA_*` changes)

When adding or changing configuration variables, keep one source-of-truth and wire every lifecycle stage:

1. Update `config/defaults.env` first (canonical default source) and classify the variable correctly as env-only or KV-tunable.
2. Keep setup/seed paths in sync so local setup is correct after standard commands:
   - update `scripts/config_seed.sh` for KV tunables,
   - update `scripts/bootstrap/setup.sh` for `.env.local` bootstrap/env-only defaults,
   - update `Makefile` env injection/help surfaces as needed (for example `SPIN_ENV_ONLY`, `env-help`, explicit dev/prod override paths).
3. Keep profile behavior intentional:
   - dev overrides should only assist manual config/monitoring/tuning and remain explicit,
   - tests must restore env/config state they mutate so later tests/runs are not left in a strange state,
   - production defaults must remain secure-by-default.
4. Update `docs/configuration.md` whenever variable meaning/default/ownership changes.

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

## Env Override Style

- Keep `.env.local` in unquoted `KEY=value` format for scalar values used in this project.
- Avoid adding quoted scalar secrets/flags in `.env.local`; `make setup` normalizes quoted values to the unquoted style.

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
