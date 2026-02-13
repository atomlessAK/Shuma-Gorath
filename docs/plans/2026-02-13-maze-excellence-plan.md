# Maze Excellence Plan

Date: 2026-02-13
Status: Proposed

## Context

Current maze generation in Shuma-Gorath is deterministic from a path hash (`path_to_seed(path)`), which makes each URL stable across requests. This keeps implementation simple but creates a fingerprinting weakness: sophisticated crawlers can compare responses for the same URL and classify the maze.

## Goals

- Make maze responses and traversal paths materially harder to fingerprint.
- Increase attacker-side compute/bandwidth/time cost faster than defender cost.
- Keep human friction low and avoid SEO regressions.
- Keep self-hosted default operation lightweight, with optional stronger enterprise modes.

## Non-goals

- Do not require external managed services for basic self-hosted maze operation.
- Do not break accessibility or legitimate no-JS usage on non-suspicious traffic.
- Do not make maze runtime cost unbounded on host infrastructure.

## State-of-the-art Signals

1. Deceptive crawl redirection is mainstreaming:
   - Cloudflare AI Labyrinth explicitly uses hidden links and decoy networks for suspected bots rather than only hard block responses.
2. Practical bot defense stacks now combine multiple orthogonal signals:
   - transport fingerprints (JA3/JA4),
   - detection IDs/heuristics,
   - request sequence/timing features.
3. Enterprise bot mitigation guidance is response-tiered:
   - monitor first, then graduate to deny/challenge/tarpit for higher-confidence traffic (Akamai Bot Manager guidance).
4. Research emphasizes that honeypots are fingerprintable:
   - static, repeated, and structurally predictable decoys are eventually evaded.
5. Request-path and timing features are effective for crawler discrimination:
   - marker-based path provenance + traversal timing materially improve detection quality.
6. Client-side challenge systems (for example JS/WASM proof-of-work) can shift cost to requestors, but must be deployed carefully to avoid harming legitimate users.

## Design Principles For Shuma Maze

1. No globally stable maze page per URL for suspicious traffic.
2. Keep generation cost mostly amortized or offloaded:
   - pre-generate where possible,
   - use client compute for expansion/proof in high-risk paths.
3. Signed traversal primitives:
   - links and steps should be verifiable, replay-limited, and depth-scoped.
4. Adaptive complexity:
   - low suspicion: minimal or no maze overhead,
   - high suspicion: deeper decoy graph + optional client proof/puzzle.
5. Bounded infrastructure:
   - hard budgets for concurrent maze work, per-IP spend, and response bytes.

## Proposed Architecture

### A. Anti-fingerprinting generation model

- Replace `seed = hash(path)` with signed, rotating entropy:
  - `seed = HMAC(maze_epoch_key, site_id | ip_bucket | ua_bucket | path | minute_bucket | chain_nonce)`.
- Keep deterministic behavior only within short TTL windows (for cacheability and debugging), not globally.
- Add content/template polymorphism:
  - structural variants (layout blocks/order),
  - lexical variants (seed corpora),
  - link graph variants (branching/depth profile).

### B. Signed traversal links and replay resistance

- Every maze link carries a compact signed token:
  - issue timestamp, expiry, depth, branch budget, previous node digest.
- Reject expired/replayed/forged tokens.
- Record traversal chain features for botness feedback:
  - depth growth rate,
  - breadth pattern,
  - timing regularity.

### C. Covert decoy injection in normal HTML

- For medium-confidence suspicious traffic, inject invisible-to-human decoy links into eligible HTML responses (not only explicit `/maze` routes).
- Keep explicit `/maze` and `/trap` as fallback/test surface.
- Ensure SEO-safe directives and avoid user-visible regressions.

### D. Client-side expansion foundation for suspicious tiers (cost shift to requester)

Short answer: yes, we can generate maze branches on the visitor side with JS.

Recommended shape (hybrid, not JS-only):
- Server returns a small signed bootstrap payload.
- Browser-side Web Worker generates branch graph and decoy content variants from signed seed.
- Client sends periodic proof-of-execution/progress markers.
- Server verifies proof before issuing deeper traversal nodes.

Tier policy:
- suspicious traffic entering maze (`medium` and `high` suspicion) uses client-side expansion by default.
- low-suspicion or non-maze traffic does not require this path.

Checkpoint cadence (explicit):
- client must submit signed progress checkpoints every 3 generated nodes or every 1500 ms, whichever happens first.
- server issues only bounded step-ahead allowance per checkpoint (max unverified depth 4).
- if checkpoint is missing/late/invalid, server stops expansion and applies configured fallback (`challenge` then `block`, or `maze` then `block` by policy).

Benefits:
- Moves part of CPU/memory cost to scraper infrastructure.
- Increases cost for bots that execute JS and quickly identifies bots that do not.

Risks and mitigations:
- Some legitimate users disable JS:
  - do not use this mode for low-risk traffic;
  - for suspicious maze traffic with no JS, fall back to bounded server-side maze traversal (reduced depth/TTL) and then challenge escalation.
  - do not hard-block solely on no-JS at first contact; require corroborating abuse signals or repeated failed fallbacks.
- Advanced bots can run JS:
  - combine with timing/sequence/provenance checks and optional micro-PoW for deep traversal steps.

### E. Host-side cost controls

- Pre-generate decoy content corpus offline or in low-priority background jobs.
- Cache reusable fragments and avoid expensive per-request synthesis.
- Apply strict budgets:
  - global concurrent maze responses,
  - per-IP/UA bucket budgets,
  - hard response byte caps/timeouts.
- Deterministic fallback when budgets are exhausted (`maze` -> `challenge` or `block` based on policy).

### F. Maze/Tarpit convergence contract (shared implementation required)

Maze and tarpit are separate defence modes, but they must share foundational mechanics to prevent drift and contradictory behavior.

Shared primitives (single implementation path):

- Signed deception token envelope:
  - shared fields and validation logic for traversal/drip progression.
- Shared budget governor:
  - global concurrency caps,
  - per-bucket spend limits,
  - byte/time caps.
- Shared fallback policy matrix:
  - deterministic fallback precedence when budgets saturate.
- Shared observability schema:
  - common labels/fields for mode, budget reason, token outcome, and fallback action.

Non-duplication rule:

- `MZ-2` and `MZ-7` must be implemented using shared primitives also consumed by HTTP tarpit tasks (`TP-*`).
- Do not create a separate tarpit-only token format or budget/fallback logic path.

### G. Cost-allocation targets (bot over host)

Target split for suspicious traffic after rollout:

| Cost component | Target bearer | How |
| --- | --- | --- |
| Branch generation CPU | Bot (primary) | Web Worker branch expansion from signed seed; server only verifies proofs/tokens. |
| Traversal bandwidth | Bot (primary) | Deeper traversal requires repeated request/proof cycles; optional micro-PoW at depth. |
| Per-step validation CPU | Shared, host-bounded | Constant-time token/proof checks and strict depth/TTL limits. |
| Content synthesis CPU | Host (bounded) | Pre-generated variant corpus + cached fragments, not full dynamic rendering per request. |
| State tracking | Host (bounded) | Compact replay cache with short TTL and per-bucket quotas. |

Operational guardrails to enforce this:

- keep server-side verification `O(1)` per request (signature + bounded replay lookup),
- cap replay-cache memory and prune by TTL,
- cap bytes per response and max traversal depth per issued token,
- disable deep traversal/micro-PoW when host load exceeds thresholds,
- fail to lower-cost actions (`challenge`, then `block`) when maze budgets saturate.

### H. Observability and feedback loop

Add metrics/log dimensions for:
- `maze_variant_id` and entropy source version,
- token validation failures (expired/replay/signature),
- client-proof pass/fail and latency,
- per-request maze generation cost and bytes sent,
- budget saturation and fallback path,
- botness uplift from maze traversal patterns.

## Rollout Strategy

1. Instrument-only phase:
   - add entropy/token/proof fields and logs without behavior change.
2. Advisory phase:
   - enable covert decoys + client proof in monitor mode for suspicious cohorts.
3. Controlled enforcement:
   - progressively increase depth/proof requirements and tarpit escalation bands.
4. Optimization pass:
   - tune for defender resource efficiency and false-positive control.

## Structured Implementation TODOs (Execution Order)

1. MZ-1 Entropy model:
   - implement rotating signed seed policy; retire path-only determinism for suspicious paths.
2. MZ-2 Tokenized traversal:
   - add signed maze link tokens with TTL/depth/replay controls.
3. MZ-3 Polymorphic rendering:
   - add layout/content/link-graph variant families with versioned selection.
4. MZ-4 Covert decoy injection:
   - inject decoys into eligible HTML for medium-confidence suspicious traffic.
5. MZ-5 Client-side expansion foundation:
   - make Web Worker branch generation + signed server verification the default maze path for suspicious tiers, with explicit checkpoint cadence and bounded step-ahead allowance.
   - add explicit no-JS fallback rules (bounded server-side maze path first, then challenge escalation) without immediate no-JS-only hard block.
6. MZ-6 Optional micro-PoW for deep traversal:
   - adaptive difficulty by risk/depth to increase attacker cost.
7. MZ-7 Budget enforcement:
   - global/per-bucket limits, byte caps, hard timeouts, deterministic fallback.
8. MZ-8 Test harness:
   - crawler simulation for replay, deterministic fingerprinting, JS/no-JS, and bypass attempts.
9. MZ-9 Observability:
   - expose maze cost/entropy/proof/budget metrics and dashboards.
10. MZ-10 Staged rollout:
   - monitor -> advisory -> enforce with explicit rollback triggers.

## Source References

- Cloudflare blog: Trapping misbehaving bots in an AI Labyrinth
  - https://blog.cloudflare.com/ai-labyrinth/
- Cloudflare docs: AI Labyrinth
  - https://developers.cloudflare.com/bots/additional-configurations/ai-labyrinth/
- Cloudflare docs: Additional bot configurations (JA3/JA4, detection IDs, sequence rules, static resource protection)
  - https://developers.cloudflare.com/bots/additional-configurations/
- Cloudflare docs: Sequence rules
  - https://developers.cloudflare.com/bots/additional-configurations/sequence-rules/
- Cloudflare docs: JA3/JA4 fingerprint
  - https://developers.cloudflare.com/bots/additional-configurations/ja3-ja4-fingerprint/
- Cloudflare docs: Detection IDs
  - https://developers.cloudflare.com/bots/additional-configurations/detection-ids/
- Cloudflare docs: JavaScript Detections
  - https://developers.cloudflare.com/cloudflare-challenges/challenge-types/javascript-detections/
- Akamai docs: Handle adversarial bots
  - https://techdocs.akamai.com/cloud-security/docs/handle-adversarial-bots
- PathMarker paper (open access)
  - https://link.springer.com/article/10.1186/s42400-019-0023-1
- Honeypot fingerprinting research metadata (DOI 10.1145/3584976)
  - https://vbn.aau.dk/en/publications/gotta-catch-em-all-a-multistage-framework-for-honeypot-fingerprin
- Anubis (JS/WASM challenge-based anti-scraping project)
  - https://github.com/TecharoHQ/anubis
