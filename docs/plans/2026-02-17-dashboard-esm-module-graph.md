# Dashboard ESM Module Graph Contract

Date: 2026-02-17  
Scope: `DSH-ESM-6`

## Layer Model

1. `core`
- Pure cross-cutting helpers with no dashboard feature/domain coupling.
- Files:
  - `dashboard/modules/core/dom.js`
  - `dashboard/modules/core/format.js`
  - `dashboard/modules/core/json-object.js`

2. `services`
- Stateful/runtime adapters and shared orchestration primitives.
- Files:
  - `dashboard/modules/services/runtime-effects.js`
  - `dashboard/modules/services/admin-endpoint.js`
  - `dashboard/modules/api-client.js`
  - `dashboard/modules/admin-session.js`
  - `dashboard/modules/dashboard-state.js`
  - `dashboard/modules/tab-lifecycle.js`
  - `dashboard/modules/config-schema.js`
  - `dashboard/modules/config-form-utils.js`
  - `dashboard/modules/config-draft-store.js`

3. `features`
- UI feature modules (derive + render boundaries) that consume service/core contracts.
- Files:
  - `dashboard/modules/monitoring-view.js`
  - `dashboard/modules/tables-view.js`
  - `dashboard/modules/tab-state-view.js`
  - `dashboard/modules/status.js`
  - `dashboard/modules/charts.js`
  - `dashboard/modules/config-controls.js`

4. `main`
- Composition/bootstrap entrypoint.
- File:
  - `dashboard/dashboard.js`

## Import Direction Rules

- `core` imports: none (or same-layer utility files only).
- `services` imports: `core` (and same-layer service helpers).
- `features` imports: `core`, `services` (and same-layer feature helpers).
- `main` imports: any lower layer.
- Reverse imports are forbidden (for example `services -> features`, `core -> services/features/main`).
- Cycles are forbidden across all dashboard JS modules.

## Enforcement

Automated guard coverage in `e2e/dashboard.modules.unit.test.js`:
- layer-direction assertions by import edge,
- cycle detection across the dashboard import graph,
- existing global-registry/class guard checks.
