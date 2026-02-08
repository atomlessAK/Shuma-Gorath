#!/bin/bash
# test_spin_http.sh - HTTP integration tests for WASM Bot Trap Spin app
# Usage: bash test_spin_http.sh [base_url]
# Default base_url: http://127.0.0.1:3000

BASE_URL="${1:-http://127.0.0.1:3000}"
SHUMA_API_KEY="${SHUMA_API_KEY:-changeme-dev-only-api-key}"

set -e

echo "[1] Test: root endpoint (should pass bot trap)"
curl -i "$BASE_URL/"
echo

echo "[2] Test: honeypot endpoint (should block)"
curl -i "$BASE_URL/bot-trap"
echo

echo "[3] Test: admin endpoint (unauthorized)"
curl -i "$BASE_URL/admin"
echo

echo "[4] Test: admin endpoint (authorized)"
curl -i -H "Authorization: Bearer $SHUMA_API_KEY" "$BASE_URL/admin"
echo

echo "[5] Test: admin/ban endpoint (authorized)"
curl -i -H "Authorization: Bearer $SHUMA_API_KEY" "$BASE_URL/admin/ban"
echo

echo "[6] Test: admin/analytics endpoint (authorized)"
curl -i -H "Authorization: Bearer $SHUMA_API_KEY" "$BASE_URL/admin/analytics"
echo
