# Challenge Flow Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the placeholder math challenge with a deterministic ARC-style 4x4 grid challenge, add a dev-only manual page at `GET /challenge` (TEST_MODE only), wire verification at `POST /challenge`, and introduce an admin-configurable risk threshold for step-up challenge gating.

**Architecture:** Challenge generation is deterministic from a signed seed; verification recomputes expected output and uses KV to prevent replay. Production gating uses a simple risk score and serves the challenge inline when threshold is exceeded. Admin config for threshold is mutable only when `CHALLENGE_CONFIG_MUTABLE=true` (set in `make dev`).

**Tech Stack:** Rust (Spin SDK), KV store, HMAC signing, HTML/JS (inline), dashboard JS/CSS.

---

### Task 1: Add challenge config to server config

**Files:**
- Modify: `src/config.rs`
- Test: `src/config_tests.rs` (create)

**Step 1: Write the failing test**

Create `src/config_tests.rs` and add:
```rust
#[cfg(test)]
mod tests {
    use super::super::config::Config;
    use spin_sdk::key_value::Store;

    #[test]
    fn default_challenge_threshold_is_3() {
        let store = Store::open_default().unwrap();
        let cfg = Config::load(&store, "default");
        assert_eq!(cfg.challenge_risk_threshold, 3);
    }
}
```

**Step 2: Run test to verify it fails**
Run: `make test` (expect compile error: missing field `challenge_risk_threshold`).

**Step 3: Write minimal implementation**
- Add `challenge_risk_threshold: u8` to `Config` with default 3.
- Add clamp helper (`1..=10`) and env override from `CHALLENGE_RISK_THRESHOLD` similar to PoW defaults.
- Add `challenge_config_mutable()` reading `CHALLENGE_CONFIG_MUTABLE`.

**Step 4: Run test to verify it passes**
Run: `make test` (expect green unit tests).

**Step 5: Commit**
```
git add src/config.rs src/config_tests.rs
 git commit -m "feat: add challenge risk threshold config"
```

---

### Task 2: Implement challenge seed + transforms

**Files:**
- Modify: `src/challenge.rs`
- Test: `src/challenge_tests.rs`

**Step 1: Write failing tests**
Add tests for:
1. Determinism from seed
2. Transform correctness
3. Submission parser

Example (pseudo):
```rust
#[test]
fn deterministic_seed_produces_same_output() {
  let seed = fixed_seed();
  let a = build_puzzle(&seed);
  let b = build_puzzle(&seed);
  assert_eq!(a.test_output, b.test_output);
}
```

**Step 2: Run test to verify it fails**
Run: `make test` (expect new test failures).

**Step 3: Write minimal implementation**
- Define `ChallengeSeed` with fields (seed_id, issued_at, expires_at, ip_bucket, grid_size, active_cells, transforms, training_count, seed).
- Add HMAC signing helpers (similar to `pow.rs`).
- Implement transforms:
  - rotate cw/ccw
  - mirror h/v
  - shift up/down/left/right (no wrap)
  - drop/crop (remove edge, shift toward removed edge, fill new edge with zeros)
- Implement `build_puzzle(seed)` returning training pairs and test input/output.
- Implement submission parser: accept `0/1` string or CSV indexes.

**Step 4: Run tests to verify pass**
Run: `make test`.

**Step 5: Commit**
```
git add src/challenge.rs src/challenge_tests.rs
 git commit -m "feat: add deterministic challenge seed and transforms"
```

---

### Task 3: Wire GET /challenge (TEST_MODE only) and POST /challenge

**Files:**
- Modify: `src/lib.rs`
- Modify: `src/challenge.rs`
- Test: `src/challenge_tests.rs`

**Step 1: Write failing tests**
Add tests for:
- `GET /challenge` 404 when `TEST_MODE=false`
- `GET /challenge` 200 when `TEST_MODE=true`
- `POST /challenge` success on correct output

**Step 2: Run test to verify it fails**
Run: `make test`.

**Step 3: Implement**
- In `lib.rs`, route `GET /challenge` to `challenge::serve_challenge_page(req)`.
- `serve_challenge_page` checks test mode and returns 404 when disabled.
- `POST /challenge` verifies signature, TTL, IP bucket, replay protection.
- Store used seeds with TTL KV key `challenge_used:{seed_id}`.
- Render HTML page with training pairs, test input, empty output grid, inline JS toggles, and submit.

**Step 4: Run tests to verify pass**
Run: `make test`.

**Step 5: Commit**
```
git add src/lib.rs src/challenge.rs src/challenge_tests.rs
 git commit -m "feat: add challenge dev page and verification"
```

---

### Task 4: Add risk scoring + inline challenge gate

**Files:**
- Modify: `src/lib.rs`
- Modify: `src/rate.rs` (helper to read current window count)
- Test: `src/rate_tests.rs` (create if needed) and `src/lib.rs` tests

**Step 1: Write failing tests**
Add tests for:
- Risk score calculation with JS needed + geo risk + rate proximity
- Challenge triggers when score >= threshold

**Step 2: Run test to verify it fails**
Run: `make test`.

**Step 3: Implement**
- Add helper `rate::current_rate_usage` to read current window count without incrementing.
- Risk score: `js_needed + geo risk + rate proximity` with proximity buckets: >50% +1, >80% +2.
- If `score >= challenge_risk_threshold`, return inline challenge HTML.

**Step 4: Run tests to verify pass**
Run: `make test`.

**Step 5: Commit**
```
git add src/lib.rs src/rate.rs src/*_tests.rs
 git commit -m "feat: add challenge risk gate"
```

---

### Task 5: Admin config + dashboard UI for threshold

**Files:**
- Modify: `src/admin.rs`
- Modify: `dashboard/index.html`
- Modify: `dashboard/dashboard.js`
- Modify: `dashboard/style.css`
- Modify: `Makefile`
- Modify: `docs/api.md` and `docs/configuration.md`

**Step 1: Write failing tests**
Add tests to ensure:
- `GET /admin/config` includes challenge threshold fields
- `POST /admin/config` rejects updates when immutable

**Step 2: Run test to verify it fails**
Run: `make test`.

**Step 3: Implement**
- Add `challenge_risk_threshold` to admin config GET/POST.
- Enforce `CHALLENGE_CONFIG_MUTABLE` gating like PoW.
- Add dashboard UI block showing current value, default, and mutability.
- Add `make dev` export `CHALLENGE_CONFIG_MUTABLE=true`.

**Step 4: Run tests to verify pass**
Run: `make test`.

**Step 5: Commit**
```
git add src/admin.rs dashboard/index.html dashboard/dashboard.js dashboard/style.css Makefile docs/api.md docs/configuration.md
 git commit -m "feat: add challenge threshold admin control"
```

---

### Task 6: Docs updates

**Files:**
- Modify: `docs/challenge-verification.md`
- Modify: `docs/api.md`
- Modify: `README.md` (if needed)

**Step 1: Write doc tests**
N/A.

**Step 2: Update docs**
Add challenge endpoints, dev-only page, config flags.

**Step 3: Commit**
```
git add docs/challenge-verification.md docs/api.md README.md
 git commit -m "docs: document challenge flow and config"
```

---

## Execution

Proceed sequentially in this worktree using strict TDD and explicit spec + quality reviews after each task.
