# Dashboard SvelteKit Full Cutover Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Move the dashboard runtime and UI delivery to SvelteKit while preserving all existing dashboard behavior, routes, and visual design.

**Architecture:** Use SvelteKit static prerendering for `/dashboard` pages and treat existing imperative dashboard logic as an explicit compatibility boundary during cutover. The public runtime becomes SvelteKit-generated assets only (`dist/dashboard`), while current behavior modules are invoked via controlled lifecycle bridges from Svelte routes.

**Tech Stack:** SvelteKit (`@sveltejs/kit`), adapter-static, Vite, existing dashboard JS modules, existing Playwright/Node test suite.

---

## 1. Decision Summary

### Chosen direction
1. Hard cutover to SvelteKit static output served by Spin at `/dashboard`.
2. Preserve route and UX contracts:
- `/dashboard` redirects to `/dashboard/index.html`.
- Login path remains `/dashboard/login.html`.
- Tab hash routes remain `#monitoring`, `#ip-bans`, `#status`, `#config`, `#tuning`.
3. Preserve all current dashboard functionality/design first, then iterate on deeper Svelte-native extraction with behavior parity tests already in place.

### Why this is optimal for this repository
1. Zero server-runtime dependency: adapter-static keeps deployment as static assets under current Spin fileserver model.
2. Lowest behavior risk: existing battle-tested modules continue to own domain behavior during cutover.
3. Strong migration seam: Svelte route lifecycle becomes the only boot surface; legacy runtime moves behind bridge modules with explicit mount semantics.
4. Test continuity: existing module and e2e suites remain valid while SvelteKit takes over page rendering.

## 2. Target Runtime Architecture

### 2.1 Delivery and hosting
1. SvelteKit source lives under `dashboard/`.
2. Build output goes to `dist/dashboard`.
3. Spin serves `dist/dashboard` via `component.dashboard-files`.
4. `Makefile` becomes canonical path to build dashboard static assets before Spin start/build verification.

### 2.2 UI composition
1. Route: `src/routes/+page.svelte` for main dashboard.
2. Route: `src/routes/login.html/+page.svelte` for login page (keeps existing URL contract).
3. Shared global styles flow through `src/app.css`.
4. Existing HTML design is preserved exactly by rendering static shell fragments in Svelte routes.

### 2.3 Behavior bridge
1. `src/lib/bridges/dashboard-runtime.js` mounts legacy dashboard runtime from Svelte `onMount`.
2. `src/lib/bridges/login-runtime.js` mounts legacy login runtime from Svelte `onMount`.
3. Bridge modules guard against duplicate bootstrap and define cleanup hooks for future full Svelte-native replacement.

### 2.4 Static assets
1. `dashboard/static/**` is the canonical SvelteKit static asset source.
2. Chart runtime (`chart-lite`) is loaded from local static asset path in Svelte head.

## 3. Execution Tasks (reviewable slices)

### Task 1: SvelteKit platform foundation

**Files:**
- Create: `dashboard/svelte.config.js`
- Create: `dashboard/vite.config.js`
- Create: `dashboard/jsconfig.json`
- Create: `dashboard/src/app.html`
- Create: `dashboard/src/app.css`
- Modify: `package.json`
- Modify: `pnpm-lock.yaml`

**Steps:**
1. Add SvelteKit tooling dependencies and build script.
2. Configure adapter-static output to `dist/dashboard`.
3. Set `kit.paths.base = '/dashboard'` and prerender entries for dashboard routes.
4. Wire Svelte/Vite config for project-rooted build in `dashboard/`.

**Verification:**
- Run: `corepack pnpm install --frozen-lockfile`
- Run: `corepack pnpm run build:dashboard`
- Expected: build completes and produces `dist/dashboard/index.html`.

### Task 2: Route and shell migration

**Files:**
- Create: `dashboard/src/routes/+page.svelte`
- Create: `dashboard/src/routes/login.html/+page.svelte`
- Create: `dashboard/src/lib/shell/dashboard-body.html`
- Create: `dashboard/src/lib/shell/login-body.html`
- Create: `dashboard/src/lib/bridges/dashboard-runtime.js`
- Create: `dashboard/src/lib/bridges/login-runtime.js`

**Steps:**
1. Extract body markup from legacy `dashboard/index.html` and `dashboard/login.html` into shell fragments.
2. Render shell fragments from Svelte routes.
3. Load chart runtime and mount bridge logic in route lifecycle.
4. Keep exact element IDs/class names to preserve behavior and test contracts.

**Verification:**
- Run: `corepack pnpm run build:dashboard`
- Expected: static pages include preserved shell markup and generated Svelte assets.

### Task 3: Spin + Make integration

**Files:**
- Modify: `spin.toml`
- Modify: `Makefile`

**Steps:**
1. Point dashboard static file source from `dashboard` to `dist/dashboard`.
2. Add canonical `make` dashboard build invocation to relevant build/dev/run paths.
3. Ensure `make dev` rebuilds dashboard assets before serving.

**Verification:**
- Run: `make build`
- Expected: wasm + dashboard static build complete.

### Task 4: Documentation and migration governance

**Files:**
- Create: `docs/adr/0002-dashboard-sveltekit-cutover.md`
- Modify: `docs/dashboard.md`
- Modify: `todos/todo.md`

**Steps:**
1. Record architecture decision and consequences in ADR.
2. Update dashboard docs to reflect SvelteKit runtime and build/serve model.
3. Replace Lit-cutover todo direction with SvelteKit-cutover direction.

**Verification:**
- Docs are consistent with runtime implementation and Makefile workflow.

### Task 5: Full regression verification

**Files:**
- Modify as needed from failures in prior tasks.

**Steps:**
1. Start Spin in one session: `make dev`.
2. In another session run canonical suite: `make test`.
3. Run release verification: `make build`.

**Verification:**
- `make test` passes (unit + integration + dashboard e2e).
- `make build` passes.

## 4. Risk Controls

1. **Behavior drift risk:** preserve existing IDs/markup contracts and run current e2e suite unchanged first.
2. **Asset path risk:** enforce `/dashboard` base path in SvelteKit config and verify generated asset URLs.
3. **Operational risk:** retain current Spin topology; only static source path changes.
4. **Rollback:** switch `spin.toml` dashboard file source back to `dashboard` and restore legacy entry pages.

## 5. Security, Ops, Resource Notes

1. Security posture is unchanged: auth endpoints, session cookie flow, and admin API contracts remain server-side as-is.
2. Operational model remains static files + Spin; no Node server in production runtime.
3. Resource impact is expected to improve from removal of Lit dependency path and explicit build-time optimization of client assets.
