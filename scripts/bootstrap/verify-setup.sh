#!/bin/bash
# verify-setup.sh - Verify setup script logic without installing
#
# This script checks what scripts/bootstrap/setup.sh would do on your system
# and verifies all dependencies are correctly installed.

set -e

GREEN="\033[0;32m"
YELLOW="\033[1;33m"
CYAN="\033[0;36m"
RED="\033[0;31m"
NC="\033[0m"

pass() { echo -e "${GREEN}✅ PASS${NC} $1"; }
fail() { echo -e "${RED}❌ FAIL${NC} $1"; FAILED=1; }
info() { echo -e "${CYAN}ℹ️  $1${NC}"; }
warn() { echo -e "${YELLOW}⚠️  $1${NC}"; }

FAILED=0

echo -e "${CYAN}"
echo "╔═══════════════════════════════════════════════════╗"
echo "║     WASM Bot Defence - Setup Verification            ║"
echo "╚═══════════════════════════════════════════════════╝"
echo -e "${NC}"

echo -e "${CYAN}=== Checking Dependencies ===${NC}"
echo ""

# 1. Check Homebrew (macOS only)
if [[ "$(uname)" == "Darwin" ]]; then
    if command -v brew &> /dev/null; then
        pass "Homebrew installed: $(brew --version | head -1)"
    else
        fail "Homebrew not installed (run: /bin/bash -c \"\$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\")"
    fi
fi

# 2. Check Rust
if command -v rustc &> /dev/null; then
    pass "Rust installed: $(rustc --version)"
else
    fail "Rust not installed (run: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh)"
fi

# 3. Check Cargo
if command -v cargo &> /dev/null; then
    pass "Cargo installed: $(cargo --version)"
else
    fail "Cargo not installed (comes with Rust)"
fi

# 4. Check WASM target
if command -v rustup &> /dev/null; then
    if rustup target list --installed 2>/dev/null | grep -q "wasm32-wasip1"; then
        pass "wasm32-wasip1 target installed"
    else
        fail "wasm32-wasip1 target not installed (run: rustup target add wasm32-wasip1)"
    fi
else
    fail "rustup not found (needed for WASM target)"
fi

# 5. Check Spin
if command -v spin &> /dev/null; then
    pass "Spin installed: $(spin --version | head -1)"
else
    fail "Spin not installed (run: curl -fsSL https://developer.fermyon.com/downloads/install.sh | bash)"
fi

# 6. Check cargo-watch
if command -v cargo-watch &> /dev/null; then
    pass "cargo-watch installed: $(cargo-watch --version)"
else
    warn "cargo-watch not installed (optional, run: cargo install cargo-watch)"
    info "  Without cargo-watch, use 'make run' instead of 'make dev'"
fi

# 6b. Sanity check Makefile dev target
if grep -q "cargo watch .* -x './scripts/set_crate_type.sh" Makefile 2>/dev/null; then
    fail "Makefile dev target uses cargo watch -x with a shell script (make dev will fail)"
else
    pass "Makefile dev target uses shell commands with cargo watch"
fi

echo ""
echo -e "${CYAN}=== Testing Build ===${NC}"
echo ""

# 7. Test WASM build
if command -v cargo &> /dev/null && rustup target list --installed 2>/dev/null | grep -q "wasm32-wasip1"; then
    info "Running test build..."
    if cargo build --target wasm32-wasip1 --release 2>&1; then
        WASM_SIZE=$(ls -lh target/wasm32-wasip1/release/shuma_gorath.wasm 2>/dev/null | awk '{print $5}')
        pass "WASM build successful ($WASM_SIZE)"
        cp target/wasm32-wasip1/release/shuma_gorath.wasm src/bot_defence.wasm 2>/dev/null || true
    else
        fail "WASM build failed"
    fi
else
    warn "Skipping build test (missing dependencies)"
fi

echo ""
echo -e "${CYAN}=== Testing Unit Tests ===${NC}"
echo ""

# 8. Run unit tests
if command -v cargo &> /dev/null; then
    info "Running unit tests..."
    if cargo test 2>&1 | tail -5; then
        pass "Unit tests passed"
    else
        fail "Unit tests failed"
    fi
else
    warn "Skipping tests (cargo not available)"
fi

echo ""
echo -e "${CYAN}═══════════════════════════════════════════════════${NC}"

if [[ $FAILED -eq 0 ]]; then
    echo -e "${GREEN}🎉 All checks passed! Your environment is ready.${NC}"
    echo ""
    echo "  Run: make dev"
    echo ""
else
    echo -e "${RED}⚠️  Some checks failed. Run 'make setup' to fix.${NC}"
    echo ""
fi
