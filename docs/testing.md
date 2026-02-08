# 🐙 Testing Guide

## 🐙 Quick Commands (Official)

```bash
make test             # Unit tests + integration if server running
make test-unit        # Unit tests only (native Rust)
make test-integration # Integration tests only (Spin required)
make test-dashboard   # Manual dashboard checklist
```

Notes:
- Use Makefile commands only (avoid running scripts directly)
- Integration tests require a running Spin server

## 🐙 Test Layers

This project uses three distinct test environments, each optimized for its scope:

1. Unit tests (native Rust)
2. Integration tests (Spin environment)
3. Dashboard checks (manual)

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
If your Spin environment sets `FORWARDED_IP_SECRET`, export it before running integration tests so the curl requests include the matching `X-Shuma-Forwarded-Secret` header:

```bash
export FORWARDED_IP_SECRET="your-forwarded-ip-secret"
```

The integration suite is implemented in `test_spin_colored.sh` and is invoked by `make test-integration`.

Integration coverage includes:
1. Health endpoint and KV availability
2. Root endpoint behavior (block page / JS challenge)
3. Honeypot ban flow
4. Admin config + test-mode toggling
5. Challenge single-use behavior (`Incorrect` then replay `Expired`)
6. Metrics endpoint
7. CDP report ingestion and auto-ban flow
8. CDP stats counters in `/admin/cdp`
9. Unban behavior

## 🐙 Build Mode Notes

The Makefile switches crate types between `rlib` (native tests) and `cdylib` (Spin WASM) via `scripts/set_crate_type.sh`.
Integration tests run `cargo clean` to avoid stale artifacts.
Use the Makefile targets rather than calling scripts directly.

## 🐙 Manual Test Sequence (Optional)

Use these steps to manually validate behavior. They mirror the integration suite but let you inspect responses in detail.
If `FORWARDED_IP_SECRET` is set, include the matching `X-Shuma-Forwarded-Secret` header on requests that use `X-Forwarded-For`.
Start the server in another terminal with `make dev` before running these steps.

1. Health check (loopback only):
```bash
curl -H "X-Forwarded-For: 127.0.0.1" \
  -H "X-Shuma-Forwarded-Secret: $FORWARDED_IP_SECRET" \
  http://127.0.0.1:3000/health
```
Expected: `OK`. If `SHUMA_DEBUG_HEADERS=true`, headers `X-KV-Status` and `X-Shuma-Fail-Mode` are also present.

2. Root endpoint (JS challenge / block page):
```bash
curl -i -H "X-Forwarded-For: 1.2.3.4" \
  -H "X-Shuma-Forwarded-Secret: $FORWARDED_IP_SECRET" \
  http://127.0.0.1:3000/
```
Expected: an "Access Blocked" page or a JS challenge that sets a `js_verified` cookie.
If `POW_ENABLED` is true, the JS challenge performs a short proof‑of‑work step first.
For browser checks, use a private window and confirm the cookie is set after the first visit.

3. Honeypot ban:
```bash
curl -s -H "X-Forwarded-For: 1.2.3.4" \
  -H "X-Shuma-Forwarded-Secret: $FORWARDED_IP_SECRET" \
  http://127.0.0.1:3000/bot-trap > /dev/null
curl -s -H "X-Forwarded-For: 1.2.3.4" \
  -H "X-Shuma-Forwarded-Secret: $FORWARDED_IP_SECRET" \
  http://127.0.0.1:3000/ | head -5
```
Expected: "Access Blocked" for the banned IP.

4. Admin ban:
```bash
curl -X POST -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"ip":"10.20.30.40","reason":"test","duration":3600}' \
  http://127.0.0.1:3000/admin/ban
```
Expected: a JSON response containing the new ban entry.
Optional: verify with `GET /admin/ban` to confirm the IP is listed.

5. Admin unban:
```bash
curl -X POST -H "Authorization: Bearer $API_KEY" \
  "http://127.0.0.1:3000/admin/unban?ip=1.2.3.4"
```
Expected: the IP removed from the ban list.
Optional: verify with `GET /admin/ban` that the entry is gone.

6. Test mode toggle:
```bash
curl -X POST -H "Authorization: Bearer $API_KEY" \
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
  -H "X-Shuma-Forwarded-Secret: $FORWARDED_IP_SECRET" \
  -d '{"cdp_detected":true,"score":0.5,"checks":["webdriver"]}' \
  http://127.0.0.1:3000/cdp-report
```
Expected: a success response and a CDP event recorded in analytics.

9. Challenge replay behavior:
```bash
challenge_page=$(curl -s -H "X-Forwarded-For: 10.0.0.150" \
  -H "X-Shuma-Forwarded-Secret: $FORWARDED_IP_SECRET" \
  http://127.0.0.1:3000/challenge)
seed=$(python3 -c 'import re,sys; m=re.search(r"name=\"seed\" value=\"([^\"]+)\"", sys.stdin.read()); print(m.group(1) if m else "")' <<< "$challenge_page")
output=$(python3 -c 'import re,sys; m=re.search(r"name=\"output\"[^>]*value=\"([^\"]+)\"", sys.stdin.read()); print(m.group(1) if m else "")' <<< "$challenge_page")
curl -s -X POST \
  -H "X-Forwarded-For: 10.0.0.150" \
  -H "X-Shuma-Forwarded-Secret: $FORWARDED_IP_SECRET" \
  --data-urlencode "seed=$seed" \
  --data-urlencode "output=$output" \
  http://127.0.0.1:3000/challenge
curl -s -X POST \
  -H "X-Forwarded-For: 10.0.0.150" \
  -H "X-Shuma-Forwarded-Secret: $FORWARDED_IP_SECRET" \
  --data-urlencode "seed=$seed" \
  --data-urlencode "output=$output" \
  http://127.0.0.1:3000/challenge
```
Expected: first submit returns `Incorrect.` with a new-challenge link; second submit returns `Expired` with the same link.

## 🐙 Complete Manual Test Sequence

Assumes the server is already running in another terminal via `make dev`.
If you are using `FORWARDED_IP_SECRET`, export it before running this sequence.

```bash
set -e
BASE_URL="http://127.0.0.1:3000"
API_KEY="${API_KEY:-changeme-dev-only-api-key}"
FORWARDED_SECRET_HEADER=()
if [[ -n "${FORWARDED_IP_SECRET:-}" ]]; then
  FORWARDED_SECRET_HEADER=(-H "X-Shuma-Forwarded-Secret: ${FORWARDED_IP_SECRET}")
fi

echo "1) Health"
curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 127.0.0.1" "$BASE_URL/health"
echo ""

echo "2) Root (JS challenge / block page)"
curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 1.2.3.4" "$BASE_URL/" | head -5
echo ""

echo "3) Honeypot ban"
curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 1.2.3.4" "$BASE_URL/bot-trap" > /dev/null
curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 1.2.3.4" "$BASE_URL/" | head -5
echo ""

echo "4) Admin ban"
curl -s "${FORWARDED_SECRET_HEADER[@]}" -X POST -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"ip":"10.20.30.40","reason":"manual_test","duration":3600}' \
  "$BASE_URL/admin/ban"
echo ""

echo "5) Admin unban"
curl -s "${FORWARDED_SECRET_HEADER[@]}" -X POST -H "Authorization: Bearer $API_KEY" \
  "$BASE_URL/admin/unban?ip=1.2.3.4"
echo ""

echo "6) Test mode on, then off"
curl -s "${FORWARDED_SECRET_HEADER[@]}" -X POST -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" -d '{"test_mode": true}' \
  "$BASE_URL/admin/config"
curl -s "${FORWARDED_SECRET_HEADER[@]}" -X POST -H "Authorization: Bearer $API_KEY" \
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

- If you visit `/bot-trap` in a browser without `X-Forwarded-For`, your IP is detected as `unknown`.
- To unban yourself locally:
```bash
curl -X POST -H "Authorization: Bearer $API_KEY" \
  "http://127.0.0.1:3000/admin/unban?ip=unknown"
```

## 🐙 Additional Manual Checks

- Whitelist: add your IP via `/admin/config` and confirm access is always allowed
- Rate limit: send a burst of requests and confirm auto-ban
- Outdated browser: send a low-version User-Agent (example: `Chrome/50`)
- Geo risk: set `X-Geo-Country` to a high-risk country code and confirm behavior
- Ban list: `GET /admin/ban` and confirm entries match recent actions

## 🐙 Troubleshooting

Problem: `/health` returns 403
- Ensure you passed `X-Forwarded-For: 127.0.0.1`
- If `FORWARDED_IP_SECRET` is set, include `X-Shuma-Forwarded-Secret`
- Confirm the server is running with `make status`

Problem: Admin calls fail with 401/403
- Confirm `API_KEY` is correct
- If `ADMIN_IP_ALLOWLIST` is set, ensure your IP is included

Problem: Integration tests were skipped
- Start the server with `make dev`
- Re-run with `make test-integration`

Problem: Unsure what IP the bot trap detected
- Query the ban list:
```bash
curl -H "Authorization: Bearer $API_KEY" \
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
- API key defaults to `changeme-dev-only-api-key` for local dev (replace in production)
- Use the dashboard Ban IP and Unban actions to validate the admin API wiring

## 🐙 Tips

Use browser developer tools to inspect:
- Network tab: headers, cookies, redirects
- Application tab: `js_verified` cookie
- Console: JS errors
