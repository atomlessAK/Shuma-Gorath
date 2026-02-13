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
- Crawler policy domain: `src/crawler_policy/` (`robots` policy generation and crawler directives)
- Maze/tarpit domain: `src/maze/` plus future tarpit implementation
- Challenge domain: `src/challenge/` (`puzzle` and future `not_a_bot` challenge modes)
- Observability domain: `src/observability/` (`metrics` and monitoring surfaces)
- Dashboard adapter: `dashboard/modules/` API/session/config adapters

## Defence Taxonomy (H3.6.1)

This taxonomy defines how modules participate in bot defense composition.

- `signal`: contributes evidence for scoring/risk decisions, no direct blocking by itself.
- `barrier`: enforces a defensive action (block/challenge/maze/tarpit/ban flow), may log metrics.
- `hybrid`: supports both signal contribution and direct enforcement, via explicit separate paths.

### Inventory

| Module Path | Class | Primary Role | Ownership | Dependency Direction |
| --- | --- | --- | --- | --- |
| `src/signals/geo/` | signal | Country extraction and GEO policy signal inputs | Signals | Can depend on config/input utils; must not depend on enforcement modules. |
| `src/signals/cdp/` | signal | Automation fingerprinting signal (CDP checks/reporting) | Signals | Can depend on config/shared types; direct enforcement calls are temporary and should migrate to policy/barrier orchestration. |
| `src/signals/js_verification.rs` | signal | JS verification signal/challenge trigger inputs | Signals | Can depend on challenge/pow helpers for presentation, but should not own hard block decisions. |
| `src/signals/browser_user_agent.rs` | signal | Browser capability/version signal | Signals | Independent utility signal source; no enforcement dependencies. |
| `src/signals/ip_identity.rs` | signal | IP bucketing utility for telemetry/signal keys | Signals | Leaf utility; no enforcement dependencies. |
| `src/signals/rate_pressure.rs` | signal | Rate-window telemetry and pressure-band scoring signals | Signals | Reads request-rate counters for scoring only; does not enforce bans/blocks. |
| `src/signals/whitelist/` | signal | Allow-list signal short-circuit inputs | Signals | Can depend on parsing/input modules; no enforcement dependencies. |
| `src/enforcement/honeypot.rs` | barrier | Honeypot path detection for immediate defensive action | Enforcement | May consume routing/config context; should not calculate botness directly. |
| `src/enforcement/ban/` | barrier | Ban persistence and ban-state enforcement primitives | Enforcement | May depend on storage/input sanitation; no direct dependence on signal module internals. |
| `src/enforcement/block_page.rs` | barrier | Block response rendering | Enforcement | Presentation-only enforcement utility. |
| `src/enforcement/rate.rs` | barrier | Hard rate-limit cap enforcement for immediate protection | Enforcement | Owns allow/deny counter writes and threshold enforcement responses. |
| `src/maze/` | barrier | Deception/maze barrier for suspicious traffic | Maze/Tarpit domain | Consumes policy decisions; should expose behavior via boundary adapters. |
| `src/challenge/` | barrier | Interactive challenge barrier flow | Challenge domain | Consumes policy decisions; should expose behavior via boundary adapters. |
| `src/challenge/pow.rs` | hybrid (candidate) | Cost-imposition challenge path + verification signal potential | Challenge domain | Current behavior is barrier-focused; future signal contribution should be explicit via normalized signal contract. |

`src/maze/` and `src/challenge/` are intentionally separate barrier domains rather than nested under `src/enforcement/`.
They are larger, multi-step flows with dedicated adapters/contracts and are likely candidates for future split-out modules.

### Dependency Rules for Composition

- Signal modules write evidence; they do not decide final blocking outcomes.
- Barrier modules consume policy outcomes; they do not silently mutate botness scoring internals.
- Hybrid modules MUST provide explicit split APIs for signal contribution and enforcement action.
- Runtime orchestration (`src/runtime/policy_pipeline.rs`) remains the ordering authority.
- Runtime observability MUST publish both per-signal availability state (`active`/`disabled`/`unavailable`) and effective mode state (`configured`, `signal_enabled`, `action_enabled`) so tuning outcomes are explainable.

## Rules For New Work

- Keep `src/lib.rs` focused on orchestration and call domains via boundary adapters.
- Add new cross-domain behavior by extending a boundary contract first.
- Avoid adding direct `crate::<domain>` calls in `src/lib.rs` when a boundary exists.
- When internal feature work touches a provider-managed capability, route behavior through `src/providers/contracts.rs` and `src/providers/registry.rs` rather than adding direct module calls in orchestration paths.
- Keep boundary contracts behavior-focused and minimal to reduce split risk.

## Provider Contracts (H4.1)

`src/providers/contracts.rs` defines internal provider seams for future swappable implementations.

- `RateLimiterProvider`
- `BanStoreProvider`
- `ChallengeEngineProvider`
- `MazeTarpitProvider`
- `FingerprintSignalProvider`

`src/providers/registry.rs` is the internal-first selection layer that reads per-capability backend selections from config:

- `rate_limiter`
- `ban_store`
- `challenge_engine`
- `maze_tarpit`
- `fingerprint_signal`

Backends currently supported at config/selection level:

- `internal` (default)
- `external` (adapter path now explicit in `src/providers/external.rs`)

Current `external` behavior:

- `fingerprint_signal` uses an explicit external stub contract (`/fingerprint-report`) until a concrete adapter is configured.
- `rate_limiter` uses a Redis-backed distributed adapter (`INCR` + TTL) when `SHUMA_RATE_LIMITER_REDIS_URL` is configured, with fallback to internal rate behavior when unavailable.
- `ban_store` uses a Redis-backed distributed adapter (JSON ban records + Redis TTL) when `SHUMA_BAN_STORE_REDIS_URL` is configured, with fallback to internal ban behavior when unavailable.
- `challenge_engine` and `maze_tarpit` still use explicit unsupported external adapters that currently delegate to internal runtime behavior.

Implementation discipline for H4 coherence:

- Internal feature work that changes behavior in any provider-managed capability MUST be implemented through the provider interface path (contract + registry + provider implementation), even when backend selection remains `internal`.

## File Naming Convention

- Prefer descriptive, role-based file names with consistent semantics across domains.
- Use clear `snake_case` names that reveal responsibility from the path itself (for example `signals/browser_user_agent.rs`, `request_validation.rs`).
- Prefer explicit module files (`foo.rs`) over opaque `foo/mod.rs` for new modules when practical.
- Use `renders.rs` for HTML/page rendering modules when directory context already defines the feature (`maze`, `challenge/puzzle`, etc.).
- Use `http.rs` for request handling modules.
- Use `tests.rs` for colocated unit tests when the surrounding module path already provides context.
- Avoid vague names when directory context alone does not clearly convey responsibility.
