#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PLAYWRIGHT_BROWSER_CACHE="${PLAYWRIGHT_BROWSERS_PATH:-${ROOT_DIR}/.cache/ms-playwright}"
PLAYWRIGHT_LOCAL_HOME="${PLAYWRIGHT_HOME:-${ROOT_DIR}/.cache/playwright-home}"

mkdir -p "${PLAYWRIGHT_BROWSER_CACHE}" \
         "${PLAYWRIGHT_LOCAL_HOME}/.config" \
         "${PLAYWRIGHT_LOCAL_HOME}/Library/Application Support/Chromium/Crashpad"

export PLAYWRIGHT_BROWSERS_PATH="${PLAYWRIGHT_BROWSER_CACHE}"
export HOME="${PLAYWRIGHT_LOCAL_HOME}"
export CFFIXED_USER_HOME="${PLAYWRIGHT_LOCAL_HOME}"
export XDG_CONFIG_HOME="${PLAYWRIGHT_LOCAL_HOME}/.config"

PLAYWRIGHT_CHROMIUM_PATH="$(
  corepack pnpm exec node -e "const { chromium } = require('@playwright/test'); process.stdout.write(chromium.executablePath() || '');" 2>/dev/null || true
)"
if [[ -z "${PLAYWRIGHT_CHROMIUM_PATH}" || ! -x "${PLAYWRIGHT_CHROMIUM_PATH}" ]]; then
  echo "Installing Playwright Chromium runtime into ${PLAYWRIGHT_BROWSERS_PATH}..."
  corepack pnpm exec playwright install chromium
fi

if ! corepack pnpm exec node scripts/tests/verify_playwright_launch.mjs; then
  if [[ "${PLAYWRIGHT_SANDBOX_ALLOW_SKIP:-0}" == "1" ]]; then
    echo "Playwright Chromium launch is blocked in this environment; skipping dashboard e2e because PLAYWRIGHT_SANDBOX_ALLOW_SKIP=1."
    exit 0
  fi
  exit 1
fi

exec corepack pnpm run test:dashboard:e2e:raw "$@"
