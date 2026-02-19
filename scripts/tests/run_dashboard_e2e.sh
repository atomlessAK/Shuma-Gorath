#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PLAYWRIGHT_BROWSER_CACHE="${PLAYWRIGHT_BROWSERS_PATH:-${ROOT_DIR}/.cache/ms-playwright}"
PLAYWRIGHT_LOCAL_HOME="${PLAYWRIGHT_HOME:-${ROOT_DIR}/.cache/playwright-home}"
ORIGINAL_HOME="${HOME:-}"
PLAYWRIGHT_FORCE_LOCAL_HOME="${PLAYWRIGHT_FORCE_LOCAL_HOME:-1}"

mkdir -p "${PLAYWRIGHT_BROWSER_CACHE}"

export PLAYWRIGHT_BROWSERS_PATH="${PLAYWRIGHT_BROWSER_CACHE}"
using_local_home=false
if [[ "${PLAYWRIGHT_FORCE_LOCAL_HOME}" == "1" ]]; then
  mkdir -p "${PLAYWRIGHT_LOCAL_HOME}/.config" \
           "${PLAYWRIGHT_LOCAL_HOME}/Library/Application Support/Chromium/Crashpad"
  export HOME="${PLAYWRIGHT_LOCAL_HOME}"
  export CFFIXED_USER_HOME="${PLAYWRIGHT_LOCAL_HOME}"
  export XDG_CONFIG_HOME="${PLAYWRIGHT_LOCAL_HOME}/.config"
  using_local_home=true
fi

PLAYWRIGHT_CHROMIUM_PATH="$(
  corepack pnpm exec node -e "const { chromium } = require('@playwright/test'); process.stdout.write(chromium.executablePath() || '');" 2>/dev/null || true
)"
if [[ -z "${PLAYWRIGHT_CHROMIUM_PATH}" || ! -x "${PLAYWRIGHT_CHROMIUM_PATH}" ]]; then
  echo "Installing Playwright Chromium runtime into ${PLAYWRIGHT_BROWSERS_PATH}..."
  corepack pnpm exec playwright install chromium
fi

status=0
corepack pnpm exec node scripts/tests/verify_playwright_launch.mjs || status=$?
if [[ "$status" -eq 42 && "${using_local_home}" == "true" && -n "${ORIGINAL_HOME}" ]]; then
  echo "Playwright launch failed under repo-local HOME; retrying preflight with system HOME..."
  using_local_home=false
  export HOME="${ORIGINAL_HOME}"
  unset CFFIXED_USER_HOME
  unset XDG_CONFIG_HOME
  status=0
  corepack pnpm exec node scripts/tests/verify_playwright_launch.mjs || status=$?
fi

if [[ "${status:-0}" -ne 0 ]]; then
  if [[ "${PLAYWRIGHT_SANDBOX_ALLOW_SKIP:-0}" == "1" ]]; then
    echo "Playwright Chromium launch is blocked in this environment; skipping dashboard e2e because PLAYWRIGHT_SANDBOX_ALLOW_SKIP=1."
    exit 0
  fi
  exit "$status"
fi

if [[ "${using_local_home}" == "true" ]]; then
  echo "Playwright preflight succeeded with repo-local HOME (${PLAYWRIGHT_LOCAL_HOME})."
else
  echo "Playwright preflight succeeded with system HOME (${HOME})."
fi

exec corepack pnpm run test:dashboard:e2e:raw "$@"
