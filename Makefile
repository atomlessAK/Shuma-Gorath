.PHONY: dev local run run-prebuilt build prod clean test test-unit unit-test test-integration integration-test test-coverage test-dashboard test-dashboard-e2e deploy logs status stop help setup verify config-seed env-help api-key-generate gen-admin-api-key api-key-show api-key-rotate api-key-validate deploy-env-validate

# Default target
.DEFAULT_GOAL := help

# Colors for output
GREEN := \033[0;32m
YELLOW := \033[1;33m
CYAN := \033[0;36m
NC := \033[0m

WASM_BUILD_OUTPUT := target/wasm32-wasip1/release/shuma_gorath.wasm
WASM_ARTIFACT := dist/wasm/shuma_gorath.wasm

# Ensure rustup-installed cargo is available in non-interactive shells
CARGO_HOME ?= $(HOME)/.cargo
PATH := $(CARGO_HOME)/bin:$(PATH)
export PATH

# Load local development overrides (created by make setup)
ENV_LOCAL ?= .env.local
ifneq ("$(wildcard $(ENV_LOCAL))","")
include $(ENV_LOCAL)
endif

# Normalize optional quoted values from .env.local (handles KEY=value and KEY="value")
strip_wrapping_quotes = $(patsubst "%",%,$(patsubst '%',%,$(strip $(1))))
SHUMA_API_KEY := $(call strip_wrapping_quotes,$(SHUMA_API_KEY))
SHUMA_ADMIN_READONLY_API_KEY := $(call strip_wrapping_quotes,$(SHUMA_ADMIN_READONLY_API_KEY))
SHUMA_JS_SECRET := $(call strip_wrapping_quotes,$(SHUMA_JS_SECRET))
SHUMA_POW_SECRET := $(call strip_wrapping_quotes,$(SHUMA_POW_SECRET))
SHUMA_CHALLENGE_SECRET := $(call strip_wrapping_quotes,$(SHUMA_CHALLENGE_SECRET))
SHUMA_FORWARDED_IP_SECRET := $(call strip_wrapping_quotes,$(SHUMA_FORWARDED_IP_SECRET))
SHUMA_HEALTH_SECRET := $(call strip_wrapping_quotes,$(SHUMA_HEALTH_SECRET))
SHUMA_ADMIN_IP_ALLOWLIST := $(call strip_wrapping_quotes,$(SHUMA_ADMIN_IP_ALLOWLIST))
SHUMA_ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE := $(call strip_wrapping_quotes,$(SHUMA_ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE))
SHUMA_EVENT_LOG_RETENTION_HOURS := $(call strip_wrapping_quotes,$(SHUMA_EVENT_LOG_RETENTION_HOURS))
SHUMA_ADMIN_CONFIG_WRITE_ENABLED := $(call strip_wrapping_quotes,$(SHUMA_ADMIN_CONFIG_WRITE_ENABLED))
SHUMA_KV_STORE_FAIL_OPEN := $(call strip_wrapping_quotes,$(SHUMA_KV_STORE_FAIL_OPEN))
SHUMA_ENFORCE_HTTPS := $(call strip_wrapping_quotes,$(SHUMA_ENFORCE_HTTPS))
SHUMA_DEBUG_HEADERS := $(call strip_wrapping_quotes,$(SHUMA_DEBUG_HEADERS))
SHUMA_POW_CONFIG_MUTABLE := $(call strip_wrapping_quotes,$(SHUMA_POW_CONFIG_MUTABLE))
SHUMA_CHALLENGE_CONFIG_MUTABLE := $(call strip_wrapping_quotes,$(SHUMA_CHALLENGE_CONFIG_MUTABLE))
SHUMA_BOTNESS_CONFIG_MUTABLE := $(call strip_wrapping_quotes,$(SHUMA_BOTNESS_CONFIG_MUTABLE))
SHUMA_RATE_LIMITER_REDIS_URL := $(call strip_wrapping_quotes,$(SHUMA_RATE_LIMITER_REDIS_URL))

# Inject env-only runtime keys into Spin from .env.local / shell env.
SPIN_ENV_ONLY := --env SHUMA_API_KEY=$(SHUMA_API_KEY) --env SHUMA_ADMIN_READONLY_API_KEY=$(SHUMA_ADMIN_READONLY_API_KEY) --env SHUMA_JS_SECRET=$(SHUMA_JS_SECRET) --env SHUMA_POW_SECRET=$(SHUMA_POW_SECRET) --env SHUMA_CHALLENGE_SECRET=$(SHUMA_CHALLENGE_SECRET) --env SHUMA_FORWARDED_IP_SECRET=$(SHUMA_FORWARDED_IP_SECRET) --env SHUMA_HEALTH_SECRET=$(SHUMA_HEALTH_SECRET) --env SHUMA_ADMIN_IP_ALLOWLIST=$(SHUMA_ADMIN_IP_ALLOWLIST) --env SHUMA_ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE=$(SHUMA_ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE) --env SHUMA_EVENT_LOG_RETENTION_HOURS=$(SHUMA_EVENT_LOG_RETENTION_HOURS) --env SHUMA_ADMIN_CONFIG_WRITE_ENABLED=$(SHUMA_ADMIN_CONFIG_WRITE_ENABLED) --env SHUMA_KV_STORE_FAIL_OPEN=$(SHUMA_KV_STORE_FAIL_OPEN) --env SHUMA_ENFORCE_HTTPS=$(SHUMA_ENFORCE_HTTPS) --env SHUMA_DEBUG_HEADERS=$(SHUMA_DEBUG_HEADERS) --env SHUMA_RATE_LIMITER_REDIS_URL=$(SHUMA_RATE_LIMITER_REDIS_URL) --env SHUMA_POW_CONFIG_MUTABLE=$(SHUMA_POW_CONFIG_MUTABLE) --env SHUMA_CHALLENGE_CONFIG_MUTABLE=$(SHUMA_CHALLENGE_CONFIG_MUTABLE) --env SHUMA_BOTNESS_CONFIG_MUTABLE=$(SHUMA_BOTNESS_CONFIG_MUTABLE)

# Optional forwarded-IP trust header for local health/test requests.
FORWARDED_SECRET_HEADER := $(if $(SHUMA_FORWARDED_IP_SECRET),-H "X-Shuma-Forwarded-Secret: $(SHUMA_FORWARDED_IP_SECRET)",)
# Optional health secret header for local health/test requests.
HEALTH_SECRET_HEADER := $(if $(SHUMA_HEALTH_SECRET),-H "X-Shuma-Health-Secret: $(SHUMA_HEALTH_SECRET)",)
DEV_ADMIN_CONFIG_WRITE_ENABLED ?= true
SPIN_DEV_OVERRIDES := --env SHUMA_CHALLENGE_CONFIG_MUTABLE=true --env SHUMA_DEBUG_HEADERS=true --env SHUMA_ADMIN_CONFIG_WRITE_ENABLED=$(DEV_ADMIN_CONFIG_WRITE_ENABLED)
SPIN_PROD_OVERRIDES := --env SHUMA_DEBUG_HEADERS=false

#--------------------------
# Setup (first-time)
#--------------------------

setup: ## Install all dependencies (Rust, Spin, cargo-watch)
	@./scripts/bootstrap/setup.sh

verify: ## Verify all dependencies are installed correctly
	@./scripts/bootstrap/verify-setup.sh

config-seed: ## Seed KV tunable config from config/defaults.env (only when missing)
	@./scripts/config_seed.sh

#--------------------------
# Development
#--------------------------

dev: ## Build and run with file watching (auto-rebuild on save)
	@echo "$(CYAN)🚀 Starting development server with file watching...$(NC)"
	@echo "$(YELLOW)📊 Dashboard: http://127.0.0.1:3000/dashboard/index.html$(NC)"
	@echo "$(YELLOW)📈 Metrics:   http://127.0.0.1:3000/metrics$(NC)"
	@echo "$(YELLOW)❤️  Health:    http://127.0.0.1:3000/health$(NC)"
	@echo "$(CYAN)👀 Watching src/*.rs, dashboard/*, and spin.toml for changes... (Ctrl+C to stop)$(NC)"
	@pkill -x spin 2>/dev/null || true
	@./scripts/set_crate_type.sh cdylib
	@cargo build --target wasm32-wasip1 --release
	@mkdir -p $(dir $(WASM_ARTIFACT))
	@cp $(WASM_BUILD_OUTPUT) $(WASM_ARTIFACT)
	@./scripts/set_crate_type.sh rlib
	@cargo watch --poll -w src -w dashboard -w spin.toml -i '*.wasm' -i 'dist/wasm/shuma_gorath.wasm' -i '.spin/**' \
		-s 'if [ ! -f $(WASM_BUILD_OUTPUT) ] || find src -name "*.rs" -newer $(WASM_BUILD_OUTPUT) -print -quit | grep -q .; then ./scripts/set_crate_type.sh cdylib && cargo build --target wasm32-wasip1 --release && mkdir -p $(dir $(WASM_ARTIFACT)) && cp $(WASM_BUILD_OUTPUT) $(WASM_ARTIFACT) && ./scripts/set_crate_type.sh rlib; else echo "No Rust changes detected; skipping WASM rebuild."; fi' \
		-s 'pkill -x spin 2>/dev/null || true; SPIN_ALWAYS_BUILD=0 spin up --direct-mounts $(SPIN_ENV_ONLY) $(SPIN_DEV_OVERRIDES) --listen 127.0.0.1:3000'

dev-closed: ## Build and run with file watching and SHUMA_KV_STORE_FAIL_OPEN=false (fail-closed)
	@echo "$(CYAN)🚨 Starting development server with SHUMA_KV_STORE_FAIL_OPEN=false (fail-closed)...$(NC)"
	@echo "$(YELLOW)📊 Dashboard: http://127.0.0.1:3000/dashboard/index.html$(NC)"
	@echo "$(YELLOW)📈 Metrics:   http://127.0.0.1:3000/metrics$(NC)"
	@echo "$(YELLOW)❤️  Health:    http://127.0.0.1:3000/health$(NC)"
	@echo "$(CYAN)👀 Watching src/*.rs, dashboard/*, and spin.toml for changes... (Ctrl+C to stop)$(NC)"
	@pkill -x spin 2>/dev/null || true
	@./scripts/set_crate_type.sh cdylib
	@cargo build --target wasm32-wasip1 --release
	@mkdir -p $(dir $(WASM_ARTIFACT))
	@cp $(WASM_BUILD_OUTPUT) $(WASM_ARTIFACT)
	@./scripts/set_crate_type.sh rlib
	@cargo watch --poll -w src -w dashboard -w spin.toml -i '*.wasm' -i 'dist/wasm/shuma_gorath.wasm' -i '.spin/**' \
		-s 'if [ ! -f $(WASM_BUILD_OUTPUT) ] || find src -name "*.rs" -newer $(WASM_BUILD_OUTPUT) -print -quit | grep -q .; then ./scripts/set_crate_type.sh cdylib && cargo build --target wasm32-wasip1 --release && mkdir -p $(dir $(WASM_ARTIFACT)) && cp $(WASM_BUILD_OUTPUT) $(WASM_ARTIFACT) && ./scripts/set_crate_type.sh rlib; else echo "No Rust changes detected; skipping WASM rebuild."; fi' \
		-s 'pkill -x spin 2>/dev/null || true; SPIN_ALWAYS_BUILD=0 spin up --direct-mounts $(SPIN_ENV_ONLY) $(SPIN_DEV_OVERRIDES) --env SHUMA_KV_STORE_FAIL_OPEN=false --listen 127.0.0.1:3000'

local: dev ## Alias for dev

run: ## Build once and run (no file watching)
	@echo "$(CYAN)🚀 Starting development server...$(NC)"
	@pkill -x spin 2>/dev/null || true
	@sleep 1
	@./scripts/set_crate_type.sh cdylib
	@cargo build --target wasm32-wasip1 --release
	@mkdir -p $(dir $(WASM_ARTIFACT))
	@cp $(WASM_BUILD_OUTPUT) $(WASM_ARTIFACT)
	@./scripts/set_crate_type.sh rlib
	@echo "$(GREEN)✅ Build complete. Starting Spin...$(NC)"
	@echo "$(YELLOW)📊 Dashboard: http://127.0.0.1:3000/dashboard/index.html$(NC)"
	@echo "$(YELLOW)📈 Metrics:   http://127.0.0.1:3000/metrics$(NC)"
	@echo "$(YELLOW)❤️  Health:    http://127.0.0.1:3000/health$(NC)"
	@spin up $(SPIN_ENV_ONLY) $(SPIN_DEV_OVERRIDES) --listen 127.0.0.1:3000

run-prebuilt: ## Run Spin using prebuilt wasm (CI helper)
	@echo "$(CYAN)🚀 Starting prebuilt server...$(NC)"
	@pkill -x spin 2>/dev/null || true
	@echo "$(YELLOW)📊 Dashboard: http://127.0.0.1:3000/dashboard/index.html$(NC)"
	@echo "$(YELLOW)📈 Metrics:   http://127.0.0.1:3000/metrics$(NC)"
	@echo "$(YELLOW)❤️  Health:    http://127.0.0.1:3000/health$(NC)"
	@spin up $(SPIN_ENV_ONLY) $(SPIN_DEV_OVERRIDES) --listen 127.0.0.1:3000

#--------------------------
# Production
#--------------------------

build: ## Build release binary only (no server start)
	@echo "$(CYAN)🔨 Building release binary...$(NC)"
	@./scripts/set_crate_type.sh cdylib
	@cargo build --target wasm32-wasip1 --release
	@mkdir -p $(dir $(WASM_ARTIFACT))
	@cp $(WASM_BUILD_OUTPUT) $(WASM_ARTIFACT)
	@echo "$(GREEN)✅ Build complete: $(WASM_ARTIFACT)$(NC)"
	@./scripts/set_crate_type.sh rlib

prod: build ## Build for production and start server
	@echo "$(CYAN)🚀 Starting production server...$(NC)"
	@pkill -x spin 2>/dev/null || true
	@spin up $(SPIN_ENV_ONLY) $(SPIN_PROD_OVERRIDES) --listen 0.0.0.0:3000

deploy: build ## Deploy to Fermyon Cloud
	@$(MAKE) --no-print-directory api-key-validate
	@$(MAKE) --no-print-directory deploy-env-validate
	@echo "$(CYAN)☁️  Deploying to Fermyon Cloud...$(NC)"
	@spin cloud deploy
	@echo "$(GREEN)✅ Deployment complete!$(NC)"

#--------------------------
# Testing
#--------------------------

test: ## Run ALL tests in series: unit, integration, and dashboard e2e (fails if server is not running)
	@echo "$(CYAN)============================================$(NC)"
	@echo "$(CYAN)  RUNNING ALL TESTS$(NC)"
	@echo "$(CYAN)============================================$(NC)"
	@echo ""
	@if ! curl -sf -H "X-Forwarded-For: 127.0.0.1" $(FORWARDED_SECRET_HEADER) $(HEALTH_SECRET_HEADER) http://127.0.0.1:3000/health > /dev/null 2>&1; then \
		echo "$(RED)❌ Error: Spin server not running. Integration tests must run and may not be skipped.$(NC)"; \
		echo "$(YELLOW)   Required flow: 1) make dev  2) make test$(NC)"; \
		exit 1; \
	fi
	@echo "$(GREEN)✅ Preflight: Spin server is running; integration and dashboard e2e tests will be executed.$(NC)"
	@echo ""
	@echo "$(CYAN)Step 1/3: Rust Unit Tests$(NC)"
	@echo "$(CYAN)--------------------------------------------$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test || exit 1
	@echo ""
	@echo "$(CYAN)Step 2/3: Integration Tests (Spin HTTP scenarios)$(NC)"
	@echo "$(CYAN)--------------------------------------------$(NC)"
	@if curl -sf -H "X-Forwarded-For: 127.0.0.1" $(FORWARDED_SECRET_HEADER) $(HEALTH_SECRET_HEADER) http://127.0.0.1:3000/health > /dev/null 2>&1; then \
		SHUMA_API_KEY="$(SHUMA_API_KEY)" SHUMA_FORWARDED_IP_SECRET="$(SHUMA_FORWARDED_IP_SECRET)" SHUMA_HEALTH_SECRET="$(SHUMA_HEALTH_SECRET)" ./scripts/tests/integration.sh || exit 1; \
	else \
		echo "$(RED)❌ Error: Spin server not running. Integration tests must run and may not be skipped.$(NC)"; \
		echo "$(YELLOW)   Start server first: make dev$(NC)"; \
		echo "$(YELLOW)   Then run tests:     make test$(NC)"; \
		exit 1; \
	fi
	@echo ""
	@echo "$(CYAN)Step 3/3: Dashboard E2E Smoke Tests$(NC)"
	@echo "$(CYAN)--------------------------------------------$(NC)"
	@$(MAKE) --no-print-directory test-dashboard-e2e || exit 1
	@echo ""
	@echo "$(GREEN)============================================$(NC)"
	@echo "$(GREEN)  ALL TESTS COMPLETE$(NC)"
	@echo "$(GREEN)============================================$(NC)"

test-unit: ## Run Rust unit tests only (34 tests)
	@echo "$(CYAN)🧪 Running Rust unit tests...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test

unit-test: test-unit ## Alias for Rust unit tests

test-integration: ## Run integration tests only (21 scenarios, requires running server)
	@echo "$(CYAN)🧪 Running integration tests...$(NC)"
	@if curl -sf -H "X-Forwarded-For: 127.0.0.1" $(FORWARDED_SECRET_HEADER) $(HEALTH_SECRET_HEADER) http://127.0.0.1:3000/health > /dev/null 2>&1; then \
		SHUMA_API_KEY="$(SHUMA_API_KEY)" SHUMA_FORWARDED_IP_SECRET="$(SHUMA_FORWARDED_IP_SECRET)" SHUMA_HEALTH_SECRET="$(SHUMA_HEALTH_SECRET)" ./scripts/tests/integration.sh; \
	else \
		echo "$(RED)❌ Error: Spin server not running$(NC)"; \
		echo "$(YELLOW)   Start the server first: make dev$(NC)"; \
		exit 1; \
	fi

integration-test: test-integration ## Alias for Spin integration tests

test-coverage: ## Run unit test coverage (requires cargo-llvm-cov)
	@echo "$(CYAN)🧪 Running Rust unit test coverage...$(NC)"
	@if ! command -v cargo-llvm-cov >/dev/null 2>&1; then \
		echo "$(RED)❌ Error: cargo-llvm-cov not found$(NC)"; \
		echo "$(YELLOW)   Install with: cargo install cargo-llvm-cov --locked$(NC)"; \
		exit 1; \
	fi
	@./scripts/set_crate_type.sh rlib
	@cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info
	@echo "$(GREEN)✅ Coverage report written to lcov.info$(NC)"

test-dashboard: ## Dashboard testing instructions (manual)
	@echo "$(CYAN)🧪 Dashboard testing (manual):$(NC)"
	@echo "1. Ensure Spin is running: make dev"
	@echo "2. Open: http://127.0.0.1:3000/dashboard/index.html"
	@echo "3. Follow checklist in docs/testing.md"

test-dashboard-e2e: ## Run Playwright dashboard smoke tests (requires running server)
	@echo "$(CYAN)🧪 Running dashboard e2e smoke tests...$(NC)"
	@if curl -sf -H "X-Forwarded-For: 127.0.0.1" $(FORWARDED_SECRET_HEADER) $(HEALTH_SECRET_HEADER) http://127.0.0.1:3000/health > /dev/null 2>&1; then \
		if ! command -v corepack >/dev/null 2>&1; then \
			echo "$(RED)❌ Error: corepack not found (install Node.js 18+).$(NC)"; \
			exit 1; \
		fi; \
		corepack enable > /dev/null 2>&1 || true; \
		corepack pnpm install --frozen-lockfile; \
		corepack pnpm exec playwright install chromium; \
		SHUMA_BASE_URL=http://127.0.0.1:3000 SHUMA_API_KEY=$(SHUMA_API_KEY) SHUMA_FORWARDED_IP_SECRET=$(SHUMA_FORWARDED_IP_SECRET) corepack pnpm run test:dashboard:e2e; \
	else \
		echo "$(RED)❌ Error: Spin server not running$(NC)"; \
		echo "$(YELLOW)   Start the server first: make dev$(NC)"; \
		exit 1; \
	fi

#--------------------------
# Utilities
#--------------------------

stop: ## Stop running Spin server
	@echo "$(CYAN)🛑 Stopping Spin server...$(NC)"
	@pkill -x spin 2>/dev/null && echo "$(GREEN)✅ Stopped$(NC)" || echo "$(YELLOW)No server running$(NC)"

status: ## Check if Spin server is running
	@if curl -sf -H "X-Forwarded-For: 127.0.0.1" $(FORWARDED_SECRET_HEADER) $(HEALTH_SECRET_HEADER) http://127.0.0.1:3000/health > /dev/null 2>&1; then \
		echo "$(GREEN)✅ Spin server is running$(NC)"; \
		echo "   Dashboard: http://127.0.0.1:3000/dashboard/index.html"; \
	else \
		echo "$(YELLOW)⚠️  Spin server is not running$(NC)"; \
	fi

clean: ## Clean build artifacts
	@echo "$(CYAN)🧹 Cleaning build artifacts...$(NC)"
	@cargo clean
	@rm -rf dist/wasm
	@rm -rf .spin
	@rm -rf playwright-report test-results
	@rm -f src/*.wasm
	@echo "$(GREEN)✅ Clean complete$(NC)"

logs: ## View Spin component logs
	@echo "$(CYAN)📜 Spin logs:$(NC)"
	@cat .spin/logs/* 2>/dev/null || echo "No logs found. Run 'make dev' first."

api-key-generate: ## Generate a high-entropy SHUMA_API_KEY using system CSPRNG tools
	@echo "$(CYAN)🔐 Generating SHUMA_API_KEY...$(NC)"
	@KEY="$$(if command -v openssl >/dev/null 2>&1; then openssl rand -hex 32; else od -An -N32 -tx1 /dev/urandom | tr -d ' \n'; fi)"; \
	echo ""; \
	echo "$$KEY"; \
	echo ""; \
	echo "$(YELLOW)Set in your secret store as: SHUMA_API_KEY=$$KEY$(NC)"

gen-admin-api-key: api-key-generate ## Alias for generating a strong SHUMA_API_KEY

api-key-show: ## Show SHUMA_API_KEY from .env.local (dashboard login key for local dev)
	@KEY="$$(grep -E '^SHUMA_API_KEY=' .env.local 2>/dev/null | tail -1 | cut -d= -f2- | sed -e 's/^"//' -e 's/"$$//')"; \
	if [ -z "$$KEY" ]; then \
		echo "$(RED)❌ No SHUMA_API_KEY found in .env.local.$(NC)"; \
		echo "$(YELLOW)Run: make setup$(NC)"; \
		exit 1; \
	fi; \
	echo "$(CYAN)Local dashboard login key (SHUMA_API_KEY):$(NC)"; \
	echo "$$KEY"

env-help: ## Show supported env-only runtime overrides
	@echo "$(CYAN)Supported env-only overrides (tunables are KV-backed):$(NC)"
	@echo "  SHUMA_API_KEY"
	@echo "  SHUMA_ADMIN_READONLY_API_KEY"
	@echo "  SHUMA_JS_SECRET"
	@echo "  SHUMA_POW_SECRET"
	@echo "  SHUMA_CHALLENGE_SECRET"
	@echo "  SHUMA_FORWARDED_IP_SECRET"
	@echo "  SHUMA_HEALTH_SECRET"
	@echo "  SHUMA_ADMIN_IP_ALLOWLIST"
	@echo "  SHUMA_ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE"
	@echo "  SHUMA_EVENT_LOG_RETENTION_HOURS"
	@echo "  SHUMA_ADMIN_CONFIG_WRITE_ENABLED"
	@echo "  SHUMA_KV_STORE_FAIL_OPEN"
	@echo "  SHUMA_ENFORCE_HTTPS"
	@echo "  SHUMA_DEBUG_HEADERS"
	@echo "  SHUMA_ENTERPRISE_MULTI_INSTANCE"
	@echo "  SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED"
	@echo "  SHUMA_RATE_LIMITER_REDIS_URL"
	@echo "  SHUMA_POW_CONFIG_MUTABLE"
	@echo "  SHUMA_CHALLENGE_CONFIG_MUTABLE"
	@echo "  SHUMA_BOTNESS_CONFIG_MUTABLE"

api-key-rotate: ## Generate a replacement SHUMA_API_KEY and print rotation guidance
	@$(MAKE) --no-print-directory api-key-generate
	@echo "$(YELLOW)Next steps: update deployment secret, redeploy/restart, then update dashboard login key.$(NC)"

api-key-validate: ## Validate SHUMA_API_KEY for deployment (must be 64-char hex and non-placeholder)
	@KEY="$(SHUMA_API_KEY)"; \
	if [ -z "$$KEY" ]; then \
		echo "$(RED)❌ SHUMA_API_KEY is empty.$(NC)"; \
		echo "$(YELLOW)Set SHUMA_API_KEY before deployment (or export it from your secret manager).$(NC)"; \
		exit 1; \
	fi; \
	case "$$KEY" in \
		changeme-dev-only-api-key|changeme-supersecret|changeme-prod-api-key) \
			echo "$(RED)❌ SHUMA_API_KEY is a placeholder value. Generate a real key first.$(NC)"; \
			exit 1 ;; \
	esac; \
	if ! printf '%s' "$$KEY" | grep -Eq '^[0-9A-Fa-f]{64}$$'; then \
		echo "$(RED)❌ SHUMA_API_KEY must be exactly 64 hexadecimal characters.$(NC)"; \
		echo "$(YELLOW)Generate one with: make api-key-generate$(NC)"; \
		exit 1; \
	fi; \
	echo "$(GREEN)✅ SHUMA_API_KEY format is valid for deployment.$(NC)"

deploy-env-validate: ## Fail deployment when unsafe debug flags are enabled, admin allowlist is missing, admin edge limits are unconfirmed, API-key rotation is unconfirmed, or enterprise multi-instance state guardrails are unmet
	@DEBUG_VAL="$${SHUMA_DEBUG_HEADERS:-false}"; \
	DEBUG_NORM="$$(printf '%s' "$$DEBUG_VAL" | tr '[:upper:]' '[:lower:]')"; \
	case "$$DEBUG_NORM" in \
		1|true|yes|on) \
			echo "$(RED)❌ Refusing deployment: SHUMA_DEBUG_HEADERS=true exposes internal headers.$(NC)"; \
			echo "$(YELLOW)Set SHUMA_DEBUG_HEADERS=false for production deploys.$(NC)"; \
			exit 1 ;; \
	esac; \
	ALLOWLIST_RAW="$${SHUMA_ADMIN_IP_ALLOWLIST:-}"; \
	ALLOWLIST_NORM="$$(printf '%s' "$$ALLOWLIST_RAW" | tr -d '[:space:]')"; \
	if [ -z "$$ALLOWLIST_NORM" ]; then \
		echo "$(RED)❌ Refusing deployment: SHUMA_ADMIN_IP_ALLOWLIST is required for production admin hardening.$(NC)"; \
		echo "$(YELLOW)Set SHUMA_ADMIN_IP_ALLOWLIST to one or more trusted IP/CIDR entries (comma-separated).$(NC)"; \
		exit 1; \
	fi; \
	case "$$ALLOWLIST_NORM" in \
		*0.0.0.0/0*|*::/0*|*\**) \
			echo "$(RED)❌ Refusing deployment: SHUMA_ADMIN_IP_ALLOWLIST is overbroad (contains wildcard/global range).$(NC)"; \
			echo "$(YELLOW)Use explicit trusted operator/VPN IPs or CIDRs only.$(NC)"; \
			exit 1 ;; \
	esac; \
	EDGE_LIMITS_CONFIRMED_RAW="$${SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED:-false}"; \
	EDGE_LIMITS_CONFIRMED_NORM="$$(printf '%s' "$$EDGE_LIMITS_CONFIRMED_RAW" | tr '[:upper:]' '[:lower:]')"; \
	case "$$EDGE_LIMITS_CONFIRMED_NORM" in \
		1|true|yes|on) ;; \
		*) \
			echo "$(RED)❌ Refusing deployment: SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED is not true.$(NC)"; \
			echo "$(YELLOW)Before deploy, configure CDN/WAF limits for POST /admin/login and /admin/*, then set SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED=true.$(NC)"; \
			exit 1 ;; \
	esac; \
	API_KEY_ROTATION_CONFIRMED_RAW="$${SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED:-false}"; \
	API_KEY_ROTATION_CONFIRMED_NORM="$$(printf '%s' "$$API_KEY_ROTATION_CONFIRMED_RAW" | tr '[:upper:]' '[:lower:]')"; \
	case "$$API_KEY_ROTATION_CONFIRMED_NORM" in \
		1|true|yes|on) ;; \
		*) \
			echo "$(RED)❌ Refusing deployment: SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED is not true.$(NC)"; \
			echo "$(YELLOW)Rotate SHUMA_API_KEY on your cadence (recommended 90 days) with make gen-admin-api-key / make api-key-rotate, then set SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED=true.$(NC)"; \
			exit 1 ;; \
	esac; \
	ENTERPRISE_MULTI_INSTANCE_RAW="$${SHUMA_ENTERPRISE_MULTI_INSTANCE:-false}"; \
	ENTERPRISE_MULTI_INSTANCE_NORM="$$(printf '%s' "$$ENTERPRISE_MULTI_INSTANCE_RAW" | tr '[:upper:]' '[:lower:]')"; \
	case "$$ENTERPRISE_MULTI_INSTANCE_NORM" in \
		1|true|yes|on) \
			EDGE_MODE_RAW="$${SHUMA_EDGE_INTEGRATION_MODE:-off}"; \
			EDGE_MODE_NORM="$$(printf '%s' "$$EDGE_MODE_RAW" | tr '[:upper:]' '[:lower:]')"; \
			case "$$EDGE_MODE_NORM" in \
				off|advisory|authoritative) ;; \
				*) \
					echo "$(RED)❌ Refusing deployment: SHUMA_EDGE_INTEGRATION_MODE must be one of off|advisory|authoritative when SHUMA_ENTERPRISE_MULTI_INSTANCE=true.$(NC)"; \
					exit 1 ;; \
			esac; \
			RATE_BACKEND_RAW="$${SHUMA_PROVIDER_RATE_LIMITER:-internal}"; \
			RATE_BACKEND_NORM="$$(printf '%s' "$$RATE_BACKEND_RAW" | tr '[:upper:]' '[:lower:]')"; \
			case "$$RATE_BACKEND_NORM" in \
				internal|external) ;; \
				*) \
					echo "$(RED)❌ Refusing deployment: SHUMA_PROVIDER_RATE_LIMITER must be internal|external when SHUMA_ENTERPRISE_MULTI_INSTANCE=true.$(NC)"; \
					exit 1 ;; \
			esac; \
			if [ "$$RATE_BACKEND_NORM" = "external" ]; then \
				RATE_REDIS_URL_RAW="$${SHUMA_RATE_LIMITER_REDIS_URL:-}"; \
				RATE_REDIS_URL_NORM="$$(printf '%s' "$$RATE_REDIS_URL_RAW" | tr -d '[:space:]')"; \
				if [ -z "$$RATE_REDIS_URL_NORM" ]; then \
					echo "$(RED)❌ Refusing deployment: SHUMA_RATE_LIMITER_REDIS_URL is required when SHUMA_ENTERPRISE_MULTI_INSTANCE=true and SHUMA_PROVIDER_RATE_LIMITER=external.$(NC)"; \
					exit 1; \
				fi; \
				case "$$RATE_REDIS_URL_NORM" in \
					redis://*|rediss://*) ;; \
					*) \
						echo "$(RED)❌ Refusing deployment: SHUMA_RATE_LIMITER_REDIS_URL must start with redis:// or rediss://.$(NC)"; \
						exit 1 ;; \
				esac; \
			fi; \
			BAN_BACKEND_RAW="$${SHUMA_PROVIDER_BAN_STORE:-internal}"; \
			BAN_BACKEND_NORM="$$(printf '%s' "$$BAN_BACKEND_RAW" | tr '[:upper:]' '[:lower:]')"; \
			case "$$BAN_BACKEND_NORM" in \
				internal|external) ;; \
				*) \
					echo "$(RED)❌ Refusing deployment: SHUMA_PROVIDER_BAN_STORE must be internal|external when SHUMA_ENTERPRISE_MULTI_INSTANCE=true.$(NC)"; \
					exit 1 ;; \
			esac; \
			UNSYNCED_LOCAL_STATE=0; \
			if [ "$$RATE_BACKEND_NORM" != "external" ] || [ "$$BAN_BACKEND_NORM" != "external" ]; then \
				UNSYNCED_LOCAL_STATE=1; \
			fi; \
			if [ "$$UNSYNCED_LOCAL_STATE" -eq 1 ]; then \
				if [ "$$EDGE_MODE_NORM" = "authoritative" ]; then \
					echo "$(RED)❌ Refusing deployment: enterprise multi-instance rollout cannot run with local-only rate/ban state in authoritative edge mode.$(NC)"; \
					echo "$(YELLOW)Use distributed state backends first, or move to advisory mode for a temporary exception window.$(NC)"; \
					exit 1; \
				fi; \
				UNSYNCED_EXCEPTION_RAW="$${SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED:-false}"; \
				UNSYNCED_EXCEPTION_NORM="$$(printf '%s' "$$UNSYNCED_EXCEPTION_RAW" | tr '[:upper:]' '[:lower:]')"; \
				case "$$UNSYNCED_EXCEPTION_NORM" in \
					1|true|yes|on) ;; \
					*) \
						echo "$(RED)❌ Refusing deployment: enterprise multi-instance rollout is using local-only rate/ban state without explicit exception attestation.$(NC)"; \
						echo "$(YELLOW)Set distributed state backends (SHUMA_PROVIDER_RATE_LIMITER=external and SHUMA_PROVIDER_BAN_STORE=external), or set SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED=true for temporary advisory-only operation.$(NC)"; \
						exit 1 ;; \
				esac; \
			fi ;; \
		0|false|no|off|"") ;; \
		*) \
			echo "$(RED)❌ Refusing deployment: SHUMA_ENTERPRISE_MULTI_INSTANCE must be a boolean value (true/false).$(NC)"; \
			exit 1 ;; \
	esac; \
	echo "$(GREEN)✅ Deployment env guardrails passed (SHUMA_DEBUG_HEADERS, SHUMA_ADMIN_IP_ALLOWLIST, SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED, SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED, enterprise multi-instance state guardrails).$(NC)"

#--------------------------
# Help
#--------------------------

help: ## Show this help message
	@echo "$(CYAN)WASM Bot Defence - Available Commands$(NC)"
	@echo ""
	@echo "$(GREEN)First-time Setup:$(NC)"
	@grep -E '^(setup|verify|config-seed):.*?## ' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  make %-15s %s\n", $$1, $$2}'
	@echo ""
	@echo "$(GREEN)Development:$(NC)"
	@grep -E '^(dev|local|run|build):.*?## ' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  make %-15s %s\n", $$1, $$2}'
	@echo ""
	@echo "$(GREEN)Production:$(NC)"
	@grep -E '^(prod|deploy):.*?## ' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  make %-15s %s\n", $$1, $$2}'
	@echo ""
	@echo "$(GREEN)Testing:$(NC)"
	@grep -E '^test.*:.*?## ' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  make %-15s %s\n", $$1, $$2}'
	@echo ""
	@echo "$(GREEN)Utilities:$(NC)"
	@grep -E '^(stop|status|clean|logs|env-help|api-key-generate|gen-admin-api-key|api-key-show|api-key-rotate|api-key-validate|deploy-env-validate|help):.*?## ' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  make %-15s %s\n", $$1, $$2}'
