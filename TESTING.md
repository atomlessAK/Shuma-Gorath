# Testing Guide

## ⚠️ CRITICAL: Three Test Layers

This project has **three distinct types of tests** that run in different environments. Each uses the most appropriate tool for its scope.

## Quick Test Commands

```bash
# Run all tests (unit + integration if server running)
make test

# Run only unit tests
make test-unit

# Run only integration tests (requires Spin server)
make dev          # In one terminal
make test-integration  # In another terminal

# Test dashboard (manual)
open http://127.0.0.1:3000/dashboard/index.html
```

> **Preferred**: Always use Makefile commands instead of running scripts directly.

## Test Environment Summary

| Test Type | Count | Environment | Command | Requirements |
|-----------|-------|-------------|---------|--------------|
| **Unit Tests** | 34 | Native Rust | `make test-unit` | None (just Rust) |
| **Integration Tests** | 15 | Spin Environment | `make test-integration` | Spin server running |
| **Dashboard Tests** | Manual | Browser | Open dashboard URL | Spin server running |

---

## 1. Unit Tests (Native Rust Environment)

### What Are They?
Unit tests verify individual functions and logic in isolation. They run in the native Rust environment with NO external dependencies.

### How to Run
```bash
make test-unit          # Run all 34 unit tests
cargo test ban          # Run ban-related tests only
cargo test whitelist    # Run whitelist-related tests only
cargo test cdp          # Run CDP-related tests only
```

### What They Test
- **ban_tests.rs** (5 tests)
  - IP banning logic
  - Ban expiry handling
  - Ban entry serialization
  - Unban IP functionality
  - Unban non-existent IP handling

- **quiz_tests.rs** (2 tests)
  - Quiz question generation
  - Answer validation logic

- **whitelist_tests.rs** (4 tests)
  - IP address matching
  - CIDR range matching
  - Comment parsing
  - Whitespace handling

- **whitelist_path_tests.rs** (4 tests)
  - Exact path matching
  - Wildcard prefix matching
  - Comment parsing
  - Empty line handling

- **cdp_tests.rs** (8 tests)
  - CDP detection script generation
  - CDP report script with endpoint
  - CDP injection into HTML head
  - CDP injection into body fallback
  - CDP injection with report endpoint
  - Minimal HTML injection
  - CDP report deserialization
  - CDP report serialization

- **robots.rs tests** (2 tests)
  - robots.txt generation for AI training block
  - Honeypot path inclusion

### Why Native Rust?
- Fast execution (no server startup)
- No external dependencies
- Easy to run in CI/CD
- Tests logic in isolation

---

## 2. Integration Tests (Spin Environment ONLY)

### What Are They?
Integration tests verify the full HTTP API end-to-end. They **MUST** run in the Spin environment because they require:
- HTTP server and routing
- Spin key-value store
- Real HTTP headers (cookies, user-agent, x-forwarded-for)
- Authentication and API endpoints

### How to Run

#### Option A: Manual (recommended for debugging)
```bash
# Terminal 1: Start Spin server
make dev

# Terminal 2: Run integration tests
make test-integration
```

#### Option B: Makefile (automatic)
```bash
make test     # Runs unit tests + integration if server running
```

### What They Test (15 Scenarios)

1. **Health Check Endpoint**
   - `GET /health`
   - Verifies basic server functionality

2. **Root Endpoint Behavior**
   - `GET /`
   - Tests access control and challenge pages

3. **Honeypot Ban Detection**
   - `POST /bot-trap`
   - Verifies honeypot triggers ban correctly

4. **Admin API Unban**
   - `POST /admin/unban`
   - Verifies unban functionality

5. **Health Check After Ban/Unban**
   - `GET /health`
   - Verifies system stable after ban/unban cycle

6. **Config API - Get Config**
   - `GET /admin/config`
   - Returns current configuration including test_mode

7. **Test Mode Enable**
   - `POST /admin/config` with `{"test_mode": true}`
   - Enables test mode via API

8. **Test Mode Behavior - Honeypot**
   - `GET /bot-trap` with test_mode enabled
   - Verifies honeypot returns TEST MODE response (no actual ban)

9. **Test Mode Disable**
   - `POST /admin/config` with `{"test_mode": false}`
   - Disables test mode via API

10. **Blocking Resumes After Test Mode**
    - `GET /bot-trap` after test_mode disabled
    - Verifies real blocking resumes

11. **Prometheus Metrics Endpoint**
    - `GET /metrics`
    - Verifies Prometheus-formatted metrics with counters

12. **CDP Report Endpoint**
    - `POST /cdp-report`
    - Tests CDP detection report submission

13. **CDP Auto-Ban with High Score**
    - `POST /cdp-report` with score >= threshold
    - Verifies automation detection triggers appropriate action

14. **CDP Config via Admin API**
    - `GET /admin/cdp`
    - Returns CDP detection configuration and stats

15. **Unban Functionality Test**
    - `POST /admin/ban` then `POST /admin/unban`
    - Verifies ban/unban cycle works correctly

### Why Spin Environment?
Integration tests **cannot** run in native Rust because:
- They need HTTP server (Spin provides routing)
- They need key-value store (Spin provides KV API)
- They need real headers (cookies, IP addresses, user-agent)
- They need authentication (Bearer tokens in HTTP headers)
- They test end-to-end behavior, not isolated functions

---

## 3. Run All Tests

### Recommended: Use Makefile
```bash
make test          # Runs unit tests + integration if server running
make test-unit     # Unit tests only (34 tests)
make test-integration  # Integration tests only (15 scenarios)
```

Or use the combined script:
```bash
./test_all_colored.sh
```

This script:
1. Runs all 34 unit tests in native Rust
2. Builds the Spin app
3. Runs all 15 integration test scenarios in Spin
4. Provides clear, colorized output showing which environment each test runs in

### Output Example
```
============================================
  UNIT TESTS (Native Rust Environment)
  Run via: make test-unit
  Count: 34 tests
============================================

PASS All 34 unit tests passed

============================================
  INTEGRATION TESTS (Spin Environment)
  Run via: make test-integration
  Count: 15 scenarios
============================================

PASS All 15 integration test scenarios passed

============================================
  ALL TESTS COMPLETE
  Unit tests: 34/34 passed
  Integration tests: 15/15 scenarios passed
============================================
```

---

## Common Issues & Solutions

### Issue: "Integration tests fail"
**Problem:** Spin server not running  
**Solution:** Run `make dev` in a separate terminal first, then `make test-integration`

### Issue: "cargo test shows wrong output"
**Problem:** Stale build artifacts with wrong crate-type  
**Solution:** Run `cargo clean` before testing (scripts do this automatically)

### Issue: "Only seeing 1 test in tests/bot_trap.rs"
**Problem:** Looking at placeholder test instead of real integration tests  
**Solution:** Real integration tests are in `test_spin_colored.sh`, not `tests/bot_trap.rs`

### Issue: "Integration tests count wrong"
**Problem:** Confusing placeholder Rust test with real integration tests  
**Solution:** 
- `tests/bot_trap.rs` = 1 placeholder (not a real test)
- `test_spin_colored.sh` = 15 real integration test scenarios

---

## Why tests/bot_trap.rs Exists

The `tests/bot_trap.rs` file contains only a placeholder test that exists to:
1. Prevent cargo from complaining about an empty `tests/` directory
2. Remind developers that integration tests run in Spin, not cargo

**It is NOT a real integration test file.** All real integration tests are in `test_spin_colored.sh`.

---

## CI/CD Considerations

When setting up CI/CD, ensure your pipeline:

1. **Runs unit tests** with `make test-unit`
   - Fast, no external dependencies
   - Should run on every commit

2. **Runs integration tests** in Spin environment
   - Requires Spin installation
   - Requires `make dev` (can use background mode)
   - Runs `make test-integration`
   - Should run before deployment

### Example CI/CD Flow
```bash
# Step 1: Unit tests (fast)
make test-unit

# Step 2: Build
make build

# Step 3: Integration tests (requires Spin)
make dev &      # Start in background
sleep 5         # Wait for server
make test-integration
make stop

# Step 4: Deploy
make deploy
```

---

## 3. Dashboard Tests (Browser Environment)

### What Are They?
Dashboard tests verify the web UI functionality, charts, and admin controls. Currently manual, but can be automated with Jest/Cypress in the future.

### How to Run (Manual)
```bash
# 1. Start Spin server
make dev

# 2. Open dashboard
open http://127.0.0.1:3000/dashboard/index.html

# 3. Follow test checklist
```

### Test Checklist
- [ ] Dashboard loads without JavaScript errors
- [ ] Stat cards display correct numbers
- [ ] Event types chart renders (doughnut)
- [ ] Top IPs chart renders (bar)
- [ ] Time series chart renders (line)
- [ ] Time range buttons work (60 mins, 24h, 7d, 30d)
- [ ] Ban IP form submits correctly
- [ ] Unban IP form submits correctly
- [ ] Quick unban buttons work in table
- [ ] Enter key submits all forms
- [ ] Auto-refresh updates data every 30s
- [ ] Invalid API key shows error

### What They Test
- **Chart rendering** (Chart.js integration)
- **API integration** (fetch calls to admin endpoints)
- **Form validation** (IP address format, etc.)
- **User interactions** (button clicks, enter key)
- **Real-time updates** (auto-refresh mechanism)

### Why Browser-based?
- ✅ Tests actual user experience
- ✅ Catches CSS/layout issues
- ✅ Verifies Chart.js integration
- ✅ Tests JavaScript logic in real browser
- ❌ Slower than unit tests
- ❌ Requires manual verification (for now)

### Future: Automated Dashboard Tests
```bash
# When implemented:
cd dashboard
npm install
npm test              # Jest unit tests for JavaScript
npm run test:e2e      # Cypress/Playwright end-to-end tests
```

---

## Quick Reference

```bash
# Unit tests only (34 tests, native Rust)
make test-unit

# Integration tests only (15 scenarios, Spin required)
make dev            # In one terminal
make test-integration  # In another terminal

# Dashboard tests (manual checklist)
make dev            # Start server
open http://127.0.0.1:3000/dashboard/index.html

# All tests (recommended)
make test

# Clean build artifacts
make clean
```

---

## Test Counts Reference

**Always remember:**
- **34 unit tests** = Native Rust (`make test-unit`)
- **15 integration tests** = Spin environment (`make test-integration`)
- **12+ dashboard checks** = Browser manual testing
- **Total: 61+ test scenarios**

---

## Why Three Test Layers?

### Philosophy
Each test layer uses the **most appropriate tool** for its scope:

1. **Rust unit tests** = Fast feedback on logic
2. **Spin integration tests** = Verify HTTP/KV behavior
3. **Dashboard tests** = Ensure UI works correctly

### Tradeoffs
**Pros:**
- ✅ Each layer tests what it's best at
- ✅ Fast unit tests catch bugs early
- ✅ Integration tests catch deployment issues
- ✅ Dashboard tests catch UX problems

**Cons:**
- ⚠️ Three different commands to run
- ⚠️ Dashboard tests currently manual
- ⚠️ Requires documentation clarity

### Mitigation
- Clear documentation (this file!)
- Simple commands (`cargo test`, `./test_spin_colored.sh`)
- Unified script (`./test_all_colored.sh` for backend)
- Future automation for dashboard tests
