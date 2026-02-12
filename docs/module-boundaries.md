# Module Boundaries

This document defines the in-repo boundaries used to prepare future repo splits.

## Current Contract Layer

`src/boundaries/contracts.rs` is the explicit contract seam between request orchestration and domain modules.

- `ChallengeBoundary`
- `MazeBoundary`
- `AdminBoundary`

`src/boundaries/adapters.rs` provides default adapters that map those contracts to current modules:

- `crate::challenge`
- `crate::maze`
- `crate::admin`

`src/lib.rs` routes through `src/boundaries/mod.rs` instead of calling those domain modules directly.

## Target Split Direction

- Core policy/orchestration: `src/lib.rs` and `src/runtime/` helpers (`request_router`, `kv_gate`, `policy_pipeline`)
- Admin adapter domain: `src/admin/` (`api.rs` endpoint surface + `auth.rs` auth/session concerns)
- Config domain: `src/config/mod.rs` (+ `src/config/tests.rs`)
- Signal domain: `src/signals/` (browser/CDP/GEO/IP/JS/whitelist)
- Enforcement domain: `src/enforcement/` (ban/block/rate/honeypot)
- Maze/tarpit domain: `src/maze/` plus future tarpit implementation
- Challenge domain: `src/challenge/`
- Dashboard adapter: `dashboard/modules/` API/session/config adapters

## Rules For New Work

- Keep `src/lib.rs` focused on orchestration and call domains via boundary adapters.
- Add new cross-domain behavior by extending a boundary contract first.
- Avoid adding direct `crate::<domain>` calls in `src/lib.rs` when a boundary exists.
- Keep boundary contracts behavior-focused and minimal to reduce split risk.
