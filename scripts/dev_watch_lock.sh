#!/bin/bash
set -euo pipefail

if [[ $# -eq 0 ]]; then
  echo "Usage: $0 <command> [args...]" >&2
  exit 1
fi

LOCK_FILE=".spin/dev-watch.lock"

process_command_for_pid() {
  local pid="$1"
  ps -o command= -p "${pid}" 2>/dev/null || true
}

is_dev_watch_holder_command() {
  local command="$1"
  if [[ "${command}" == *"scripts/dev_watch_lock.sh"* ]]; then
    return 0
  fi
  if [[ "${command}" == *"cargo-watch watch --poll"* ]] && [[ "${command}" == *"-w src -w dashboard -w spin.toml"* ]]; then
    return 0
  fi
  return 1
}

acquire_lock() {
  mkdir -p .spin

  if ln -s "$$" "${LOCK_FILE}" 2>/dev/null; then
    return
  fi

  local existing_pid
  existing_pid="$(readlink "${LOCK_FILE}" 2>/dev/null || true)"
  if [[ -n "${existing_pid}" ]] && [[ "${existing_pid}" =~ ^[0-9]+$ ]] && kill -0 "${existing_pid}" 2>/dev/null; then
    local existing_command
    existing_command="$(process_command_for_pid "${existing_pid}")"
    if [[ -z "${existing_command}" ]]; then
      echo "A dev watcher lock exists for live pid ${existing_pid}, but process inspection is unavailable. Run 'make stop' to clear it." >&2
      exit 1
    fi
    if is_dev_watch_holder_command "${existing_command}"; then
      echo "A dev watcher is already running (pid ${existing_pid}). Stop it first with 'make stop'." >&2
      exit 1
    fi
  fi

  rm -f "${LOCK_FILE}"
  if ! ln -s "$$" "${LOCK_FILE}" 2>/dev/null; then
    echo "Another dev watcher is starting. Retry in a moment." >&2
    exit 1
  fi
}

cleanup() {
  if [[ "$(readlink "${LOCK_FILE}" 2>/dev/null || true)" == "$$" ]]; then
    rm -f "${LOCK_FILE}"
  fi
}

acquire_lock
trap cleanup EXIT INT TERM

"$@"
