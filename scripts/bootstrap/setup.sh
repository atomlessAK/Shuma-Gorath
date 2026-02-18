#!/bin/bash
# setup.sh - One-command setup for WASM Bot Defence development
#
# Usage: make setup
#
# This script installs all required dependencies for macOS:
#   - Homebrew (if missing)
#   - Rust/Cargo (via rustup)
#   - wasm32-wasip1 target
#   - Fermyon Spin CLI
#   - cargo-watch (for file watching)
#   - Node.js + corepack (for dashboard toolchain)
#   - pnpm dependencies from lockfile
#   - Playwright Chromium runtime (for dashboard e2e)
#
# After setup, run: make dev

set -e

# Colors
GREEN="\033[0;32m"
YELLOW="\033[1;33m"
CYAN="\033[0;36m"
RED="\033[0;31m"
NC="\033[0m"

info() { echo -e "${CYAN}ℹ️  $1${NC}"; }
success() { echo -e "${GREEN}✅ $1${NC}"; }
warn() { echo -e "${YELLOW}⚠️  $1${NC}"; }
error() { echo -e "${RED}❌ $1${NC}"; exit 1; }

ENV_LOCAL_FILE=".env.local"
DEFAULTS_FILE="config/defaults.env"

generate_hex_secret() {
    local bytes="${1:-32}"
    if command -v openssl &> /dev/null; then
        openssl rand -hex "$bytes"
    else
        od -An -N"$bytes" -tx1 /dev/urandom | tr -d ' \n'
    fi
}

read_env_local_value() {
    local key="$1"
    local raw=""
    if [[ -f "$ENV_LOCAL_FILE" ]]; then
        raw="$(grep -E "^${key}=" "$ENV_LOCAL_FILE" | tail -1 || true)"
    fi
    raw="${raw#*=}"
    if [[ ${#raw} -ge 2 ]]; then
        if [[ "${raw:0:1}" == "\"" && "${raw: -1}" == "\"" ]]; then
            raw="${raw:1:${#raw}-2}"
        elif [[ "${raw:0:1}" == "'" && "${raw: -1}" == "'" ]]; then
            raw="${raw:1:${#raw}-2}"
        fi
    fi
    printf '%s' "$raw"
}

upsert_env_local_value() {
    local key="$1"
    local value="$2"
    local tmp_file
    tmp_file="$(mktemp)"
    if [[ -f "$ENV_LOCAL_FILE" ]] && grep -q -E "^${key}=" "$ENV_LOCAL_FILE"; then
        awk -v target_key="$key" -v target_value="$value" '
            $0 ~ ("^" target_key "=") { print target_key "=" target_value; next }
            { print }
        ' "$ENV_LOCAL_FILE" > "$tmp_file"
    else
        if [[ -f "$ENV_LOCAL_FILE" ]]; then
            cat "$ENV_LOCAL_FILE" > "$tmp_file"
        fi
        printf '%s=%s\n' "$key" "$value" >> "$tmp_file"
    fi
    mv "$tmp_file" "$ENV_LOCAL_FILE"
}

normalize_env_local_unquoted_style() {
    local tmp_file
    tmp_file="$(mktemp)"
    awk '
        BEGIN { single_quote = sprintf("%c", 39) }
        /^[A-Za-z_][A-Za-z0-9_]*=/ {
            key = substr($0, 1, index($0, "=") - 1)
            value = substr($0, index($0, "=") + 1)
            if (length(value) >= 2) {
                if (substr(value, 1, 1) == "\"" && substr(value, length(value), 1) == "\"") {
                    value = substr(value, 2, length(value) - 2)
                } else if (substr(value, 1, 1) == single_quote && substr(value, length(value), 1) == single_quote) {
                    value = substr(value, 2, length(value) - 2)
                }
            }
            print key "=" value
            next
        }
        { print }
    ' "$ENV_LOCAL_FILE" > "$tmp_file"
    mv "$tmp_file" "$ENV_LOCAL_FILE"
}

ensure_env_local_file() {
    if [[ ! -f "$ENV_LOCAL_FILE" ]]; then
        info "Creating $ENV_LOCAL_FILE for local development overrides..."
        cat > "$ENV_LOCAL_FILE" <<EOF
# Local development overrides (gitignored)
# Created by `make setup`. Edit values for local development only.
SHUMA_API_KEY=${SHUMA_API_KEY:-}
SHUMA_ADMIN_READONLY_API_KEY=${SHUMA_ADMIN_READONLY_API_KEY:-}
SHUMA_JS_SECRET=${SHUMA_JS_SECRET:-}
SHUMA_POW_SECRET=${SHUMA_POW_SECRET:-}
SHUMA_CHALLENGE_SECRET=${SHUMA_CHALLENGE_SECRET:-}
SHUMA_MAZE_PREVIEW_SECRET=${SHUMA_MAZE_PREVIEW_SECRET:-}
SHUMA_FORWARDED_IP_SECRET=${SHUMA_FORWARDED_IP_SECRET:-}
SHUMA_HEALTH_SECRET=${SHUMA_HEALTH_SECRET:-}
SHUMA_ADMIN_IP_ALLOWLIST=${SHUMA_ADMIN_IP_ALLOWLIST:-}
SHUMA_ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE=${SHUMA_ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE:-}
SHUMA_EVENT_LOG_RETENTION_HOURS=${SHUMA_EVENT_LOG_RETENTION_HOURS:-}
SHUMA_ADMIN_CONFIG_WRITE_ENABLED=${SHUMA_ADMIN_CONFIG_WRITE_ENABLED:-}
SHUMA_KV_STORE_FAIL_OPEN=${SHUMA_KV_STORE_FAIL_OPEN:-}
SHUMA_ENFORCE_HTTPS=${SHUMA_ENFORCE_HTTPS:-}
SHUMA_DEBUG_HEADERS=${SHUMA_DEBUG_HEADERS:-}
SHUMA_ENTERPRISE_MULTI_INSTANCE=${SHUMA_ENTERPRISE_MULTI_INSTANCE:-}
SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED=${SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED:-}
SHUMA_RATE_LIMITER_REDIS_URL=${SHUMA_RATE_LIMITER_REDIS_URL:-}
SHUMA_BAN_STORE_REDIS_URL=${SHUMA_BAN_STORE_REDIS_URL:-}
SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN=${SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN:-}
SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH=${SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH:-}
EOF
    fi
    chmod 600 "$ENV_LOCAL_FILE" 2>/dev/null || true
}

ensure_local_dev_secret() {
    local key="$1"
    local bytes="$2"
    local current_value=""
    local should_generate=0

    current_value="$(read_env_local_value "$key")"
    case "$key" in
        SHUMA_API_KEY)
            case "$current_value" in
                ""|changeme-dev-only-api-key|changeme-supersecret|changeme-prod-api-key)
                    should_generate=1
                    ;;
            esac
            ;;
        SHUMA_JS_SECRET)
            case "$current_value" in
                ""|changeme-dev-only-js-secret|changeme-js-secret|changeme-prod-js-secret)
                    should_generate=1
                    ;;
            esac
            ;;
        SHUMA_FORWARDED_IP_SECRET)
            case "$current_value" in
                ""|changeme-dev-only-ip-secret|changeme-prod-forwarded-ip-secret)
                    should_generate=1
                    ;;
            esac
            ;;
    esac

    if [[ "$should_generate" -eq 1 ]]; then
        upsert_env_local_value "$key" "$(generate_hex_secret "$bytes")"
    fi
}

ensure_env_local_default_from_defaults() {
    local key="$1"
    local current_value=""
    local default_value=""

    current_value="$(read_env_local_value "$key")"
    default_value="${!key:-}"
    if [[ -z "$current_value" ]]; then
        upsert_env_local_value "$key" "$default_value"
    fi
}

detect_primary_ipv4() {
    local iface=""
    local ip=""
    if [[ "$(uname)" == "Darwin" ]] && command -v route &> /dev/null && command -v ipconfig &> /dev/null; then
        iface="$(route -n get default 2>/dev/null | awk '/interface:/{print $2; exit}')"
        if [[ -n "$iface" ]]; then
            ip="$(ipconfig getifaddr "$iface" 2>/dev/null || true)"
        fi
    fi
    printf '%s' "$ip"
}

default_admin_ip_allowlist_value() {
    local primary_ip=""
    primary_ip="$(detect_primary_ipv4)"
    if [[ -n "$primary_ip" ]]; then
        printf '127.0.0.1,::1,%s' "$primary_ip"
    else
        printf '127.0.0.1,::1'
    fi
}

ensure_admin_ip_allowlist_local_default() {
    local current_value=""
    current_value="$(read_env_local_value "SHUMA_ADMIN_IP_ALLOWLIST")"
    if [[ -n "$current_value" ]]; then
        return
    fi
    upsert_env_local_value "SHUMA_ADMIN_IP_ALLOWLIST" "$(default_admin_ip_allowlist_value)"
}

echo -e "${CYAN}"
echo "╔═══════════════════════════════════════════════════╗"
echo "║     WASM Bot Defence - Development Setup             ║"
echo "╚═══════════════════════════════════════════════════╝"
echo -e "${NC}"
info "If setup needs sudo (for example, to install Spin), run this in an interactive terminal so you can authorize prompts."

if [[ ! -f "$DEFAULTS_FILE" ]]; then
    error "Missing ${DEFAULTS_FILE}. Cannot initialize local defaults."
fi
# shellcheck disable=SC1090
set -a
source "$DEFAULTS_FILE"
set +a

#--------------------------
# Check macOS
#--------------------------
if [[ "$(uname)" != "Darwin" ]]; then
    warn "This script is designed for macOS. You may need to adapt for your OS."
    warn "Linux users: Replace Homebrew commands with your package manager."
fi

#--------------------------
# Homebrew
#--------------------------
if command -v brew &> /dev/null; then
    success "Homebrew already installed"
else
    info "Installing Homebrew..."
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    
    # Add to PATH for Apple Silicon Macs
    if [[ -f "/opt/homebrew/bin/brew" ]]; then
        eval "$(/opt/homebrew/bin/brew shellenv)"
        echo 'eval "$(/opt/homebrew/bin/brew shellenv)"' >> ~/.zprofile
    fi
    success "Homebrew installed"
fi

#--------------------------
# Node.js / corepack
#--------------------------
if command -v node &> /dev/null; then
    NODE_VERSION="$(node --version)"
    success "Node.js already installed (${NODE_VERSION})"
else
    if command -v brew &> /dev/null; then
        info "Installing Node.js via Homebrew..."
        brew install node
        success "Node.js installed ($(node --version 2>/dev/null || echo unknown))"
    else
        error "Node.js not found and Homebrew is unavailable. Install Node.js 18+ and re-run make setup."
    fi
fi

if command -v corepack &> /dev/null; then
    success "corepack already available"
else
    if command -v npm &> /dev/null; then
        info "Installing corepack via npm..."
        npm install -g corepack
        success "corepack installed"
    else
        error "npm is unavailable; cannot install corepack. Install Node.js 18+ and re-run make setup."
    fi
fi
corepack enable > /dev/null 2>&1 || true

#--------------------------
# Rust / Cargo
#--------------------------
if command -v rustc &> /dev/null; then
    RUST_VERSION=$(rustc --version | cut -d' ' -f2)
    success "Rust already installed (v$RUST_VERSION)"
else
    info "Installing Rust via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    success "Rust installed"
fi

# Ensure cargo is in PATH for this session
if [[ -f "$HOME/.cargo/env" ]]; then
    source "$HOME/.cargo/env"
fi

# Ensure future shells load Cargo (so make targets can find cargo)
if [[ -f "$HOME/.cargo/env" ]]; then
    CARGO_ENV_LINE='source "$HOME/.cargo/env"'
    PROFILE_FILES=(
        "$HOME/.zprofile"
        "$HOME/.zshrc"
        "$HOME/.bash_profile"
        "$HOME/.bashrc"
        "$HOME/.profile"
    )
    FOUND_PROFILE=0
    for PROFILE in "${PROFILE_FILES[@]}"; do
        if [[ -f "$PROFILE" ]]; then
            FOUND_PROFILE=1
            if ! grep -Fq "$CARGO_ENV_LINE" "$PROFILE"; then
                echo "$CARGO_ENV_LINE" >> "$PROFILE"
            fi
        fi
    done
    if [[ "$FOUND_PROFILE" -eq 0 ]]; then
        echo "$CARGO_ENV_LINE" >> "$HOME/.profile"
    fi
fi

#--------------------------
# WASM target
#--------------------------
if rustup target list --installed | grep -q "wasm32-wasip1"; then
    success "wasm32-wasip1 target already installed"
else
    info "Adding wasm32-wasip1 target..."
    rustup target add wasm32-wasip1
    success "wasm32-wasip1 target installed"
fi

#--------------------------
# Fermyon Spin
#--------------------------
if command -v spin &> /dev/null; then
    SPIN_VERSION=$(spin --version | head -1)
    success "Spin already installed ($SPIN_VERSION)"
else
    info "Installing Fermyon Spin..."
    SPIN_INSTALL_DIR="/usr/local/bin"
    TMP_SPIN_DIR="$(mktemp -d /tmp/shuma-gorath-spin.XXXXXX)"
    cleanup_spin_tmp() { rm -rf "$TMP_SPIN_DIR"; }
    trap cleanup_spin_tmp EXIT

    (
        cd "$TMP_SPIN_DIR"
        curl -fsSL https://developer.fermyon.com/downloads/install.sh | bash

        if [[ ! -f "spin" ]]; then
            error "Spin installer did not produce a 'spin' binary in $TMP_SPIN_DIR"
        fi

        if [[ -w "$SPIN_INSTALL_DIR" ]]; then
            mv "$TMP_SPIN_DIR/spin" "$SPIN_INSTALL_DIR/spin"
        else
            if ! command -v sudo &> /dev/null; then
                error "sudo not available; cannot move spin into $SPIN_INSTALL_DIR"
            fi

            if [[ ! -t 0 ]]; then
                error "This step needs sudo to move spin into $SPIN_INSTALL_DIR. Please run make setup in an interactive terminal where you can authorize sudo."
            fi

            if ! sudo /bin/mv "$TMP_SPIN_DIR/spin" "$SPIN_INSTALL_DIR/spin"; then
                error "Failed to move spin into $SPIN_INSTALL_DIR. Please re-run make setup in an interactive terminal and authorize sudo."
            fi
        fi
    )
    success "Spin installed"
fi

#--------------------------
# cargo-watch
#--------------------------
if command -v cargo-watch &> /dev/null; then
    success "cargo-watch already installed"
else
    info "Installing cargo-watch (for file watching)..."
    cargo install cargo-watch
    success "cargo-watch installed"
fi

#--------------------------
# Dashboard JS dependencies + Playwright browser runtime
#--------------------------
dashboard_deps_ready() {
    [[ -d "node_modules/.pnpm" ]] && \
    [[ -x "node_modules/.bin/vite" ]] && \
    [[ -d "node_modules/svelte" ]] && \
    [[ -d "node_modules/@sveltejs/kit" ]] && \
    [[ -d "node_modules/@playwright/test" ]]
}

if dashboard_deps_ready; then
    success "Dashboard dependencies already installed"
else
    info "Refreshing dashboard dependencies from lockfile..."
    corepack pnpm install --offline --frozen-lockfile || corepack pnpm install --frozen-lockfile
    if dashboard_deps_ready; then
        success "Dashboard dependencies installed from lockfile"
    else
        error "Dashboard dependencies are incomplete after pnpm install."
    fi
fi

PLAYWRIGHT_CHROMIUM_PATH="$(corepack pnpm exec node -e "const { chromium } = require('@playwright/test'); process.stdout.write(chromium.executablePath() || '');" 2>/dev/null || true)"
if [[ -n "$PLAYWRIGHT_CHROMIUM_PATH" && -x "$PLAYWRIGHT_CHROMIUM_PATH" ]]; then
    success "Playwright Chromium already installed ($PLAYWRIGHT_CHROMIUM_PATH)"
else
    info "Installing Playwright Chromium runtime..."
    corepack pnpm exec playwright install chromium
    PLAYWRIGHT_CHROMIUM_PATH="$(corepack pnpm exec node -e "const { chromium } = require('@playwright/test'); process.stdout.write(chromium.executablePath() || '');" 2>/dev/null || true)"
    if [[ -n "$PLAYWRIGHT_CHROMIUM_PATH" && -x "$PLAYWRIGHT_CHROMIUM_PATH" ]]; then
        success "Playwright Chromium installed ($PLAYWRIGHT_CHROMIUM_PATH)"
    else
        error "Playwright Chromium install did not produce an executable browser runtime."
    fi
fi

#--------------------------
# Local dev secrets
#--------------------------
ensure_env_local_file
ensure_local_dev_secret "SHUMA_API_KEY" 32
ensure_env_local_default_from_defaults "SHUMA_ADMIN_READONLY_API_KEY"
ensure_local_dev_secret "SHUMA_JS_SECRET" 32
ensure_local_dev_secret "SHUMA_FORWARDED_IP_SECRET" 32
ensure_env_local_default_from_defaults "SHUMA_POW_SECRET"
ensure_env_local_default_from_defaults "SHUMA_CHALLENGE_SECRET"
ensure_env_local_default_from_defaults "SHUMA_MAZE_PREVIEW_SECRET"
ensure_env_local_default_from_defaults "SHUMA_HEALTH_SECRET"
ensure_admin_ip_allowlist_local_default
ensure_env_local_default_from_defaults "SHUMA_ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE"
ensure_env_local_default_from_defaults "SHUMA_EVENT_LOG_RETENTION_HOURS"
ensure_env_local_default_from_defaults "SHUMA_ADMIN_CONFIG_WRITE_ENABLED"
ensure_env_local_default_from_defaults "SHUMA_KV_STORE_FAIL_OPEN"
ensure_env_local_default_from_defaults "SHUMA_ENFORCE_HTTPS"
ensure_env_local_default_from_defaults "SHUMA_DEBUG_HEADERS"
ensure_env_local_default_from_defaults "SHUMA_ENTERPRISE_MULTI_INSTANCE"
ensure_env_local_default_from_defaults "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED"
ensure_env_local_default_from_defaults "SHUMA_RATE_LIMITER_REDIS_URL"
ensure_env_local_default_from_defaults "SHUMA_BAN_STORE_REDIS_URL"
ensure_env_local_default_from_defaults "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN"
ensure_env_local_default_from_defaults "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH"
normalize_env_local_unquoted_style
success "Local dev secrets are ready in $ENV_LOCAL_FILE"

info "Seeding/backfilling KV tunables from config/defaults.env..."
make --no-print-directory config-seed
success "KV tunables are seeded"

#--------------------------
# Makefile sanity (dev target)
#--------------------------
if grep -q "cargo watch .* -x './scripts/set_crate_type.sh" Makefile 2>/dev/null; then
    warn "Makefile dev target uses cargo watch -x with a shell script; make dev will fail until updated."
fi

#--------------------------
# Verify installation
#--------------------------
echo ""
echo -e "${CYAN}═══════════════════════════════════════════════════${NC}"
echo -e "${GREEN}🎉 Setup complete! Installed versions:${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════${NC}"
echo ""

echo -n "  Rust:         "
rustc --version 2>/dev/null || echo "not found"

echo -n "  Cargo:        "
cargo --version 2>/dev/null || echo "not found"

echo -n "  WASM target:  "
if rustup target list --installed | grep -q "wasm32-wasip1"; then
    echo "wasm32-wasip1 ✓"
else
    echo "not installed"
fi

echo -n "  Spin:         "
spin --version 2>/dev/null | head -1 || echo "not found"

echo -n "  cargo-watch:  "
cargo-watch --version 2>/dev/null || echo "not found"

echo -n "  Node.js:      "
node --version 2>/dev/null || echo "not found"

echo -n "  corepack:     "
corepack --version 2>/dev/null || echo "not found"

echo -n "  pnpm:         "
corepack pnpm --version 2>/dev/null || echo "not found"

echo -n "  Chromium:     "
PLAYWRIGHT_CHROMIUM_PATH="$(corepack pnpm exec node -e "const { chromium } = require('@playwright/test'); process.stdout.write(chromium.executablePath() || '');" 2>/dev/null || true)"
if [[ -n "$PLAYWRIGHT_CHROMIUM_PATH" && -x "$PLAYWRIGHT_CHROMIUM_PATH" ]]; then
    echo "$PLAYWRIGHT_CHROMIUM_PATH"
else
    echo "not found"
fi

echo ""
echo -e "${CYAN}═══════════════════════════════════════════════════${NC}"
echo -e "${GREEN}🚀 Ready to go! Run these commands:${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════${NC}"
echo ""
echo "  If commands are missing, open a new terminal or run: source ~/.zshrc"
echo ""
echo "  make dev      # Start dev server with file watching"
echo "  make run      # Build once and run (no watching)"
echo "  make test     # Run tests"
echo "  make api-key-show # Show local dashboard login key (SHUMA_API_KEY)"
echo "  make help     # Show all commands"
echo ""
echo -e "${YELLOW}📊 Dashboard: http://127.0.0.1:3000/dashboard/index.html${NC}"
echo ""
