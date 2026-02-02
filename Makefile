.PHONY: dev local run build prod clean test test-unit test-integration test-dashboard deploy logs status stop help

# Default target
.DEFAULT_GOAL := help

# Colors for output
GREEN := \033[0;32m
YELLOW := \033[1;33m
CYAN := \033[0;36m
NC := \033[0m

#--------------------------
# Development
#--------------------------

dev: ## Build and run with file watching (auto-rebuild on save)
	@echo "$(CYAN)🚀 Starting development server with file watching...$(NC)"
	@echo "$(YELLOW)📊 Dashboard: http://127.0.0.1:3000/dashboard/index.html$(NC)"
	@echo "$(YELLOW)📈 Metrics:   http://127.0.0.1:3000/metrics$(NC)"
	@echo "$(YELLOW)❤️  Health:    http://127.0.0.1:3000/health$(NC)"
	@echo "$(CYAN)👀 Watching for changes... (Ctrl+C to stop)$(NC)"
	@pkill -f "spin up" 2>/dev/null || true
	@cargo watch -x 'build --target wasm32-wasip1 --release' -s 'pkill -f "spin up" 2>/dev/null; cp target/wasm32-wasip1/release/wasm_bot_trap.wasm src/bot_trap.wasm && spin up --listen 127.0.0.1:3000'

local: dev ## Alias for dev

run: ## Build once and run (no file watching)
	@echo "$(CYAN)🚀 Starting development server...$(NC)"
	@pkill -f "spin up" 2>/dev/null || true
	@sleep 1
	@cargo build --target wasm32-wasip1 --release
	@cp target/wasm32-wasip1/release/wasm_bot_trap.wasm src/bot_trap.wasm 2>/dev/null || true
	@echo "$(GREEN)✅ Build complete. Starting Spin...$(NC)"
	@echo "$(YELLOW)📊 Dashboard: http://127.0.0.1:3000/dashboard/index.html$(NC)"
	@echo "$(YELLOW)📈 Metrics:   http://127.0.0.1:3000/metrics$(NC)"
	@echo "$(YELLOW)❤️  Health:    http://127.0.0.1:3000/health$(NC)"
	@spin up --listen 127.0.0.1:3000

#--------------------------
# Production
#--------------------------

build: ## Build release binary only (no server start)
	@echo "$(CYAN)🔨 Building release binary...$(NC)"
	@cargo build --target wasm32-wasip1 --release
	@cp target/wasm32-wasip1/release/wasm_bot_trap.wasm src/bot_trap.wasm
	@echo "$(GREEN)✅ Build complete: src/bot_trap.wasm$(NC)"

prod: build ## Build for production and start server
	@echo "$(CYAN)🚀 Starting production server...$(NC)"
	@pkill -f "spin up" 2>/dev/null || true
	@spin up --listen 0.0.0.0:3000

deploy: build ## Deploy to Fermyon Cloud
	@echo "$(CYAN)☁️  Deploying to Fermyon Cloud...$(NC)"
	@spin cloud deploy
	@echo "$(GREEN)✅ Deployment complete!$(NC)"

#--------------------------
# Testing
#--------------------------

test: test-unit ## Run all tests (unit + integration if server running)
	@if curl -s http://127.0.0.1:3000/health > /dev/null 2>&1; then \
		$(MAKE) test-integration; \
	else \
		echo "$(YELLOW)⚠️  Spin not running. Skipping integration tests.$(NC)"; \
		echo "$(YELLOW)   Run 'make dev' first, then 'make test-integration'$(NC)"; \
	fi
	@echo "$(GREEN)✅ Tests complete!$(NC)"

test-unit: ## Run Rust unit tests
	@echo "$(CYAN)🧪 Running unit tests...$(NC)"
	@cargo test

test-integration: ## Run Spin integration tests (requires running server)
	@echo "$(CYAN)🧪 Running integration tests...$(NC)"
	@./test_spin_colored.sh

test-dashboard: ## Instructions for dashboard testing
	@echo "$(CYAN)🧪 Dashboard testing (manual):$(NC)"
	@echo "1. Ensure Spin is running: make dev"
	@echo "2. Open: http://127.0.0.1:3000/dashboard/index.html"
	@echo "3. Follow checklist in TESTING.md"

#--------------------------
# Utilities
#--------------------------

stop: ## Stop running Spin server
	@echo "$(CYAN)🛑 Stopping Spin server...$(NC)"
	@pkill -f "spin up" 2>/dev/null && echo "$(GREEN)✅ Stopped$(NC)" || echo "$(YELLOW)No server running$(NC)"

status: ## Check if Spin server is running
	@if curl -s http://127.0.0.1:3000/health > /dev/null 2>&1; then \
		echo "$(GREEN)✅ Spin server is running$(NC)"; \
		echo "   Dashboard: http://127.0.0.1:3000/dashboard/index.html"; \
	else \
		echo "$(YELLOW)⚠️  Spin server is not running$(NC)"; \
	fi

clean: ## Clean build artifacts
	@echo "$(CYAN)🧹 Cleaning build artifacts...$(NC)"
	@cargo clean
	@rm -rf .spin
	@echo "$(GREEN)✅ Clean complete$(NC)"

logs: ## View Spin component logs
	@echo "$(CYAN)📜 Spin logs:$(NC)"
	@cat .spin/logs/* 2>/dev/null || echo "No logs found. Run 'make dev' first."

#--------------------------
# Help
#--------------------------

help: ## Show this help message
	@echo "$(CYAN)WASM Bot Trap - Available Commands$(NC)"
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
	@grep -E '^(stop|status|clean|logs|help):.*?## ' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  make %-15s %s\n", $$1, $$2}'
