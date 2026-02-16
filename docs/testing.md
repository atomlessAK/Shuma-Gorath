# 🐙 Testing Guide

## 🐙 Quick Commands (Official)

```bash
make test             # Full umbrella suite: unit + maze benchmark + Spin integration + dashboard e2e
make test-unit        # Unit tests only (native Rust)
make unit-test        # alias for make test-unit
make test-maze-benchmark # Deterministic maze asymmetry benchmark gate
make test-integration # Integration tests only (waits for existing Spin readiness)
make integration-test # alias for make test-integration
make test-coverage    # Unit coverage to lcov.info (requires cargo-llvm-cov)
make test-dashboard-e2e # Playwright dashboard smoke tests (waits for existing Spin readiness)
make seed-dashboard-data # Seed local dashboard sample records against running Spin
make test-dashboard   # Manual dashboard checklist
```

Notes:
- Use Makefile commands only (avoid running scripts directly)
- Integration tests require a running Spin server (`make dev`); test targets do not start Spin.
- `make test`, `make test-integration`, and `make test-dashboard-e2e` wait for `/health` readiness before failing.
- `make test` includes maze asymmetry benchmark gating plus Playwright dashboard e2e and fails if any stage cannot run.
- `make test` now reseeds dashboard sample data at the end, so charts/tables stay populated for local inspection after the run.

## 🐙 Test Layers

This project uses five distinct test environments, each optimized for its scope:

1. Unit tests (native Rust)
2. Integration tests (Spin environment)
3. Dashboard module unit tests (Node `node:test`)
4. Dashboard e2e smoke tests (Playwright)
5. Dashboard checks (manual)

## 🐙 Test Layout Conventions

Rust test layout is now standardized as follows:

- Unit tests should live with the owning module, wired via `#[cfg(test)] mod tests;`.
- Module-specific test files should be placed under that module directory (for example `src/ban/tests.rs` or `src/whitelist/path_tests.rs`).
- Shared unit-test utilities belong in `src/test_support.rs` (request builders, env lock, in-memory KV store fixtures).
- New black-box integration tests should be added in `tests/` when they can rely on public interfaces only.
- Cross-module crate-internal suites should live under `src/lib_tests/`.

## 🐙 Why Two Environments

Unit tests run natively in Rust and validate logic in isolation.
Integration tests must run in Spin because they require the HTTP server, routing, Spin KV storage, and real request headers.

## 🐙 Unit Tests (Native Rust)

Run with:

```bash
make test-unit
```

Unit tests validate core logic in isolation (ban logic, whitelist parsing, config defaults, CDP parsing, etc.).
Test counts may change as coverage evolves; rely on `make test-unit` output for the current total.
Coverage includes ban/unban flows, whitelists, maze generation, challenge logic, CDP parsing, and helper utilities.

## 🐙 Integration Tests (Spin Environment)

Run with:

```bash
# Terminal 1
make dev

# Terminal 2
make test-integration
```

These tests exercise the full HTTP + KV runtime and are required for end-to-end validation.
If your Spin environment sets `SHUMA_FORWARDED_IP_SECRET`, export it before running integration tests so the curl requests include the matching `X-Shuma-Forwarded-Secret` header:

```bash
export SHUMA_FORWARDED_IP_SECRET="your-forwarded-ip-secret"
```

If you configured `SHUMA_HEALTH_SECRET`, export it too so health checks include `X-Shuma-Health-Secret`:

```bash
export SHUMA_HEALTH_SECRET="your-health-secret"
```

The integration suite is implemented in `scripts/tests/integration.sh` and is invoked by `make test-integration`.

Integration coverage includes:
1. Health endpoint and KV availability
2. Root endpoint behavior (block page / JS challenge)
3. Honeypot ban flow
4. Admin config + test-mode toggling
5. Challenge single-use behavior (`Incorrect` then replay `Expired`)
6. Metrics endpoint
7. CDP report ingestion and auto-ban flow
8. CDP stats counters in `/admin/cdp`
9. Monitoring summary endpoint in `/admin/monitoring`
10. Unban behavior

## 🐙 Dashboard E2E Smoke Tests (Playwright)

Run with:

```bash
# Terminal 1
make dev

# Terminal 2
make test-dashboard-e2e
```

Behavior:
1. Installs pinned Playwright dependencies via `pnpm` (through `corepack`).
2. Runs dashboard module unit tests (`node --test e2e/dashboard.modules.unit.test.js`) for response adapters (including missing `Content-Type` JSON fallback), local chart runtime legend/axis rendering checks, shared state primitives, and tab-router helpers.
3. Seeds deterministic dashboard data before tests (admin ban + CDP report + admin view events).
4. Runs browser smoke checks for core dashboard behavior:
   - page loads and refresh succeeds
   - runtime page errors or failed JS/CSS loads fail the run
   - only one dashboard tab panel is visible at a time (panel exclusivity)
   - seeded events/tables are visible
   - clean-state API payloads render explicit empty placeholders (no crash/blank UI)
   - form validation/submit-state behavior works
   - tab hash/keyboard routing works
   - `/dashboard` canonical path redirects to `/dashboard/index.html`
   - tab-level error states surface backend failures
   - sticky table headers remain applied
5. `make test` executes a final dashboard seed step (`make seed-dashboard-data`) after e2e so local dashboards retain recent sample data.

Notes:
- Seeding is test-only and does not run during `make setup`.
- Seeded rows are operational test data and may appear in local dashboard history.

## 🐙 Build Mode Notes

The Makefile switches crate types between `rlib` (native tests) and `cdylib` (Spin WASM) via `scripts/set_crate_type.sh`.
Integration tests do not run `cargo clean`; this avoids interrupting an already-running `make dev` watcher session.
Integration PoW/challenge sequence checks use a fixed test user-agent plus timing guardrails/retries for deterministic behavior.
Use the Makefile targets rather than calling scripts directly.

## 🐙 Generated Directories

These directories are generated locally/CI and should never be committed:

- `dist/wasm/` - built Spin component artifact (`shuma_gorath.wasm`)
- `target/` - Rust build cache/output
- `.spin/` - local Spin runtime data/logs
- `playwright-report/` - Playwright HTML report output
- `test-results/` - Playwright test result artifacts

`make clean` removes these generated outputs, including stale local `src/*.wasm` artifacts.

## 🐙 Manual Test Sequence (Optional)

Use these steps to manually validate behavior. They mirror the integration suite but let you inspect responses in detail.
If `SHUMA_FORWARDED_IP_SECRET` is set, include the matching `X-Shuma-Forwarded-Secret` header on requests that use `X-Forwarded-For`.
If `SHUMA_HEALTH_SECRET` is set, include `X-Shuma-Health-Secret` on `/health`.
Start the server in another terminal with `make dev` before running these steps.

1. Health check (loopback only):
```bash
curl -H "X-Forwarded-For: 127.0.0.1" \
  -H "X-Shuma-Forwarded-Secret: $SHUMA_FORWARDED_IP_SECRET" \
  -H "X-Shuma-Health-Secret: $SHUMA_HEALTH_SECRET" \
  http://127.0.0.1:3000/health
```
Expected: `OK`. If `SHUMA_DEBUG_HEADERS=true`, headers `X-KV-Status` and `X-Shuma-Fail-Mode` are also present.

2. Root endpoint (JS challenge / block page):
```bash
curl -i -H "X-Forwarded-For: 1.2.3.4" \
  -H "X-Shuma-Forwarded-Secret: $SHUMA_FORWARDED_IP_SECRET" \
  http://127.0.0.1:3000/
```
Expected: an "Access Blocked" page or a JS verification interstitial.
If `SHUMA_JS_REQUIRED_ENFORCED=true`, the interstitial is used when no valid `js_verified` cookie is present.
If `SHUMA_POW_ENABLED=true`, the interstitial performs a short proof‑of‑work step before `js_verified` is issued by `/pow/verify`.
If `SHUMA_POW_ENABLED=false`, the interstitial sets `js_verified` directly in browser JS.
After a valid `js_verified` cookie is set, the originally requested page reloads and access is re-evaluated.
For browser checks, use a private window and confirm the cookie is set after the first visit.

3. Honeypot ban:
```bash
curl -s -H "X-Forwarded-For: 1.2.3.4" \
  -H "X-Shuma-Forwarded-Secret: $SHUMA_FORWARDED_IP_SECRET" \
  http://127.0.0.1:3000/instaban > /dev/null
curl -s -H "X-Forwarded-For: 1.2.3.4" \
  -H "X-Shuma-Forwarded-Secret: $SHUMA_FORWARDED_IP_SECRET" \
  http://127.0.0.1:3000/ | head -5
```
Expected: "Access Blocked" for the banned IP.

4. Admin ban:
```bash
curl -X POST -H "Authorization: Bearer $SHUMA_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"ip":"10.20.30.40","duration":3600}' \
  http://127.0.0.1:3000/admin/ban
```
Expected: a JSON response containing the new ban entry.
Optional: verify with `GET /admin/ban` to confirm the IP is listed.

5. Admin unban:
```bash
curl -X POST -H "Authorization: Bearer $SHUMA_API_KEY" \
  "http://127.0.0.1:3000/admin/unban?ip=1.2.3.4"
```
Expected: the IP removed from the ban list.
Optional: verify with `GET /admin/ban` that the entry is gone.

6. Test mode toggle:
```bash
curl -X POST -H "Authorization: Bearer $SHUMA_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"test_mode": true}' \
  http://127.0.0.1:3000/admin/config
```
Expected: a JSON response with `"test_mode":true`.

7. Metrics endpoint:
```bash
curl http://127.0.0.1:3000/metrics
```
Expected: Prometheus metrics output.

8. CDP report intake:
```bash
curl -X POST -H "Content-Type: application/json" \
  -H "X-Forwarded-For: 10.0.0.200" \
  -H "X-Shuma-Forwarded-Secret: $SHUMA_FORWARDED_IP_SECRET" \
  -d '{"cdp_detected":true,"score":0.5,"checks":["webdriver"]}' \
  http://127.0.0.1:3000/cdp-report
```
Expected: a success response and a CDP event recorded in analytics.

9. Challenge replay behavior:
```bash
challenge_page=$(curl -s -H "X-Forwarded-For: 10.0.0.150" \
  -H "X-Shuma-Forwarded-Secret: $SHUMA_FORWARDED_IP_SECRET" \
  http://127.0.0.1:3000/challenge/puzzle)
seed=$(python3 -c 'import re,sys; m=re.search(r"name=\"seed\" value=\"([^\"]+)\"", sys.stdin.read()); print(m.group(1) if m else "")' <<< "$challenge_page")
output=$(python3 -c 'import re,sys; m=re.search(r"name=\"output\"[^>]*value=\"([^\"]+)\"", sys.stdin.read()); print(m.group(1) if m else "")' <<< "$challenge_page")
curl -s -X POST \
  -H "X-Forwarded-For: 10.0.0.150" \
  -H "X-Shuma-Forwarded-Secret: $SHUMA_FORWARDED_IP_SECRET" \
  --data-urlencode "seed=$seed" \
  --data-urlencode "output=$output" \
  http://127.0.0.1:3000/challenge/puzzle
curl -s -X POST \
  -H "X-Forwarded-For: 10.0.0.150" \
  -H "X-Shuma-Forwarded-Secret: $SHUMA_FORWARDED_IP_SECRET" \
  --data-urlencode "seed=$seed" \
  --data-urlencode "output=$output" \
  http://127.0.0.1:3000/challenge/puzzle
```
Expected: first submit returns `Incorrect.` with a new-challenge link; second submit returns `Expired` with the same link.

## 🐙 Complete Manual Test Sequence

Assumes the server is already running in another terminal via `make dev`.
If you are using `SHUMA_FORWARDED_IP_SECRET`, export it before running this sequence.

```bash
set -e
BASE_URL="http://127.0.0.1:3000"
if [[ -z "${SHUMA_API_KEY:-}" ]]; then
  SHUMA_API_KEY="$(grep -E '^SHUMA_API_KEY=' .env.local | tail -1 | cut -d= -f2- | sed -e 's/^"//' -e 's/"$//')"
fi
FORWARDED_SECRET_HEADER=()
if [[ -n "${SHUMA_FORWARDED_IP_SECRET:-}" ]]; then
  FORWARDED_SECRET_HEADER=(-H "X-Shuma-Forwarded-Secret: ${SHUMA_FORWARDED_IP_SECRET}")
fi
HONEYPOT_PATH="$(curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "Authorization: Bearer $SHUMA_API_KEY" "$BASE_URL/admin/config" | python3 -c 'import json,sys; d=json.loads(sys.stdin.read()); print((d.get("honeypots") or ["/instaban"])[0])')"

echo "1) Health"
curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 127.0.0.1" "$BASE_URL/health"
echo ""

echo "2) Root (JS challenge / block page)"
curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 1.2.3.4" "$BASE_URL/" | head -5
echo ""

echo "3) Honeypot ban"
curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 1.2.3.4" "$BASE_URL$HONEYPOT_PATH" > /dev/null
curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 1.2.3.4" "$BASE_URL/" | head -5
echo ""

echo "4) Admin ban"
curl -s "${FORWARDED_SECRET_HEADER[@]}" -X POST -H "Authorization: Bearer $SHUMA_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"ip":"10.20.30.40","reason":"manual_test","duration":3600}' \
  "$BASE_URL/admin/ban"
echo ""

echo "5) Admin unban"
curl -s "${FORWARDED_SECRET_HEADER[@]}" -X POST -H "Authorization: Bearer $SHUMA_API_KEY" \
  "$BASE_URL/admin/unban?ip=1.2.3.4"
echo ""

echo "6) Test mode on, then off"
curl -s "${FORWARDED_SECRET_HEADER[@]}" -X POST -H "Authorization: Bearer $SHUMA_API_KEY" \
  -H "Content-Type: application/json" -d '{"test_mode": true}' \
  "$BASE_URL/admin/config"
curl -s "${FORWARDED_SECRET_HEADER[@]}" -X POST -H "Authorization: Bearer $SHUMA_API_KEY" \
  -H "Content-Type: application/json" -d '{"test_mode": false}' \
  "$BASE_URL/admin/config"
echo ""

echo "7) Metrics"
curl -s "$BASE_URL/metrics" | head -20
echo ""

echo "8) CDP report"
curl -s "${FORWARDED_SECRET_HEADER[@]}" -X POST -H "Content-Type: application/json" \
  -H "X-Forwarded-For: 10.0.0.200" \
  -d '{"cdp_detected":true,"score":0.5,"checks":["webdriver"]}' \
  "$BASE_URL/cdp-report"
echo ""
```

## 🐙 Local Testing Notes

- If you visit `/instaban` in a browser without `X-Forwarded-For`, your IP is detected as `unknown`.
- To unban yourself locally:
```bash
curl -X POST -H "Authorization: Bearer $SHUMA_API_KEY" \
  "http://127.0.0.1:3000/admin/unban?ip=unknown"
```

## 🐙 Additional Manual Checks

- Whitelist: add your IP via `/admin/config` and confirm access is always allowed
- Rate limit: send a burst of requests and confirm auto-ban
- Outdated browser: send a low-version User-Agent (example: `Chrome/50`)
- GEO policy: set `geo_*` lists via `/admin/config`, then send `X-Geo-Country` with a trusted forwarded-secret request and verify `allow/challenge/maze/block` routing precedence
- Ban list: `GET /admin/ban` and confirm entries match recent actions

## 🐙 Troubleshooting

Problem: `/health` returns 403
- Ensure you passed `X-Forwarded-For: 127.0.0.1`
- If `SHUMA_FORWARDED_IP_SECRET` is set, include `X-Shuma-Forwarded-Secret`
- If `SHUMA_HEALTH_SECRET` is set, include `X-Shuma-Health-Secret`
- Confirm the server is running with `make status`

Problem: Admin calls fail with 401/403
- Confirm `SHUMA_API_KEY` is correct
- If `SHUMA_ADMIN_IP_ALLOWLIST` is set, ensure your IP is included

Problem: `make test` failed preflight (server not ready)
- Start the server with `make dev`
- Re-run with `make test`
- If startup is slow, increase wait timeout: `make test SPIN_READY_TIMEOUT_SECONDS=180`

Problem: Unsure what IP the bot defence detected
- Query the ban list:
```bash
curl -H "Authorization: Bearer $SHUMA_API_KEY" \
  http://127.0.0.1:3000/admin/ban
```

## 🐙 Dashboard Manual Check

Open:
- `http://127.0.0.1:3000/dashboard/index.html`

Verify:
- Stats update on refresh
- Charts render correctly
- Ban/unban controls work
- Test mode toggle updates banner
- Fail-open/closed indicator matches deployment policy
- Login key should match `make api-key-show` (or your deployed `SHUMA_API_KEY`)
- Use the dashboard Ban IP and Unban actions to validate the admin API wiring

## 🐙 Tips

Use browser developer tools to inspect:
- Network tab: headers, cookies, redirects
- Application tab: `js_verified` cookie
- Console: JS errors
