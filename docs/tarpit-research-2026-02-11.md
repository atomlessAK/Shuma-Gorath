# Tarpit Research Report (2026-02-11)

## Scope

This report evaluates:

1. The Deadlocked Labyrinth concept from ASRG.
2. Current tarpit patterns in active/open GitHub projects.
3. A suitable tarpit design for Shuma-Gorath's architecture.

## Executive Summary

Shuma-Gorath should adopt a **tiered HTTP tarpit** that combines:

- deterministic synthetic maze content (already aligned with current `/maze` behavior),
- optional bounded slow-drip responses for high-confidence bot traffic,
- strict concurrency budgets and fail-safe fallback behavior.

This gives most of the defensive value from modern tarpits without turning the component into a self-DoS risk.

## Deadlocked Labyrinth: What Is Innovative

From the ASRG Deadlocked page, the key ideas are:

- infinite linked pages that keep crawlers exploring a dead-end graph,
- deterministic but nonsensical generated text,
- fake but valid-looking media assets,
- operational goal: trap crawler time and degrade extraction value.

Important practical note: Deadlocked currently presents itself as work in progress and references external codebases rather than a fully stable, widely deployed package.

## GitHub Landscape (What Is Actually Being Built)

### 1. Protocol-level tarpits (mature pattern)

- `skeeto/endlessh` (8.4k stars): classic SSH banner tarpit.
- `shizunge/endlessh-go` (1.2k stars): same concept with metrics and dashboarding.
- `Freaky/tarssh` and `iBug/fakessh`: modern SSH tarpit variants.

Takeaway: "slow trickle + long hold" is proven at protocol level, but this does not map 1:1 to web workloads without strict budgets.

### 2. HTTP tarpits (older but still relevant)

- `die-net/http-tarpit`: one-byte periodic HTTP responses to tie client pools with low server cost.
- `msigley/PHP-HTTP-Tarpit`: mixed deception modes (garbage, redirects, etc.).
- `p0pr0ck5/lua-resty-tarpit`: OpenResty latency inflation for brute-force-prone endpoints.

Takeaway: delay-based HTTP tarpits work, but need careful resource accounting and route scoping.

### 3. AI crawler-focused tarpits (newer wave)

- `JasonLovesDoggo/caddy-defender` (497 stars, recently active): includes tarpit responder with configurable content source, timeout, and bytes/sec.
- `0x4D31/finch` (283 stars, recently active): fingerprint-aware reverse proxy; supports `tarpit` action and explicitly limits tarpit concurrency.
- `amenyxia/Sarracenia` (recently active): anti-scraper tarpit with staged threat levels, Markov/template generation, and optional drip-feed.
- `circa10a/ai-troller` and `JonasLong/Pyison`: content-streaming and synthetic-link maze approaches targeting crawler cost.

Takeaway: cutting-edge implementations are converging on:

- adaptive policy triggers (fingerprints/heuristics),
- bounded slow streaming,
- generated decoy content,
- operator controls and observability.

## Fit for Shuma-Gorath

Shuma already has strong primitives:

- botness scoring and routing thresholds,
- maze generation with deterministic fake pages and link graphs,
- admin controls and metrics infrastructure.

This means Shuma should **extend current maze/routing**, not create a separate tarpit subsystem from scratch.

## Recommended Design for Shuma: Tiered Adaptive Tarpit

### Tier 1: Content Maze (default tarpit path)

- Reuse and expand existing maze behavior for suspicious traffic.
- Increase decoy depth/entropy safely (more link fan-out, varied templates, fake static assets).
- Keep responses fast enough to avoid server-side connection hoarding.

### Tier 2: Bounded Slow-Drip (high-confidence bots only)

- Apply only when confidence is high (botness threshold and/or additional signals).
- Stream at low rate (for example 16-48 B/s) with hard timeout (for example 15-45s).
- Enforce strict budgets:
  - max concurrent tarpit streams,
  - per-IP/per-bucket stream caps,
  - fallback action when budget exhausted (maze or block).

### Tier 3: Escalation

- If tarpit interaction persists past threshold, escalate to ban/block policy (existing ban path).

## Proposed Runtime Controls (Admin Config)

- `tarpit_enabled` (bool)
- `tarpit_mode` (`maze_only` | `maze_plus_drip`)
- `tarpit_trigger_score` (u8)
- `tarpit_stream_timeout_seconds` (u64)
- `tarpit_bytes_per_second` (u32)
- `tarpit_max_concurrent_streams` (u32)
- `tarpit_max_streams_per_ip_bucket` (u32)
- `tarpit_fallback_action` (`maze` | `block`)

## Observability Requirements

Add metrics and admin visibility for:

- `tarpit_requests_total`
- `tarpit_streams_active`
- `tarpit_stream_duration_seconds_total`
- `tarpit_bytes_sent_total`
- `tarpit_budget_exhausted_total`
- `tarpit_escalated_to_ban_total`

Without these, tuning is guesswork.

## Risks and Mitigations

1. Self-DoS risk from long-lived responses
- Mitigation: hard concurrency caps + timeout + fallback action.

2. Multi-instance consistency drift
- Mitigation: treat tarpit counters as best-effort now; align durable distributed limits with planned Redis/global-state work.

3. Legal/reputation risk from explicit "poisoning" claims
- Mitigation: frame and implement as defensive resource sink/deception only, not training-data sabotage.

4. False positives harming real users
- Mitigation: only apply drip tier on high-confidence signals; keep challenge and allow paths intact.

## Delivery Plan

### Phase A (low risk)

- Implement `maze_only` tarpit mode in existing routing flow.
- Add admin controls and metrics.
- No long-lived streaming yet.

### Phase B (guarded drip)

- Add bounded drip streaming path with strict caps and fallback.
- Add alerting for budget saturation and latency impact.

### Phase C (distributed hardening)

- Integrate with planned Redis/distributed state for stronger global limits and cross-instance consistency.

## Bottom Line Recommendation

For Shuma, the most suitable tarpit is:

- **adaptive maze-first**, with
- **optional capped drip tier** for high-confidence abusive traffic, and
- **clear operator controls plus metrics**.

This follows the strongest ideas from Deadlocked and current GitHub implementations while staying compatible with Shuma's current architecture and risk profile.

## Sources

- ASRG Deadlocked:
  - https://algorithmic-sabotage.gitlab.io/asrg/deadlocked/
- Babble (referenced by Deadlocked):
  - https://git.jsbarretto.com/zesterer/babble
- fakejpeg (referenced by Deadlocked):
  - https://github.com/gw1urf/fakejpeg
- endlessh:
  - https://github.com/skeeto/endlessh
- endlessh-go:
  - https://github.com/shizunge/endlessh-go
- tarssh:
  - https://github.com/Freaky/tarssh
- fakessh:
  - https://github.com/iBug/fakessh
- http-tarpit:
  - https://github.com/die-net/http-tarpit
- lua-resty-tarpit:
  - https://github.com/p0pr0ck5/lua-resty-tarpit
- PHP-HTTP-Tarpit:
  - https://github.com/msigley/PHP-HTTP-Tarpit
- caddy-defender:
  - https://github.com/JasonLovesDoggo/caddy-defender
  - https://raw.githubusercontent.com/JasonLovesDoggo/caddy-defender/main/docs/examples.md
- Finch:
  - https://github.com/0x4D31/finch
  - https://raw.githubusercontent.com/0x4D31/finch/main/docs/rule-schema.md
- Sarracenia:
  - https://github.com/amenyxia/Sarracenia
- ai-troller:
  - https://github.com/circa10a/ai-troller
- Pyison:
  - https://github.com/JonasLong/Pyison
