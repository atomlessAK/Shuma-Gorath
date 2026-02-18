# 0002 Dashboard SvelteKit Cutover

- Status: Accepted
- Date: 2026-02-18
- Supersedes: Svelte-relevant portions of `docs/plans/2026-02-17-dashboard-lit-full-cutover.md`

## Context

The dashboard was mid-migration from vanilla JavaScript toward Lit, but the project now requires a SvelteKit-first architecture. We need to preserve all current dashboard functionality and design while improving frontend structure and maintainability.

Repository constraints:
1. Production runtime is Spin + static files (no Node server at runtime).
2. Dashboard must remain served at `/dashboard` with existing login/session behavior.
3. Canonical verification must continue through `Makefile` targets.

## Decision

Adopt SvelteKit with `adapter-static` as the canonical dashboard runtime.

1. Build source from `dashboard/` to `dist/dashboard`.
2. Serve `dist/dashboard` through existing Spin static-files component.
3. Preserve behavior and route contracts, including:
- `/dashboard/index.html`
- `/dashboard/login.html`
- existing hash-based tab routes.
4. During cutover, treat existing imperative dashboard/login JavaScript as a bounded compatibility layer mounted from Svelte route lifecycle bridges.

## Rationale

1. SvelteKit gives a clean component/lifecycle foundation and modern frontend ergonomics.
2. Static adapter preserves current deploy topology and avoids operational complexity.
3. Compatibility bridges let us complete migration quickly without breaking behavior.
4. Existing tests remain high-value guardrails while implementation migrates.

## Consequences

### Positive
1. One canonical dashboard frontend architecture (SvelteKit).
2. No runtime CDN or framework boot indirection required for app shell.
3. Cleaner route ownership and explicit boot lifecycle at Svelte route boundaries.

### Trade-offs
1. Build step is now required for dashboard static assets.
2. Legacy runtime modules remain temporarily, now explicitly categorized as compatibility code.

## Security impact

1. No trust-boundary changes: auth/session/admin API contracts remain server-enforced.
2. Static-only production dashboard runtime avoids introducing long-lived Node service attack surface.

## Operational impact

1. `make dev` and `make build` must include dashboard static build.
2. Spin static files source changes from `dashboard` to `dist/dashboard`.

## Resource impact

1. Build-time optimization should reduce client transfer size versus unoptimized static scripts.
2. Development CPU usage may increase slightly due SvelteKit build step.

## Rollback

1. Repoint `spin.toml` dashboard static source back to `dashboard`.
2. Restore legacy HTML/JS entrypoints as static served runtime.
3. Re-run `make test` and `make build` before deploy rollback.
