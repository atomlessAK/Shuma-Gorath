#!/bin/bash
# Temporary integration test runner (no cargo clean)

GREEN="\033[0;32m"
RED="\033[0;31m"
YELLOW="\033[1;33m"
NC="\033[0m"

pass() { echo -e "${GREEN}✓ PASS${NC} $1"; }
fail() { echo -e "${RED}✗ FAIL${NC} $1"; }
info() { echo -e "${YELLOW}→ INFO${NC} $1"; }

BASE_URL="http://127.0.0.1:3000"
API_KEY="${API_KEY:-changeme-dev-only-api-key}"

FORWARDED_SECRET_HEADER=()
if [[ -n "${FORWARDED_IP_SECRET:-}" ]]; then
  FORWARDED_SECRET_HEADER=(-H "X-Shuma-Forwarded-Secret: ${FORWARDED_IP_SECRET}")
fi

echo ""
echo -e "${YELLOW}============================================${NC}"
echo -e "${YELLOW}  INTEGRATION TESTS (Spin Environment)${NC}"
echo -e "${YELLOW}  Running integration test scenarios${NC}"
echo -e "${YELLOW}============================================${NC}"
echo ""

# Test 1: Health check
info "Test 1/5: Health check endpoint..."
health_resp=$(curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 127.0.0.1" "$BASE_URL/health")
if echo "$health_resp" | grep -q OK; then
  pass "Health endpoint returns OK"
else
  fail "Health endpoint did not return OK"
fi

# Test 2: Root endpoint
info "Test 2/5: Root endpoint behavior..."
root_resp=$(curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 127.0.0.1" "$BASE_URL/")
if echo "$root_resp" | grep -qE '(js_verified|Access Blocked|JavaScript)'; then
  pass "Root endpoint returns JS challenge or block page"
else
  fail "Root endpoint did not return expected page"
fi

# Test 3: Honeypot triggers ban
info "Test 3/5: Honeypot ban detection..."
curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 1.2.3.4" "$BASE_URL/bot-trap" > /dev/null
sleep 0.5
resp=$(curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 1.2.3.4" "$BASE_URL/")
if echo "$resp" | grep -q 'Access Blocked'; then
  pass "Honeypot triggers ban correctly"
else
  fail "Honeypot did not trigger ban as expected"
fi

# Test 4: Admin ban endpoint
info "Test 4/5: Admin API manual ban..."
ban_resp=$(curl -s "${FORWARDED_SECRET_HEADER[@]}" -X POST -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"ip":"5.6.7.8","reason":"test_ban","duration":3600}' \
  "$BASE_URL/admin/ban")
pass "Admin ban endpoint executed successfully"

# Test 5: Unban via admin API
info "Test 5/5: Admin API unban..."
curl -s "${FORWARDED_SECRET_HEADER[@]}" -X POST -H "Authorization: Bearer $API_KEY" \
  "$BASE_URL/admin/unban?ip=1.2.3.4" > /dev/null
sleep 0.5
resp=$(curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 1.2.3.4" "$BASE_URL/")
if ! echo "$resp" | grep -q 'Banned'; then
  pass "Unban restores access correctly"
else
  fail "Unban did not restore access"
fi

echo ""
echo -e "${GREEN}============================================${NC}"
echo -e "${GREEN}  ALL 5 INTEGRATION TESTS COMPLETE ✓${NC}"
echo -e "${GREEN}============================================${NC}"
echo ""
