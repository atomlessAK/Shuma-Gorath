#!/bin/bash
# Seed/backfill local Spin KV config from config/defaults.env.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEFAULTS_FILE="${ROOT_DIR}/config/defaults.env"
DB_PATH="${ROOT_DIR}/.spin/sqlite_key_value.db"
STORE_NAME="default"
CONFIG_KEY="config:default"

if [[ ! -f "${DEFAULTS_FILE}" ]]; then
  echo "❌ Missing defaults file: ${DEFAULTS_FILE}" >&2
  exit 1
fi

if ! command -v sqlite3 >/dev/null 2>&1; then
  echo "❌ sqlite3 is required for config-seed." >&2
  exit 1
fi

if ! command -v python3 >/dev/null 2>&1; then
  echo "❌ python3 is required for config-seed." >&2
  exit 1
fi

mkdir -p "$(dirname "${DB_PATH}")"

# shellcheck disable=SC1090
set -a
source "${DEFAULTS_FILE}"
set +a

bool_norm() {
  local v
  v="$(printf '%s' "${1:-}" | tr '[:upper:]' '[:lower:]')"
  case "${v}" in
    1|true|yes|on) echo "true" ;;
    0|false|no|off) echo "false" ;;
    *)
      echo "❌ Invalid boolean value: ${1}" >&2
      exit 1
      ;;
  esac
}

make_tmp_file() {
  local prefix="$1"
  local tmp=""
  tmp="$(mktemp "/tmp/${prefix}.XXXXXX" 2>/dev/null)" || \
    tmp="$(mktemp -t "${prefix}" 2>/dev/null)" || {
      echo "❌ Failed to allocate temp file for ${prefix}" >&2
      exit 1
    }
  printf '%s' "${tmp}"
}

sqlite3 "${DB_PATH}" <<'SQL'
CREATE TABLE IF NOT EXISTS spin_key_value (
  store TEXT NOT NULL,
  key   TEXT NOT NULL,
  value BLOB NOT NULL,
  PRIMARY KEY (store, key)
);
SQL

tmp_json="$(make_tmp_file "shuma-config-seed")"
tmp_merged="$(make_tmp_file "shuma-config-merged")"
tmp_existing="$(make_tmp_file "shuma-config-existing")"
trap 'rm -f "${tmp_json}" "${tmp_merged}" "${tmp_existing}"' EXIT

cat > "${tmp_json}" <<EOF
{
  "ban_duration": ${SHUMA_BAN_DURATION},
  "ban_durations": {
    "honeypot": ${SHUMA_BAN_DURATION_HONEYPOT},
    "rate_limit": ${SHUMA_BAN_DURATION_RATE_LIMIT},
    "browser": ${SHUMA_BAN_DURATION_BROWSER},
    "admin": ${SHUMA_BAN_DURATION_ADMIN},
    "cdp": ${SHUMA_BAN_DURATION_CDP}
  },
  "rate_limit": ${SHUMA_RATE_LIMIT},
  "honeypot_enabled": $(bool_norm "${SHUMA_HONEYPOT_ENABLED}"),
  "honeypots": ${SHUMA_HONEYPOTS},
  "browser_block": ${SHUMA_BROWSER_BLOCK},
  "browser_whitelist": ${SHUMA_BROWSER_WHITELIST},
  "geo_risk": ${SHUMA_GEO_RISK_COUNTRIES},
  "geo_allow": ${SHUMA_GEO_ALLOW_COUNTRIES},
  "geo_challenge": ${SHUMA_GEO_CHALLENGE_COUNTRIES},
  "geo_maze": ${SHUMA_GEO_MAZE_COUNTRIES},
  "geo_block": ${SHUMA_GEO_BLOCK_COUNTRIES},
  "whitelist": ${SHUMA_WHITELIST},
  "path_whitelist": ${SHUMA_PATH_WHITELIST},
  "test_mode": $(bool_norm "${SHUMA_TEST_MODE}"),
  "maze_enabled": $(bool_norm "${SHUMA_MAZE_ENABLED}"),
  "maze_auto_ban": $(bool_norm "${SHUMA_MAZE_AUTO_BAN}"),
  "maze_auto_ban_threshold": ${SHUMA_MAZE_AUTO_BAN_THRESHOLD},
  "maze_rollout_phase": "${SHUMA_MAZE_ROLLOUT_PHASE}",
  "maze_token_ttl_seconds": ${SHUMA_MAZE_TOKEN_TTL_SECONDS},
  "maze_token_max_depth": ${SHUMA_MAZE_TOKEN_MAX_DEPTH},
  "maze_token_branch_budget": ${SHUMA_MAZE_TOKEN_BRANCH_BUDGET},
  "maze_replay_ttl_seconds": ${SHUMA_MAZE_REPLAY_TTL_SECONDS},
  "maze_entropy_window_seconds": ${SHUMA_MAZE_ENTROPY_WINDOW_SECONDS},
  "maze_client_expansion_enabled": $(bool_norm "${SHUMA_MAZE_CLIENT_EXPANSION_ENABLED}"),
  "maze_checkpoint_every_nodes": ${SHUMA_MAZE_CHECKPOINT_EVERY_NODES},
  "maze_checkpoint_every_ms": ${SHUMA_MAZE_CHECKPOINT_EVERY_MS},
  "maze_step_ahead_max": ${SHUMA_MAZE_STEP_AHEAD_MAX},
  "maze_no_js_fallback_max_depth": ${SHUMA_MAZE_NO_JS_FALLBACK_MAX_DEPTH},
  "maze_micro_pow_enabled": $(bool_norm "${SHUMA_MAZE_MICRO_POW_ENABLED}"),
  "maze_micro_pow_depth_start": ${SHUMA_MAZE_MICRO_POW_DEPTH_START},
  "maze_micro_pow_base_difficulty": ${SHUMA_MAZE_MICRO_POW_BASE_DIFFICULTY},
  "maze_max_concurrent_global": ${SHUMA_MAZE_MAX_CONCURRENT_GLOBAL},
  "maze_max_concurrent_per_ip_bucket": ${SHUMA_MAZE_MAX_CONCURRENT_PER_IP_BUCKET},
  "maze_max_response_bytes": ${SHUMA_MAZE_MAX_RESPONSE_BYTES},
  "maze_max_response_duration_ms": ${SHUMA_MAZE_MAX_RESPONSE_DURATION_MS},
  "maze_server_visible_links": ${SHUMA_MAZE_SERVER_VISIBLE_LINKS},
  "maze_max_links": ${SHUMA_MAZE_MAX_LINKS},
  "maze_max_paragraphs": ${SHUMA_MAZE_MAX_PARAGRAPHS},
  "maze_path_entropy_segment_len": ${SHUMA_MAZE_PATH_ENTROPY_SEGMENT_LEN},
  "maze_covert_decoys_enabled": $(bool_norm "${SHUMA_MAZE_COVERT_DECOYS_ENABLED}"),
  "maze_seed_provider": "${SHUMA_MAZE_SEED_PROVIDER}",
  "maze_seed_refresh_interval_seconds": ${SHUMA_MAZE_SEED_REFRESH_INTERVAL_SECONDS},
  "maze_seed_refresh_rate_limit_per_hour": ${SHUMA_MAZE_SEED_REFRESH_RATE_LIMIT_PER_HOUR},
  "maze_seed_refresh_max_sources": ${SHUMA_MAZE_SEED_REFRESH_MAX_SOURCES},
  "maze_seed_metadata_only": $(bool_norm "${SHUMA_MAZE_SEED_METADATA_ONLY}"),
  "robots_enabled": $(bool_norm "${SHUMA_ROBOTS_ENABLED}"),
  "robots_block_ai_training": $(bool_norm "${SHUMA_ROBOTS_BLOCK_AI_TRAINING}"),
  "robots_block_ai_search": $(bool_norm "${SHUMA_ROBOTS_BLOCK_AI_SEARCH}"),
  "robots_allow_search_engines": $(bool_norm "${SHUMA_ROBOTS_ALLOW_SEARCH_ENGINES}"),
  "ai_policy_block_training": $(bool_norm "${SHUMA_AI_POLICY_BLOCK_TRAINING}"),
  "ai_policy_block_search": $(bool_norm "${SHUMA_AI_POLICY_BLOCK_SEARCH}"),
  "ai_policy_allow_search_engines": $(bool_norm "${SHUMA_AI_POLICY_ALLOW_SEARCH_ENGINES}"),
  "robots_crawl_delay": ${SHUMA_ROBOTS_CRAWL_DELAY},
  "cdp_detection_enabled": $(bool_norm "${SHUMA_CDP_DETECTION_ENABLED}"),
  "cdp_auto_ban": $(bool_norm "${SHUMA_CDP_AUTO_BAN}"),
  "cdp_detection_threshold": ${SHUMA_CDP_DETECTION_THRESHOLD},
  "js_required_enforced": $(bool_norm "${SHUMA_JS_REQUIRED_ENFORCED}"),
  "pow_enabled": $(bool_norm "${SHUMA_POW_ENABLED}"),
  "pow_difficulty": ${SHUMA_POW_DIFFICULTY},
  "pow_ttl_seconds": ${SHUMA_POW_TTL_SECONDS},
  "challenge_enabled": $(bool_norm "${SHUMA_CHALLENGE_ENABLED}"),
  "challenge_transform_count": ${SHUMA_CHALLENGE_TRANSFORM_COUNT},
  "challenge_risk_threshold": ${SHUMA_CHALLENGE_RISK_THRESHOLD},
  "botness_maze_threshold": ${SHUMA_BOTNESS_MAZE_THRESHOLD},
  "botness_weights": {
    "js_required": ${SHUMA_BOTNESS_WEIGHT_JS_REQUIRED},
    "geo_risk": ${SHUMA_BOTNESS_WEIGHT_GEO_RISK},
    "rate_medium": ${SHUMA_BOTNESS_WEIGHT_RATE_MEDIUM},
    "rate_high": ${SHUMA_BOTNESS_WEIGHT_RATE_HIGH},
    "maze_behavior": ${SHUMA_BOTNESS_WEIGHT_MAZE_BEHAVIOR}
  },
  "defence_modes": {
    "rate": "${SHUMA_MODE_RATE}",
    "geo": "${SHUMA_MODE_GEO}",
    "js": "${SHUMA_MODE_JS}"
  },
  "provider_backends": {
    "rate_limiter": "${SHUMA_PROVIDER_RATE_LIMITER}",
    "ban_store": "${SHUMA_PROVIDER_BAN_STORE}",
    "challenge_engine": "${SHUMA_PROVIDER_CHALLENGE_ENGINE}",
    "maze_tarpit": "${SHUMA_PROVIDER_MAZE_TARPIT}",
    "fingerprint_signal": "${SHUMA_PROVIDER_FINGERPRINT_SIGNAL}"
  },
  "edge_integration_mode": "${SHUMA_EDGE_INTEGRATION_MODE}"
}
EOF

existing_json="$(sqlite3 "${DB_PATH}" "SELECT CAST(value AS TEXT) FROM spin_key_value WHERE store='${STORE_NAME}' AND key='${CONFIG_KEY}' LIMIT 1;")"

if [[ -z "${existing_json}" ]]; then
  sqlite3 "${DB_PATH}" "INSERT INTO spin_key_value(store,key,value) VALUES('${STORE_NAME}','${CONFIG_KEY}',readfile('${tmp_json}'));"
  echo "✅ Seeded KV config from config/defaults.env into ${CONFIG_KEY}"
  exit 0
fi

merged_changed="$(
  printf '%s' "${existing_json}" > "${tmp_existing}"
  python3 - "${tmp_json}" "${tmp_merged}" "${tmp_existing}" <<'PY'
import json
import sys

defaults_path, merged_path, existing_path = sys.argv[1], sys.argv[2], sys.argv[3]
with open(defaults_path, "r", encoding="utf-8") as handle:
    defaults = json.load(handle)

with open(existing_path, "r", encoding="utf-8") as handle:
    existing_raw = handle.read().strip()
if not existing_raw:
    merged = defaults
    changed = True
else:
    try:
        existing = json.loads(existing_raw)
    except json.JSONDecodeError as exc:
        raise SystemExit(f"❌ Existing KV config is not valid JSON: {exc}")

    def merge_missing(existing_value, defaults_value):
        if isinstance(existing_value, dict) and isinstance(defaults_value, dict):
            merged_map = dict(existing_value)
            changed_local = False
            for key, default_child in defaults_value.items():
                if key in merged_map:
                    merged_child, changed_child = merge_missing(merged_map[key], default_child)
                    merged_map[key] = merged_child
                    changed_local = changed_local or changed_child
                else:
                    merged_map[key] = default_child
                    changed_local = True
            return merged_map, changed_local
        return existing_value, False

    merged, changed = merge_missing(existing, defaults)

with open(merged_path, "w", encoding="utf-8") as handle:
    json.dump(merged, handle, separators=(",", ":"))

sys.stdout.write("1" if changed else "0")
PY
)"

if [[ "${merged_changed}" == "1" ]]; then
  sqlite3 "${DB_PATH}" "UPDATE spin_key_value SET value=readfile('${tmp_merged}') WHERE store='${STORE_NAME}' AND key='${CONFIG_KEY}';"
  echo "✅ Backfilled missing KV config keys from config/defaults.env into ${CONFIG_KEY}"
else
  echo "✅ KV config already seeded/backfilled (${CONFIG_KEY}); no missing keys."
fi
