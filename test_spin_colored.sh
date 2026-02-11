#!/bin/bash
# test_spin_colored.sh
# Integration test suite for Spin app with colored output
#
# ⚠️ IMPORTANT: These tests MUST run in the Spin environment!
# They require HTTP server, key-value store, and real headers.
#
# PREREQUISITES:
#   1. Start Spin server: spin up
#   2. Run this script: ./test_spin_colored.sh
#
# This script runs integration test scenarios:
#   1. Health check endpoint (GET /health)
#   2. PoW challenge + verification (if enabled)
#   3. Root endpoint behavior (GET /)
#   4. Honeypot ban detection (POST /instaban)
#   5. Admin API unban (POST /admin/unban)
#   6. Health check after ban/unban (GET /health)
#   7. Config API - get config (GET /admin/config)
#   8. Test mode toggle (POST /admin/config)
#   9. Challenge single-use flow (incorrect -> replay expired)
#   10. Test mode behavior verification
#   11. Test mode disable and blocking resumes
#   12. Verify blocking after test mode disabled
#   13. GEO policy challenge route
#   14. GEO policy maze route
#   15. GEO policy block route
#   16. Prometheus metrics endpoint
#   17. CDP report endpoint (POST /cdp-report)
#   18. CDP auto-ban with high score
#   19. CDP config via admin API
#   20. CDP stats counters reflect reports
#   21. Unban functionality test

set -e

# Always clean before integration tests to avoid stale artifacts
cargo clean
GREEN="\033[0;32m"
RED="\033[0;31m"
YELLOW="\033[1;33m"
NC="\033[0m" # No Color

FAILURES=0
TEST_HONEYPOT_IP="10.0.0.88"
HONEYPOT_PATH="/instaban"

pass() { echo -e "${GREEN}PASS${NC} $1"; }
fail() {
  echo -e "${RED}FAIL${NC} $1"
  FAILURES=$((FAILURES + 1))
}
info() { echo -e "${YELLOW}INFO${NC} $1"; }

BASE_URL="http://127.0.0.1:3000"

read_env_local_value() {
  local key="$1"
  if [[ ! -f ".env.local" ]]; then
    return 1
  fi
  local line
  line=$(grep -E "^${key}=" .env.local | tail -1 || true)
  if [[ -z "$line" ]]; then
    return 1
  fi
  local value="${line#*=}"
  value="${value%\"}"
  value="${value#\"}"
  value="${value%\'}"
  value="${value#\'}"
  printf '%s' "$value"
}

if [[ -z "${SHUMA_API_KEY:-}" ]]; then
  SHUMA_API_KEY="$(read_env_local_value SHUMA_API_KEY || true)"
fi

if [[ -z "${SHUMA_API_KEY:-}" ]]; then
  fail "Missing SHUMA_API_KEY. Run make setup (or export SHUMA_API_KEY) before integration tests."
  exit 1
fi

case "$SHUMA_API_KEY" in
  changeme-dev-only-api-key|changeme-supersecret|changeme-prod-api-key)
    fail "SHUMA_API_KEY is a placeholder. Run make setup or make api-key-generate first."
    exit 1
    ;;
esac

FORWARDED_SECRET_HEADER=()
if [[ -z "${SHUMA_FORWARDED_IP_SECRET:-}" ]]; then
  SHUMA_FORWARDED_IP_SECRET="$(read_env_local_value SHUMA_FORWARDED_IP_SECRET || true)"
fi

if [[ -n "${SHUMA_FORWARDED_IP_SECRET:-}" ]]; then
  FORWARDED_SECRET_HEADER=(-H "X-Shuma-Forwarded-Secret: ${SHUMA_FORWARDED_IP_SECRET}")
fi

# Test 1: Health check
info "Testing /health endpoint..."

health_resp=$(curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 127.0.0.1" "$BASE_URL/health")
if echo "$health_resp" | grep -q OK; then
  pass "/health returns OK"
else
  fail "/health did not return OK"
  echo -e "${YELLOW}DEBUG /health response:${NC} $health_resp"
fi

# Preflight: normalize runtime config so tests are deterministic
info "Resetting test_mode=false before integration scenarios..."
curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 127.0.0.1" -X POST \
  -H "Authorization: Bearer $SHUMA_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"test_mode": false}' \
  "$BASE_URL/admin/config" > /dev/null || true

info "Resetting GEO policy lists to empty defaults..."
curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 127.0.0.1" -X POST \
  -H "Authorization: Bearer $SHUMA_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"geo_risk":[],"geo_allow":[],"geo_challenge":[],"geo_maze":[],"geo_block":[]}' \
  "$BASE_URL/admin/config" > /dev/null || true

info "Resetting whitelist/path whitelist to empty defaults..."
curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 127.0.0.1" -X POST \
  -H "Authorization: Bearer $SHUMA_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"whitelist":[],"path_whitelist":[]}' \
  "$BASE_URL/admin/config" > /dev/null || true

info "Clearing bans for integration test IPs..."
for ip in "${TEST_HONEYPOT_IP}" 10.0.0.99 10.0.0.100 10.0.0.150 10.0.0.210 10.0.0.211 10.0.0.212; do
  curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 127.0.0.1" \
    -H "Authorization: Bearer $SHUMA_API_KEY" \
    "$BASE_URL/admin/unban?ip=${ip}" > /dev/null || true
done

info "Resolving configured honeypot path..."
config_snapshot=$(curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "Authorization: Bearer $SHUMA_API_KEY" "$BASE_URL/admin/config")
resolved_honeypot=$(python3 -c 'import json,sys
try:
    data=json.loads(sys.stdin.read())
except Exception:
    print("")
    raise SystemExit(0)
paths=data.get("honeypots") or []
print(paths[0] if paths else "")' <<< "$config_snapshot")
if [[ -n "$resolved_honeypot" ]]; then
  HONEYPOT_PATH="$resolved_honeypot"
fi
info "Using honeypot path: $HONEYPOT_PATH"

# Test 2: PoW challenge (if enabled)
info "Testing PoW challenge..."
pow_resp=$(curl -s -w "HTTPSTATUS:%{http_code}" "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 127.0.0.1" "$BASE_URL/pow")
pow_body=$(echo "$pow_resp" | sed -e 's/HTTPSTATUS:.*//')
pow_status=$(echo "$pow_resp" | tr -d '\n' | sed -e 's/.*HTTPSTATUS://')
if [[ "$pow_status" == "404" ]]; then
  info "PoW disabled; skipping PoW verification"
elif ! python3 -c 'import json,sys; json.loads(sys.stdin.read())' <<< "$pow_body" >/dev/null 2>&1; then
  fail "PoW challenge did not return JSON (check forwarded secret and test_mode)"
  echo -e "${YELLOW}DEBUG /pow status:${NC} $pow_status"
  echo -e "${YELLOW}DEBUG /pow body:${NC} $pow_body"
else
  seed=$(python3 -c 'import json,sys; print(json.loads(sys.stdin.read())["seed"])' <<< "$pow_body")
  difficulty=$(python3 -c 'import json,sys; print(int(json.loads(sys.stdin.read())["difficulty"]))' <<< "$pow_body")
  nonce=$(python3 - "$seed" "$difficulty" <<'PY'
import hashlib
import sys

seed = sys.argv[1]
bits = int(sys.argv[2])
max_iter = 5000000

def has_leading_zero_bits(h, bits):
    remaining = bits
    for b in h:
        if remaining <= 0:
            return True
        if remaining >= 8:
            if b != 0:
                return False
            remaining -= 8
        else:
            mask = 0xFF << (8 - remaining)
            return (b & mask) == 0
    return True

nonce = 0
while True:
    msg = f"{seed}:{nonce}".encode()
    digest = hashlib.sha256(msg).digest()
    if has_leading_zero_bits(digest, bits):
        print(nonce)
        break
    nonce += 1
    if nonce >= max_iter:
        print(-1)
        break
PY
)
  if [[ "$nonce" == "-1" ]]; then
    fail "PoW solve exceeded iteration cap"
  else
  payload=$(python3 - "$seed" "$nonce" <<'PY'
import json,sys
seed=sys.argv[1]
nonce=sys.argv[2]
print(json.dumps({"seed": seed, "nonce": nonce}))
PY
)
  verify_resp=$(curl -s -w "HTTPSTATUS:%{http_code}" "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 127.0.0.1" \
    -H "Content-Type: application/json" -X POST -d "$payload" "$BASE_URL/pow/verify")
  verify_status=$(echo "$verify_resp" | tr -d '\n' | sed -e 's/.*HTTPSTATUS://')
  if [[ "$verify_status" == "200" ]]; then
    pass "PoW verification succeeded"
  else
    fail "PoW verification failed"
  fi
  fi
fi

# Test 3: Root endpoint (should return JS challenge or OK)
info "Testing root endpoint..."

root_resp=$(curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: ${TEST_HONEYPOT_IP}" "$BASE_URL/")
if echo "$root_resp" | grep -qE '(js_verified|JavaScript|Verifying|pow|Proof-of-work)'; then
  pass "/ returns JS challenge (PoW or standard)"
else
  fail "/ did not return expected JS challenge page"
  echo -e "${YELLOW}DEBUG / response:${NC} $root_resp"
fi

# Test 4: Honeypot triggers ban
info "Testing honeypot ban..."
honeypot_status=$(curl -s -o /tmp/shuma_honeypot_body.txt -w "%{http_code}" "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: ${TEST_HONEYPOT_IP}" "$BASE_URL$HONEYPOT_PATH")
resp=$(curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: ${TEST_HONEYPOT_IP}" "$BASE_URL/")
if [[ "$honeypot_status" == "403" ]] && echo "$resp" | grep -q 'Access Blocked'; then
  pass "Honeypot triggers ban and / returns Access Blocked"
else
  fail "Honeypot did not trigger ban as expected"
  echo -e "${YELLOW}DEBUG honeypot status:${NC} $honeypot_status"
  echo -e "${YELLOW}DEBUG honeypot body:${NC} $(cat /tmp/shuma_honeypot_body.txt)"
  echo -e "${YELLOW}DEBUG root response:${NC} $resp"
fi

# Test 5: Unban integration test IP via admin API
info "Testing admin unban for integration test IP..."
curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 127.0.0.1" "$BASE_URL/admin/unban?ip=${TEST_HONEYPOT_IP}" -H "Authorization: Bearer $SHUMA_API_KEY" > /dev/null
resp=$(curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: ${TEST_HONEYPOT_IP}" "$BASE_URL/")
if ! echo "$resp" | grep -q 'Access Blocked'; then
  pass "Unban for integration test IP works"
else
  fail "Unban for integration test IP did not work"
fi

# Test 6: Health check after ban/unban
info "Testing /health endpoint again..."
if curl -sf "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 127.0.0.1" "$BASE_URL/health" | grep -q OK; then
  pass "/health returns OK after ban/unban"
else
  fail "/health did not return OK after ban/unban"
fi

# Test 7: Get config via admin API
info "Testing GET /admin/config..."
config_resp=$(curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "Authorization: Bearer $SHUMA_API_KEY" "$BASE_URL/admin/config")
if echo "$config_resp" | grep -q '"test_mode"'; then
  pass "GET /admin/config returns test_mode field"
else
  fail "GET /admin/config did not return test_mode"
  echo -e "${YELLOW}DEBUG config response:${NC} $config_resp"
fi

# Test 8: Enable test mode
info "Testing POST /admin/config to enable test_mode..."
enable_resp=$(curl -s "${FORWARDED_SECRET_HEADER[@]}" -X POST -H "Authorization: Bearer $SHUMA_API_KEY" -H "Content-Type: application/json" \
  -d '{"test_mode": true}' "$BASE_URL/admin/config")
if echo "$enable_resp" | grep -q '"test_mode":true'; then
  pass "POST /admin/config enables test_mode"
else
  fail "POST /admin/config did not enable test_mode"
  echo -e "${YELLOW}DEBUG enable response:${NC} $enable_resp"
fi

# Test 9: Challenge flow is single-use (incorrect then replay expires)
info "Testing /challenge single-use flow..."
challenge_page=$(curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 10.0.0.150" "$BASE_URL/challenge")
if echo "$challenge_page" | grep -q 'Puzzle'; then
  pass "GET /challenge returns challenge page in test mode"
else
  fail "GET /challenge did not return challenge page in test mode"
  echo -e "${YELLOW}DEBUG challenge page:${NC} $challenge_page"
fi

challenge_seed=$(python3 -c 'import re,sys; html=sys.stdin.read(); m=re.search(r"name=\"seed\" value=\"([^\"]+)\"", html); print(m.group(1) if m else "")' <<< "$challenge_page")
challenge_output=$(python3 -c 'import re,sys; html=sys.stdin.read(); m=re.search(r"name=\"output\"[^>]*value=\"([^\"]+)\"", html); print(m.group(1) if m else "")' <<< "$challenge_page")

if [[ -z "$challenge_seed" || -z "$challenge_output" ]]; then
  fail "Could not parse challenge seed/output from page"
  echo -e "${YELLOW}DEBUG parsed seed:${NC} $challenge_seed"
  echo -e "${YELLOW}DEBUG parsed output:${NC} $challenge_output"
else
  incorrect_resp=$(curl -s -w "HTTPSTATUS:%{http_code}" "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 10.0.0.150" \
    --data-urlencode "seed=$challenge_seed" \
    --data-urlencode "output=$challenge_output" \
    -X POST "$BASE_URL/challenge")
  incorrect_body=$(echo "$incorrect_resp" | sed -e 's/HTTPSTATUS:.*//')
  incorrect_status=$(echo "$incorrect_resp" | tr -d '\n' | sed -e 's/.*HTTPSTATUS://')
  if [[ "$incorrect_status" == "403" ]] && echo "$incorrect_body" | grep -q 'Incorrect'; then
    pass "POST /challenge returns Incorrect for wrong answer"
  else
    fail "POST /challenge did not return expected Incorrect response"
    echo -e "${YELLOW}DEBUG incorrect status:${NC} $incorrect_status"
    echo -e "${YELLOW}DEBUG incorrect body:${NC} $incorrect_body"
  fi

  replay_resp=$(curl -s -w "HTTPSTATUS:%{http_code}" "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 10.0.0.150" \
    --data-urlencode "seed=$challenge_seed" \
    --data-urlencode "output=$challenge_output" \
    -X POST "$BASE_URL/challenge")
  replay_body=$(echo "$replay_resp" | sed -e 's/HTTPSTATUS:.*//')
  replay_status=$(echo "$replay_resp" | tr -d '\n' | sed -e 's/.*HTTPSTATUS://')
  if [[ "$replay_status" == "403" ]] && echo "$replay_body" | grep -q 'Expired'; then
    pass "Replay submit returns Expired for consumed seed"
  else
    fail "Replay submit did not return expected Expired response"
    echo -e "${YELLOW}DEBUG replay status:${NC} $replay_status"
    echo -e "${YELLOW}DEBUG replay body:${NC} $replay_body"
  fi
fi

# Test 10: Test mode allows honeypot access without blocking
info "Testing test_mode behavior (honeypot should not block)..."
# First, unban the test IP to ensure clean state
curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "Authorization: Bearer $SHUMA_API_KEY" "$BASE_URL/admin/unban?ip=10.0.0.99" > /dev/null
# Hit honeypot with test IP
honeypot_resp=$(curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 10.0.0.99" "$BASE_URL$HONEYPOT_PATH")
if echo "$honeypot_resp" | grep -q 'TEST MODE'; then
  pass "Test mode returns TEST MODE response for honeypot"
else
  fail "Test mode did not return expected TEST MODE response"
  echo -e "${YELLOW}DEBUG honeypot response:${NC} $honeypot_resp"
fi

# Verify IP was NOT actually banned
subsequent_resp=$(curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 10.0.0.99" "$BASE_URL/")
if echo "$subsequent_resp" | grep -q 'TEST MODE'; then
  pass "Test mode: IP not actually banned after honeypot"
else
  fail "Test mode: IP was banned when it should not have been"
  echo -e "${YELLOW}DEBUG subsequent response:${NC} $subsequent_resp"
fi

# Test 11: Disable test mode and verify blocking resumes
info "Testing POST /admin/config to disable test_mode..."
disable_resp=$(curl -s "${FORWARDED_SECRET_HEADER[@]}" -X POST -H "Authorization: Bearer $SHUMA_API_KEY" -H "Content-Type: application/json" \
  -d '{"test_mode": false}' "$BASE_URL/admin/config")
if echo "$disable_resp" | grep -q '"test_mode":false'; then
  pass "POST /admin/config disables test_mode"
else
  fail "POST /admin/config did not disable test_mode"
  echo -e "${YELLOW}DEBUG disable response:${NC} $disable_resp"
fi

# Test 12: Verify blocking works again after test mode disabled
info "Testing that blocking resumes after test_mode disabled..."
# Unban first to get clean state
curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "Authorization: Bearer $SHUMA_API_KEY" "$BASE_URL/admin/unban?ip=10.0.0.100" > /dev/null
# Hit honeypot - should now actually ban
curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 10.0.0.100" "$BASE_URL$HONEYPOT_PATH" > /dev/null
block_resp=$(curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 10.0.0.100" "$BASE_URL/")
if echo "$block_resp" | grep -q 'Access Blocked'; then
  pass "Blocking resumes: honeypot triggers real ban after test_mode disabled"
else
  fail "Blocking did not resume after test_mode disabled"
  echo -e "${YELLOW}DEBUG block response:${NC} $block_resp"
fi

# Cleanup: unban test IPs
curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "Authorization: Bearer $SHUMA_API_KEY" "$BASE_URL/admin/unban?ip=10.0.0.99" > /dev/null
curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "Authorization: Bearer $SHUMA_API_KEY" "$BASE_URL/admin/unban?ip=10.0.0.100" > /dev/null
curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "Authorization: Bearer $SHUMA_API_KEY" "$BASE_URL/admin/unban?ip=10.0.0.150" > /dev/null

# Test 13-15: GEO policy routing (requires trusted forwarded headers)
if [[ -z "${SHUMA_FORWARDED_IP_SECRET:-}" ]]; then
  info "Skipping GEO policy routing tests (SHUMA_FORWARDED_IP_SECRET is not set)"
else
  info "Testing GEO policy route: challenge tier..."
  geo_challenge_cfg=$(curl -s "${FORWARDED_SECRET_HEADER[@]}" -X POST -H "Authorization: Bearer $SHUMA_API_KEY" -H "Content-Type: application/json" \
    -d '{"geo_risk":[],"geo_allow":[],"geo_challenge":["BR"],"geo_maze":[],"geo_block":[]}' "$BASE_URL/admin/config")
  if ! echo "$geo_challenge_cfg" | grep -q '"geo_challenge":\["BR"\]'; then
    fail "Failed to apply GEO challenge policy config"
    echo -e "${YELLOW}DEBUG geo challenge config:${NC} $geo_challenge_cfg"
  else
    geo_challenge_resp=$(curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 10.0.0.210" -H "X-Geo-Country: BR" "$BASE_URL/")
    if echo "$geo_challenge_resp" | grep -q 'Puzzle'; then
      pass "GEO challenge tier routes request to challenge page"
    else
      fail "GEO challenge tier did not route to challenge page"
      echo -e "${YELLOW}DEBUG geo challenge response:${NC} $geo_challenge_resp"
    fi
  fi

  info "Testing GEO policy route: maze tier..."
  geo_maze_cfg=$(curl -s "${FORWARDED_SECRET_HEADER[@]}" -X POST -H "Authorization: Bearer $SHUMA_API_KEY" -H "Content-Type: application/json" \
    -d '{"geo_risk":[],"geo_allow":[],"geo_challenge":[],"geo_maze":["RU"],"geo_block":[],"maze_enabled":true,"maze_auto_ban":false}' "$BASE_URL/admin/config")
  if ! echo "$geo_maze_cfg" | grep -q '"geo_maze":\["RU"\]'; then
    fail "Failed to apply GEO maze policy config"
    echo -e "${YELLOW}DEBUG geo maze config:${NC} $geo_maze_cfg"
  else
    geo_maze_resp=$(curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 10.0.0.211" -H "X-Geo-Country: RU" "$BASE_URL/")
    if echo "$geo_maze_resp" | grep -q '/maze/'; then
      pass "GEO maze tier routes request to maze page"
    else
      fail "GEO maze tier did not route to maze page"
      echo -e "${YELLOW}DEBUG geo maze response:${NC} $geo_maze_resp"
    fi
  fi

  info "Testing GEO policy route: block tier..."
  geo_block_cfg=$(curl -s "${FORWARDED_SECRET_HEADER[@]}" -X POST -H "Authorization: Bearer $SHUMA_API_KEY" -H "Content-Type: application/json" \
    -d '{"geo_risk":[],"geo_allow":[],"geo_challenge":[],"geo_maze":[],"geo_block":["KP"]}' "$BASE_URL/admin/config")
  if ! echo "$geo_block_cfg" | grep -q '"geo_block":\["KP"\]'; then
    fail "Failed to apply GEO block policy config"
    echo -e "${YELLOW}DEBUG geo block config:${NC} $geo_block_cfg"
  else
    geo_block_resp=$(curl -s -w "HTTPSTATUS:%{http_code}" "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 10.0.0.212" -H "X-Geo-Country: KP" "$BASE_URL/")
    geo_block_body=$(echo "$geo_block_resp" | sed -e 's/HTTPSTATUS:.*//')
    geo_block_status=$(echo "$geo_block_resp" | tr -d '\n' | sed -e 's/.*HTTPSTATUS://')
    if [[ "$geo_block_status" == "403" ]] && echo "$geo_block_body" | grep -qE 'Access Blocked|Access Restricted'; then
      pass "GEO block tier returns 403 block page"
    else
      fail "GEO block tier did not block as expected"
      echo -e "${YELLOW}DEBUG geo block status:${NC} $geo_block_status"
      echo -e "${YELLOW}DEBUG geo block body:${NC} $geo_block_body"
    fi
  fi

  info "Resetting GEO policy lists after routing tests..."
  curl -s "${FORWARDED_SECRET_HEADER[@]}" -X POST -H "Authorization: Bearer $SHUMA_API_KEY" -H "Content-Type: application/json" \
    -d '{"geo_risk":[],"geo_allow":[],"geo_challenge":[],"geo_maze":[],"geo_block":[]}' "$BASE_URL/admin/config" > /dev/null || true
fi

# Test 16: Prometheus metrics endpoint
info "Testing GET /metrics (Prometheus format)..."
metrics_resp=$(curl -s "${FORWARDED_SECRET_HEADER[@]}" "$BASE_URL/metrics")
if echo "$metrics_resp" | grep -q 'bot_defence_requests_total'; then
  pass "/metrics returns Prometheus-formatted metrics"
else
  fail "/metrics did not return expected Prometheus format"
  echo -e "${YELLOW}DEBUG metrics response:${NC} $metrics_resp"
fi

# Verify metrics contain expected counters
if echo "$metrics_resp" | grep -q 'bot_defence_bans_total'; then
  pass "/metrics includes ban counters"
else
  fail "/metrics missing ban counters"
fi

# Test 17: CDP report endpoint exists
info "Testing POST /cdp-report endpoint..."
cdp_report='{"cdp_detected":true,"score":0.5,"checks":["webdriver"]}'
cdp_resp=$(curl -s "${FORWARDED_SECRET_HEADER[@]}" -X POST -H "Content-Type: application/json" -H "X-Forwarded-For: 10.0.0.200" -d "$cdp_report" "$BASE_URL/cdp-report")
if echo "$cdp_resp" | grep -qiE 'received|disabled|detected'; then
  pass "/cdp-report endpoint accepts detection reports"
else
  fail "/cdp-report endpoint not working"
  echo -e "${YELLOW}DEBUG cdp response:${NC} $cdp_resp"
fi

# Test 18: CDP report with high score triggers action (when enabled)
info "Testing CDP auto-ban with high score..."
cdp_high='{"cdp_detected":true,"score":0.95,"checks":["webdriver","automation_props","cdp_timing"]}'
cdp_high_resp=$(curl -s "${FORWARDED_SECRET_HEADER[@]}" -X POST -H "Content-Type: application/json" -H "X-Forwarded-For: 10.0.0.201" -d "$cdp_high" "$BASE_URL/cdp-report")
if echo "$cdp_high_resp" | grep -qiE 'banned|received|disabled'; then
  pass "/cdp-report handles high-score detection"
else
  fail "/cdp-report failed for high-score detection"
  echo -e "${YELLOW}DEBUG cdp high response:${NC} $cdp_high_resp"
fi

# Test 19: CDP config available via admin API
info "Testing CDP config in /admin/cdp..."
cdp_config=$(curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "Authorization: Bearer $SHUMA_API_KEY" "$BASE_URL/admin/cdp")
if echo "$cdp_config" | grep -qE '"enabled"|cdp_detection'; then
  pass "/admin/cdp returns CDP configuration"
else
  fail "/admin/cdp not returning expected config"
  echo -e "${YELLOW}DEBUG cdp config:${NC} $cdp_config"
fi

# Test 20: CDP stats counters reflect reports
info "Testing CDP stats counters..."
cdp_stats_resp=$(curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "Authorization: Bearer $SHUMA_API_KEY" "$BASE_URL/admin/cdp")
cdp_total_detections=$(python3 -c 'import json,sys; d=json.loads(sys.stdin.read()); print(int(d.get("stats",{}).get("total_detections",0)))' <<< "$cdp_stats_resp")
cdp_total_autobans=$(python3 -c 'import json,sys; d=json.loads(sys.stdin.read()); print(int(d.get("stats",{}).get("auto_bans",0)))' <<< "$cdp_stats_resp")
if [[ "$cdp_total_detections" -ge 2 ]] && [[ "$cdp_total_autobans" -ge 1 ]]; then
  pass "CDP stats counters increment for detections and auto-bans"
else
  fail "CDP stats counters did not increment as expected"
  echo -e "${YELLOW}DEBUG /admin/cdp:${NC} $cdp_stats_resp"
fi

# Test 21: unban_ip function works via admin endpoint  
info "Testing unban functionality..."
# First ban an IP
curl -s "${FORWARDED_SECRET_HEADER[@]}" -X POST -H "Authorization: Bearer $SHUMA_API_KEY" "$BASE_URL/admin/ban?ip=10.0.0.202&reason=test" > /dev/null
# Then unban it
unban_resp=$(curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "Authorization: Bearer $SHUMA_API_KEY" "$BASE_URL/admin/unban?ip=10.0.0.202")
if echo "$unban_resp" | grep -qi 'unbanned'; then
  pass "Unban via admin API works correctly"
else
  fail "Unban did not work as expected"
  echo -e "${YELLOW}DEBUG unban response:${NC} $unban_resp"
fi

# Cleanup: unban test CDP/IPs
curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "Authorization: Bearer $SHUMA_API_KEY" "$BASE_URL/admin/unban?ip=${TEST_HONEYPOT_IP}" > /dev/null
curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "Authorization: Bearer $SHUMA_API_KEY" "$BASE_URL/admin/unban?ip=10.0.0.200" > /dev/null
curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "Authorization: Bearer $SHUMA_API_KEY" "$BASE_URL/admin/unban?ip=10.0.0.201" > /dev/null

echo -e "\n${GREEN}All integration tests complete.${NC}"

if [[ "$FAILURES" -ne 0 ]]; then
  echo -e "${RED}${FAILURES} integration test(s) failed.${NC}"
  exit 1
fi
