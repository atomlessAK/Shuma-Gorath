# TODO Roadmap

Last updated: 2026-02-13

This is the active work queue.
`todos/security-review.md` tracks security finding validity and closure status.

## Direction Snapshot (for next implementation stages)
- [ ] Follow internal-first delivery policy: harden Shuma-native capability paths before completing external-provider parity for the same capability; use enterprise/Akamai patterns to inform design, not as baseline dependencies.
- [ ] Evolve maze behavior toward Cloudflare-style selective covert decoys for suspicious traffic while keeping explicit `/maze` and `/trap` endpoints as fallback and test surface.
- [ ] Build Shuma-native bounded slow-drip tarpit behavior in Rust/Spin; treat external projects (for example Finch/Sarracenia/caddy-defender) as design references, not runtime dependencies.
- [ ] Strengthen fingerprinting by hardening internal baseline signals first, then ingesting trusted upstream edge signals (JA3/JA4 and similar) with provenance checks and explicit internal fallback when edge headers are absent or untrusted.
- [ ] Refactor to clearer in-repo modules first; defer multi-repo splits until module boundaries and interfaces are stable.

## P1 Research Dossiers (Paper-by-Paper TODOs)
Completion rule for every paper TODO below: capture key findings, map to `self_hosted_minimal` vs `enterprise_akamai` ownership, and propose concrete Shuma TODO updates.

### Fingerprinting, JS Verification, and CDP-Adjacent Detection
- [ ] R-FP-01 Review Peter Eckersley, "How Unique Is Your Web Browser?" (PETS 2010) and extract entropy-design implications for Shuma fingerprint signals and replay windows. https://link.springer.com/chapter/10.1007/978-3-642-14527-8_1
- [ ] R-FP-02 Review Acar et al., "The Web Never Forgets" (CCS 2014) and derive tracking/fingerprint abuse patterns relevant to bot-detection evasion hardening. https://doi.org/10.1145/2660267.2660347
- [ ] R-FP-03 Review Vastel et al., "FP-STALKER" (IEEE S&P 2018) and define time-evolution checks for Shuma fingerprint consistency logic. https://doi.org/10.1109/SP.2018.00008
- [ ] R-FP-04 Review Jonker/Krumnow/Vlot, "Fingerprint Surface-Based Detection of Web Bot Detectors" (ESORICS 2019) and identify detector-surface minimization requirements. https://doi.org/10.1007/978-3-030-29962-0_28
- [ ] R-FP-05 Review Azad et al., "Web Runner 2049: Evaluating Third-Party Anti-bot Services" and extract anti-evasion architecture lessons for internal-vs-edge integration boundaries. https://pmc.ncbi.nlm.nih.gov/articles/PMC7338186/
- [ ] R-FP-06 Review Iliou et al., "Detection of advanced web bots by combining web logs with mouse behavioural biometrics" (DTRAP 2021) and assess feasibility of low-friction behavior features in Shuma. https://doi.org/10.1145/3447815
- [ ] R-FP-07 Review Zhao et al., "Toward the flow-centric detection of browser fingerprinting" (Computers & Security 2024) and evaluate flow-level JS signal extraction options. https://doi.org/10.1016/j.cose.2023.103642
- [ ] R-FP-08 Review Venugopalan et al., "FP-Inconsistent: Detecting Evasive Bots using Browser Fingerprint Inconsistencies" (2024) and define cross-attribute consistency checks for Shuma scoring. https://arxiv.org/abs/2406.07647
- [ ] R-FP-09 Review Bursztein et al., "Picasso: Lightweight Device Class Fingerprinting for Web Clients" (SPSM 2016) and assess replay-resistant challenge-bound fingerprint options. https://doi.org/10.1145/2994459.2994467
- [ ] R-FP-10 Review Li et al., "PathMarker: protecting web contents against inside crawlers" (Cybersecurity 2019) and map path/timing marker concepts to Shuma detection IDs. https://cybersecurity.springeropen.com/articles/10.1186/s42400-019-0023-1

### Challenges: PoW, Not-a-Bot, and Puzzle Escalation
- [ ] R-CH-01 Review Dwork/Naor, "Pricing via Processing or Combatting Junk Mail" (CRYPTO 1992) and extract adaptive requester-cost principles for modern web bot defence. https://www.microsoft.com/en-us/research/publication/pricing-via-processing-or-combatting-junk-mail/
- [ ] R-CH-02 Review Juels/Brainard, "Client Puzzles" (NDSS 1999) and define stateless verification patterns for Shuma PoW endpoints. https://www.ndss-symposium.org/ndss1999/cryptographic-defense-against-connection-depletion-attacks/
- [ ] R-CH-03 Review Adam Back, "Hashcash: A Denial of Service Counter-Measure" (2002) and assess modern browser-side PoW cost tuning constraints. https://nakamotoinstitute.org/library/hashcash/
- [ ] R-CH-04 Review von Ahn et al., "CAPTCHA: Using Hard AI Problems for Security" (EUROCRYPT 2003) and capture challenge-design principles still valid for Challenge Lite. https://doi.org/10.1007/3-540-39200-9_18
- [ ] R-CH-05 Review von Ahn et al., "reCAPTCHA: Human-based character recognition via Web security measures" (Science 2008) and extract lessons for useful-human-work and abuse resistance tradeoffs. https://doi.org/10.1126/science.1160379
- [ ] R-CH-06 Review Bursztein et al., "Easy Does It: More Usable CAPTCHAs" (CHI 2014) and derive practical usability thresholds/metrics for Shuma challenge UX. https://doi.org/10.1145/2556288.2557322
- [ ] R-CH-07 Review Golle, "Machine Learning Attacks Against the ASIRRA CAPTCHA" (CCS 2008) and define anti-ML solvability requirements for puzzle challenge variants. https://doi.org/10.1145/1455770.1455838
- [ ] R-CH-08 Review AI_Adaptive_POW (Software Impacts 2022) and evaluate adaptive-difficulty policies for botness-tiered PoW in Shuma. https://doi.org/10.1016/j.simpa.2022.100335
- [ ] R-CH-09 Review Alsuhibany, "A Survey on Adversarial Perturbations and Attacks on CAPTCHAs" (Applied Sciences 2023) and map attack classes to Shuma challenge threat model updates. https://doi.org/10.3390/app13074602
- [ ] R-CH-10 Review Uysal, "Revisiting Text-Based CAPTCHAs" (Electronics 2025) and evaluate current CNN-solvability implications for fallback challenge modes. https://doi.org/10.3390/electronics14224403

### Rate Limiting, Tarpit, and Cost-Imposition
- [ ] R-RL-01 Review Raghavan et al., "Cloud Control with Distributed Rate Limiting" (SIGCOMM 2007) and extract distributed limiter semantics for Shuma provider adapters. https://www.microsoft.com/en-us/research/publication/cloud-control-with-distributed-rate-limiting/
- [ ] R-RL-02 Review Kuzmanovic/Knightly, "Low-Rate TCP-Targeted Denial of Service Attacks" (SIGCOMM 2003) and map low-rate adversary behaviors to Shuma tarpit/limiter heuristics. https://doi.org/10.1145/863955.863966
- [ ] R-RL-03 Review Veroff et al., "Evaluation of a low-rate DoS attack against application servers" (Computers & Security 2008) and capture queue/resource-starvation mitigation patterns. https://doi.org/10.1016/j.cose.2008.07.004
- [ ] R-RL-04 Review Veroff et al., "Defense techniques for low-rate DoS attacks against application servers" (Computer Networks 2010) and identify bounded-randomization strategies usable in Shuma tarpit controls. https://doi.org/10.1016/j.comnet.2010.05.002
- [ ] R-RL-05 Review Srivatsa et al., "Mitigating application-level denial of service attacks on Web servers" (ACM TWEB 2008) and assess admission/congestion control patterns for Shuma policy pipeline. https://research.ibm.com/publications/mitigating-application-level-denial-of-service-attacks-on-web-servers-a-client-transparent-approach
- [ ] R-RL-06 Review Lemon, "Resisting SYN flood DoS attacks with a SYN cache" (BSDCon 2002) and capture edge-vs-origin queue protection lessons relevant to Akamai authoritative mode. https://www.usenix.org/legacy/publications/library/proceedings/bsdcon02/full_papers/lemon/lemon_html/index.html
- [ ] R-RL-07 Review Chen et al., "SMARTCOOKIE" (USENIX Security 2024) and evaluate split-proxy edge-cookie architecture fit for enterprise Akamai deployments. https://collaborate.princeton.edu/en/publications/smartcookie-blocking-large-scale-syn-floods-with-a-split-proxy-de/
- [ ] R-RL-08 Review Vedula et al., "On the Detection of Low-Rate Denial of Service Attacks at Transport and Application Layers" (Electronics 2021) and map detector candidates to Shuma observability/tuning. https://doi.org/10.3390/electronics10172105

### IP Range Policy, Reputation Feeds, and GEO Fencing
- [ ] R-IP-01 Review Ramanathan et al., "BLAG: Improving the Accuracy of Blacklists" (NDSS 2020) and derive feed-aggregation + false-positive controls for managed CIDR sets. https://doi.org/10.14722/ndss.2020.24232
- [ ] R-IP-02 Review Sheng et al., "An Empirical Analysis of Phishing Blacklists" (2009) and extract freshness/latency requirements for update cadence and rollout safety. https://kilthub.cmu.edu/articles/journal_contribution/An_Empirical_Analysis_of_Phishing_Blacklists/6469805
- [ ] R-IP-03 Review Oest et al., "PhishTime" (USENIX Security 2020) and map continuous quality-measurement methodology to Shuma feed validation. https://www.usenix.org/conference/usenixsecurity20/presentation/oest-phishtime
- [ ] R-IP-04 Review Li et al., "HADES Attack" (NDSS 2025) and define anti-poisoning controls for any external blocklist ingestion pipeline. https://doi.org/10.14722/ndss.2025.242156
- [ ] R-IP-05 Review Deri/Fusco, "Evaluating IP Blacklists effectiveness" (FiCloud 2023) and identify practical precision/recall limits for aggressive edge enforcement. https://research.ibm.com/publications/evaluating-ip-blacklists-effectiveness
- [ ] R-GEO-01 Review Hu/Heidemann/Pradkin, "Towards Geolocation of Millions of IP Addresses" (IMC 2012) and capture scalability/error-tradeoff implications for GEO policy confidence scoring. https://doi.org/10.1145/2398776.2398790
- [ ] R-GEO-02 Review Dan/Parikh/Davison, "Improving IP Geolocation using Query Logs" (WSDM 2016) and define data-quality assumptions for geo-based enforcement. https://doi.org/10.1145/2835776.2835820
- [ ] R-GEO-03 Review Mazel et al., "Smartphone-based geolocation of Internet hosts" (Computer Networks 2017) and assess delay-model caveats for operational geofencing. https://doi.org/10.1016/j.comnet.2017.02.006
- [ ] R-GEO-04 Review Saxon/Feamster, "GPS-Based Geolocation of Consumer IP Addresses" (2021) and define confidence thresholds for city-level policy decisions. https://arxiv.org/abs/2105.13389

### SSH Tarpit and Honeypot Evasion Resistance
- [ ] R-SSH-01 Review Vasilomanolakis et al., "Gotta catch 'em all: A Multistage Framework for Honeypot Fingerprinting" (Digital Threats 2023) and derive anti-fingerprint requirements for SSH tarpit realism. https://doi.org/10.1145/3584976
- [ ] R-SSH-02 Review Bythwood et al., "Fingerprinting Bots in a Hybrid Honeypot" (IEEE SoutheastCon 2023) and assess hybrid interaction design implications for SSH deception tiers. https://doi.org/10.1109/SoutheastCon51012.2023.10115143
- [ ] R-SSH-03 Review Vetterl et al., "A Comparison of an Adaptive Self-Guarded Honeypot with Conventional Honeypots" (Applied Sciences 2022) and evaluate adaptive risk-vs-observability controls for Shuma SSH tarpit mode. https://doi.org/10.3390/app12105224
- [ ] R-SSH-04 Review Cordeiro/Vasilomanolakis, "Towards agnostic OT honeypot fingerprinting" (TMA 2025) and extract transport-stack realism requirements applicable to SSH tarpit surfaces. https://doi.org/10.23919/TMA66427.2025.11097018

## Modularization Sprint (Active)
- [x] Define sprint guardrails: refactor-only, no behavior changes, no new dependencies, tests must pass before each checkoff.
- [x] M1 Extract inline test-mode block from `src/lib.rs` into dedicated test-mode module (`src/runtime/test_mode/mod.rs`).
- [x] M2 Add focused unit tests for extracted test-mode behavior (cover bypass, block, and allow outcomes).
- [x] M3 Keep `src/lib.rs` behavior identical by routing existing test-mode flow through the new module.
- [x] M4 Run verification (`cargo test` and integration smoke path) and record result.
- [x] M5 Plan and execute next extraction slice from `src/lib.rs` (routing/decision helpers) with similarly scoped checklist items.
- [x] M5.1 Extract early endpoint routing (`/health`, `/admin`, `/metrics`, `/robots.txt`, challenge endpoints) into a dedicated router helper/module without changing semantics.
- [x] M5.2 Extract KV outage/open-close gate into a dedicated helper to isolate fail-open/fail-closed behavior.
- [x] M5.3 Extract post-config enforcement pipeline ordering into named helpers (honeypot, rate, ban, geo policy, botness, JS).
- [x] M5.4 Add regression tests for routing order and short-circuit precedence after extraction.
- [x] M6 Dashboard decomposition track (`dashboard/dashboard.js` split into domain modules under `dashboard/modules/`).
- [x] M6.1 Extract charts/timeseries logic into `dashboard/modules/charts.js` and wire via a stable API surface.
- [x] M6.2 Extract status panel state/rendering into `dashboard/modules/status.js` and remove status-specific globals from root script.
- [x] M6.3 Extract config/tuning save handlers into `dashboard/modules/config-controls.js`.
- [x] M6.4 Extract admin actions/session endpoint wiring into `dashboard/modules/admin-session.js`.
- [x] M6.5 Add frontend smoke checks for key interactions (login/session restore, chart refresh, config save buttons enabled/disabled state).
- [x] M7 Maze domain decomposition track (split `src/maze.rs` into `src/maze/` submodules: generation, routing hooks, telemetry, templates).
- [x] M7.1 Convert `src/maze.rs` into `src/maze/mod.rs` with identical public API (`is_maze_path`, `handle_maze_request`, `generate_maze_page`, `MazeConfig`).
- [x] M7.2 Extract deterministic generation primitives (`SeededRng`, path seeding, content generators) into focused submodules.
- [x] M7.3 Extract HTML rendering/template assembly into a dedicated maze template module.
- [x] M7.4 Isolate maze request/path helpers so later telemetry/routing-hook extraction from `src/lib.rs` has a stable seam.
- [x] M7.5 Keep/extend maze tests after extraction and run verification (`cargo test`, integration smoke).
- [x] M8 Challenge domain decomposition track (split `src/challenge.rs` into `src/challenge/` submodules: token/crypto, puzzle generation, HTTP handlers, validation/anti-replay).
- [x] M8.1 Convert `src/challenge.rs` into `src/challenge/mod.rs` while preserving public API used by `src/lib.rs` and tests.
- [x] M8.2 Extract seed token/HMAC logic into a dedicated `src/challenge/puzzle/token.rs` module.
- [x] M8.3 Extract puzzle generation/transform/validation logic into `src/challenge/puzzle/mod.rs`.
- [x] M8.4 Extract rendering and submit/anti-replay flow into focused HTTP modules (`src/challenge/puzzle/renders.rs`, `src/challenge/puzzle/submit.rs`).
- [x] M8.5 Run verification (`cargo test`) to confirm no behavior change.
- [x] M9 Directory structure prep for future repo boundaries (core policy, maze+tarpit, challenge, dashboard adapter) with explicit interface contracts.
- [x] M9.1 Add explicit Rust boundary contracts for challenge/maze/admin in `src/boundaries/contracts.rs`.
- [x] M9.2 Add default adapter implementations in `src/boundaries/adapters.rs` and route `src/lib.rs` through `src/boundaries/`.
- [x] M9.3 Document boundary rules and target split direction in `docs/module-boundaries.md`.

## P0 Immediate (ops and abuse containment)
- [x] Enforce `SHUMA_ADMIN_IP_ALLOWLIST` in every production environment.
- [x] Configure CDN/WAF rate limits for `POST /admin/login` and all `/admin/*` in every deployment (Cloudflare and Akamai guidance already documented).
- [x] Rotate `SHUMA_API_KEY` using `make gen-admin-api-key` and set a regular rotation cadence.
- [x] Add deployment runbook checks for admin exposure, allowlist status, and login rate-limit posture.
- [x] Add stronger admin controls for production tuning: split read/write privileges and keep audit visibility for write actions. (`src/auth.rs`, `src/admin.rs`, dashboard, docs)
- [x] P0.1 slice completed: hardened `make deploy-env-validate` to require non-empty/non-overbroad `SHUMA_ADMIN_IP_ALLOWLIST` and expanded deployment runbook checklist coverage for admin exposure, allowlist status, and login rate-limit posture.
- [x] P0.2 slice completed: added deploy-time edge-rate-limit attestation guard (`SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED=true`) so production deploys fail until `/admin/login` and `/admin/*` CDN/WAF rate limits are explicitly confirmed.
- [x] P0.3 slice completed: added optional read-only admin bearer key (`SHUMA_ADMIN_READONLY_API_KEY`), enforced write-access checks on mutating admin routes, hardened `/admin/unban` to POST-only, and logged denied write attempts for audit visibility.
- [x] P0.4 slice completed: added `gen-admin-api-key` alias + deploy-time API key rotation attestation guard (`SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED=true`) and documented a recommended 90-day rotation cadence in deployment/security runbooks.

## P1 Distributed State and Limiter Correctness
- [x] (Enterprise/hybrid track; non-blocking for `self_hosted_minimal`) Formalize profile-gated state-plane architecture: shared policy engine across personas, swappable state backends by profile (`self_hosted_minimal` vs `enterprise_akamai`), and no persona-specific policy fork.
- [x] P1.0 slice completed: documented the profile-gated state-plane architecture as ADR `docs/adr/0001-profile-gated-state-plane.md` and synchronized policy/deployment/config docs.
- [x] (Enterprise/hybrid track; non-blocking for `self_hosted_minimal`) Design and implement atomic distributed rate limiting (Redis `INCR`/Lua) for main traffic and admin auth, aligned with edge-state sync work. (`src/rate.rs`, `src/auth.rs`, `spin.toml`)
- [ ] (Enterprise/hybrid track; non-blocking for `self_hosted_minimal`) Define outage posture for distributed limiter (`fail-open` vs `fail-closed`) and add monitoring/alerts for limiter backend health. (architecture, ops, `docs/deployment.md`)
- [ ] (Enterprise/hybrid track; non-blocking for `self_hosted_minimal`) Design strategy for syncing bans/unbans across global edge instances. (architecture, ops)
- [ ] (Enterprise/hybrid track) Add monitoring for limiter fallback usage, sync lag, and distributed state drift when distributed/external state paths are enabled.
- [x] (Enterprise/hybrid track) Add deploy guardrails that block unsafe multi-instance enterprise rollouts when `rate_limiter`/`ban_store` remain local-only, with explicit override attestation for temporary advisory-only exceptions.
- [x] P1.1 slice completed: `make deploy-env-validate` now enforces multi-instance enterprise state posture (`SHUMA_ENTERPRISE_MULTI_INSTANCE`) and blocks authoritative local-only rate/ban state while requiring explicit advisory/off exception attestation (`SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED=true`) for temporary unsynced operation.
- [x] (Enterprise/hybrid track) Add startup/runtime warnings or hard-fail hooks for enterprise multi-instance local-only state posture, aligned with deploy guardrail semantics.
- [x] P1.2 slice completed: runtime config loading now enforces enterprise multi-instance state guardrails (hard-fail on unsafe posture) and `/admin/config` surfaces enterprise guardrail warnings/errors and attestation visibility fields.
- [x] P1.3 slice completed: replaced external `rate_limiter` stub with a Redis-backed distributed adapter (`INCR` + TTL window key), added explicit fallback-to-internal behavior, and enforced `SHUMA_RATE_LIMITER_REDIS_URL` guardrails for enterprise multi-instance posture.
- [x] P1.4 slice completed: replaced external `ban_store` stub with a Redis-backed distributed adapter (JSON ban entries + Redis TTL), routed admin ban/unban/list paths through provider selection, and enforced `SHUMA_BAN_STORE_REDIS_URL` guardrails for enterprise multi-instance posture.
- [x] P1.5 slice completed: routed admin auth failure throttling through the provider-selected rate limiter so external distributed rate-limiter mode covers admin auth (`/admin/login`, `/admin/logout`, unauthorized admin endpoints) with safe internal fallback when runtime config/provider selection is unavailable.

## P1 Staged Adaptive Defence (maze to slow-drip)

### Stage 1: Policy and signal prerequisites
- [ ] Add stable detection ID taxonomy and policy matching (`allow`, `monitor`, `challenge`, `maze`, `block`).
- [ ] Add static-resource bypass defaults to avoid expensive bot checks on obvious static assets.
- [ ] Add request-sequence signal primitives (operation IDs, ordering windows, timing thresholds).
- [ ] Add AI-bot policy controls as first-class admin config (separate from robots-only controls).

### Stage 2: Maze excellence execution (Cloudflare-inspired, Shuma-native)
- [ ] MZ-S1: Keep Stage 2 completion criteria internal-first (no external-provider dependency).
- [ ] MZ-S2: Execute Stage 2 delivery order as `MZ-1 -> MZ-2 -> MZ-5 -> MZ-3 -> MZ-4 -> MZ-7 -> MZ-8 -> MZ-9 -> MZ-10 -> MZ-6`.
- [ ] MZ-1: Replace path-only deterministic seeding with rotating signed entropy for suspicious traffic; keep short TTL deterministic windows for cacheability/debugging.
- [ ] MZ-2: Add signed traversal-link tokens with TTL, depth scope, branch budget, and replay protection.
- [ ] MZ-5: Make client-side expansion foundational for suspicious maze tiers (Web Worker branch generation + signed server verification) with explicit checkpoint cadence (every 3 nodes or 1500 ms), bounded step-ahead allowance, and no-JS fallback rules.
- [ ] MZ-3: Add polymorphic maze rendering (layout/content/link-graph variant families with versioned selection).
- [ ] MZ-3.1: Implement pluggable maze content-seed providers (internal default corpus + operator-provided source adapters).
- [ ] MZ-3.2: Add manual/scheduled seed refresh for provider-fed corpora with robots/compliance guardrails, caching, and rate limits.
- [ ] MZ-3.3: Enforce metadata/keyword-first extraction (avoid article-body copying) to reduce legal risk, bandwidth, and fingerprintability.
- [ ] MZ-4: Inject covert decoys into eligible non-maze HTML responses for medium-confidence suspicious traffic while preserving UX/SEO safety.
- [ ] MZ-7: Enforce maze cost budgets (global concurrency, per-bucket spend, response byte/time caps) with deterministic fallback behavior.
- [ ] MZ-8: Add a crawler simulation harness covering replay, deterministic fingerprinting attempts, JS/no-JS cohorts, and bypass attempts.
- [ ] MZ-9: Feed maze traversal behavior into botness scoring/detection IDs and add observability for entropy/token/proof/cost/budget signals.
- [ ] MZ-10: Roll out by phase (`instrument -> advisory -> enforce`) with explicit rollback triggers and operator runbook checks.
- [ ] MZ-6: Add optional adaptive micro-PoW for deeper traversal tiers.
- [ ] ~~Inject covert decoy links into eligible HTML responses for medium-confidence suspicious traffic.~~ Superseded by MZ-4.
- [ ] ~~Keep decoys invisible to normal users and compliant crawlers; avoid UX/SEO regressions.~~ Superseded by MZ-4 acceptance criteria.
- [ ] ~~Increase maze entropy (template diversity, fake static assets, path diversity) to reduce fingerprintability.~~ Superseded by MZ-1 and MZ-3.
- [ ] ~~Add pluggable maze content-seed providers (default static corpus + operator-supplied dynamic seeds) to reduce hard-coded vocabulary.~~ Superseded by MZ-3.1.
- [ ] ~~Add a manual/scheduled seed-refresh tool for operator-provided URLs/feeds (for example homepage headlines) with robots/compliance guardrails, caching, and rate limits.~~ Superseded by MZ-3.2.
- [ ] ~~Prefer metadata/keyword extraction over copying article bodies to minimize legal risk, bandwidth, and fingerprintability.~~ Superseded by MZ-3.3.
- [ ] ~~Feed maze interaction behavior back into botness scoring and detection IDs.~~ Superseded by MZ-9.

### Stage 3: Bounded slow-drip tarpit
- [ ] TP-C1: Reuse shared deception token primitives from maze scope (`MZ-2`) for tarpit progression; do not introduce a tarpit-only token format.
- [ ] TP-C2: Reuse shared budget/fallback primitives from maze scope (`MZ-7`) for tarpit limits and deterministic fallback; do not fork budget logic by mode.
- [ ] Implement `maze_plus_drip` mode with configurable byte rate and hard timeout using shared primitives.
- [ ] Enforce strict tarpit budgets (global concurrent streams and per-IP-bucket caps) via shared budget governor.
- [ ] Add deterministic fallback action when tarpit budget is exhausted (`maze` or `block`) via shared fallback matrix.
- [ ] Add tarpit metrics/admin visibility for activation, saturation, duration, bytes sent, and escalation outcomes.

### Stage 4: Escalation and distributed hardening
- [ ] Escalate persistent tarpit clients to ban/block with guardrails to minimize false positives.
- [ ] Integrate tarpit budgets/counters with distributed state work for multi-instance consistency.

## P1 Fingerprint Strengthening
- [ ] Add trusted-header ingestion for transport fingerprints supplied by CDN/proxy.
- [ ] Normalize fingerprint signals with provenance/confidence metadata for rule evaluation.
- [ ] Add mismatch heuristics (for example UA/client-hint versus transport fingerprint anomalies).
- [ ] Add fingerprint-centric admin visibility for investigations and tuning.
- [ ] Run a Finch compatibility spike as an optional upstream sidecar experiment and document tradeoffs for Shuma (no direct dependency in core runtime).

## P1 IP Range Policy Controls
- [ ] Add CIDR/IP-range policy evaluation to block, challenge, maze, or otherwise handle requests from configured ranges.
- [ ] Ship managed built-in CIDR sets for major AI service traffic (for example OpenAI, DeepSeek, GitHub Copilot) with explicit update/versioning process.
- [ ] Add operator-defined custom CIDR lists in config/admin with strict validation and clear precedence against managed sets.
- [ ] Extend response-action execution to support: `403_forbidden`, custom-message response, connection drop, `308` redirect (custom URL), `rate_limit`, `honeypot`, `maze`, and `tarpit`.
- [ ] Document operational guidance for IP-range policy safety (false-positive controls, dry-run/test mode, observability, rollback).

## P2 Challenge Roadmap
- [ ] Implement Challenge Lite (`/challenge/not-a-bot-checkbox`) per `todos/challenge-lite-spec.md` with signed short-lived single-use nonce and IP-bucket binding.
- [ ] Implement Challenge Lite telemetry capture/validation and scoring model (`0..10`) with server-side threshold routing (`pass`, `escalate_puzzle`, `maze_or_block`).
- [ ] Add Challenge Lite verification marker/token issuance after pass and enforce it in routing flow.
- [ ] Add Challenge Lite admin visibility/config controls for thresholds, TTL, and attempt caps (read-only defaults plus optional mutability controls).
- [ ] Add Challenge Lite metrics and dashboard exposure (`served`, `pass`, `escalate`, `fail`, `replay`, latency).
- [ ] Add unit/integration/e2e coverage for Challenge Lite lifecycle and replay/abuse paths.
- [ ] Add an accessibility-equivalent challenge modality with equivalent verification strength (expiry, single-use, signature, IP-bucket checks).
- [ ] Add post-success human-verification token issuance and enforcement for protected flows.
- [ ] Add challenge operational metrics for abandonment/latency (for example median solve time and incomplete challenge rate).

## P2 GEO Defence Maturity
- [ ] Add ASN/network dimensions in GEO policy logic (not just country list). (`src/signals/geo/mod.rs`, `src/config/mod.rs`, `src/admin/api.rs`)
- [ ] Add GEO/ASN observability and alerting (metrics, dashboard panels, docs). (`src/observability/metrics.rs`, dashboard, docs)

## P2 Modularization and Future Repository Boundaries
- [ ] Restructure source into clearer domain modules (policy engine, maze/tarpit, challenges, fingerprint signals, admin adapters).
- [ ] Extract policy decision flow from HTTP plumbing to enable isolated testing and future reuse.
- [ ] Define module interface contracts and dependency direction (core domain first, adapters second).
- [ ] Write objective criteria for future repo splits (API stability, release cadence, ownership, operational coupling).

## P2 Repository and Architecture Hardening (Structure + Pluggability)

### H1 Artifact and workspace hygiene
- [x] Keep all generated build artifacts out of `src/` (including WASM binaries) and move them to a dedicated artifacts path (for example `dist/wasm/`).
- [x] Update `spin.toml`, Makefile targets, and bootstrap scripts to consume the new artifacts path without changing runtime behavior.
- [x] Keep Playwright and test outputs ephemeral (`playwright-report/`, `test-results/`) and confirm ignore rules remain correct after any directory changes.
- [x] Add a short doc section describing expected generated directories and what should never be committed.

### H2 Test layout modernization (Rust idiomatic split)
- [x] Define and document project test conventions:
  unit tests colocated with module code,
  integration/behavior tests in `tests/`.
- [x] Create a shared test support module (request builders, env guards, common fixtures) to reduce duplication across current `src/*_tests.rs`.
- [x] Incrementally migrate top-level `src/*_tests.rs` files into colocated module tests and/or `tests/` integration suites (no behavior changes).
- [x] Keep test discovery and CI commands stable (`cargo test`, Make targets) throughout migration.
- [x] Add/adjust regression tests to ensure routing and enforcement order remain stable while tests move (runtime-backed early routes should be covered in integration-level tests, not native unit tests).
- [x] H2 slice A completed: moved ban/CDP/GEO/request-router/test-mode/whitelist test files to module-local paths and removed corresponding top-level test module wiring in `src/lib.rs`.
- [x] H2 slice A completed: added shared unit-test helpers in `src/test_support.rs` and adopted them in env-sensitive suites.
- [x] H2 slice A verification: `cargo test` passes after migration with no behavior changes.
- [x] H2 slice B completed: migrated config tests from `src/config_tests.rs` to module-local `src/config/tests.rs`, including shared env-lock adoption.
- [x] H2 slice C completed: migrated challenge tests from `src/challenge_tests.rs` to module-local `src/challenge/tests.rs`.
- [x] H2 slice B/C verification: `cargo test` passes with module-local config/challenge suites.
- [x] H2 slice D completed: migrated remaining crate-level test files (`risk`, `security`, `logging`) into structured `src/lib_tests/` modules and removed legacy top-level `src/*_tests.rs`.
- [x] H2 slice D verification: `cargo test` passes with stable test discovery after `src/lib_tests/` adoption.

### H3 Domain directory deepening (beyond first modularization pass)
- [x] Move orchestration helpers (`request_router`, `kv_gate`, `policy_pipeline`) into a cohesive runtime/policy directory with clear ownership boundaries.
- [x] Group admin/auth/config concerns into a cohesive adapter/domain boundary layout with minimal cross-module leakage.
- [x] Group signal and enforcement modules by domain (for example risk signals, enforcement actions, challenge/maze) and reduce root-level file sprawl.
- [x] Add thin compatibility re-exports during moves so refactors remain reviewable and low-risk.
- [x] Remove temporary compatibility shims once imports are fully migrated.
- [x] H3.1 slice completed: moved request orchestration modules into `src/runtime/` (`runtime/request_router.rs`, `runtime/kv_gate.rs`, `runtime/policy_pipeline.rs`) and rewired `src/lib.rs` call sites without behavior changes.
- [x] H3.2 slice completed: moved admin/auth into `src/admin/` (`admin/mod.rs`, `admin/api.rs`, `admin/auth.rs`) and moved config into `src/config/mod.rs`, then rewired module imports with no behavior change.
- [x] H3.3/H3.4 slice completed: regrouped signal modules under `src/signals/` and enforcement modules under `src/enforcement/`, then added crate-level compatibility re-exports in `src/lib.rs` to keep call sites stable during the move.
- [x] H3.5 slice completed: migrated remaining call sites to `src/signals/*` and `src/enforcement/*` paths and removed temporary compatibility re-exports from `src/lib.rs`.

### H3.6 Composable Defence + Signal Foundation (internal-first)
- [x] Define and document the defence taxonomy with an explicit inventory of `signal`, `barrier`, and `hybrid` modules (for example `rate` as hybrid); include ownership and dependency direction. (`docs/module-boundaries.md`, Defence Taxonomy section)
- [x] Introduce a canonical per-request signal contract (for example `BotSignal` + `SignalAccumulator`) that every signal/hybrid module writes to.
- [x] Add explicit signal availability semantics (`active`, `disabled`, `unavailable`) so botness logic never treats missing modules as silent zero.
- [x] Split hybrid modules into distinct paths:
  rate telemetry signal contribution for scoring,
  hard rate-limit enforcement barrier for immediate protection.
- [x] Add composability modes for eligible modules (`off`, `signal`, `enforce`, `both`) while keeping safety-critical controls non-disableable.
- [x] Define clear behavior for each mode in config/admin surfaces and runtime flow (including invalid combinations and defaults).
- [x] Refactor botness scoring to consume normalized accumulator output rather than direct module internals.
- [x] Lock explicit pre-launch default mode semantics and enforce via tests (`rate=both`, `geo=both`, `js=both`, with JS still gated by `js_required_enforced`).
- [x] Add unit and integration regression tests for mode matrix behavior and ordering invariants (especially hybrid modules and early-route interactions).
- [x] Add observability for mode and signal-state visibility (metrics/log fields indicating enabled/disabled/unavailable contributors).
- [x] Update docs (`configuration`, `features`, `observability`, `module-boundaries`) to explain composability semantics and tuning implications.
- [x] Keep implementations internal-only for now; defer external provider registry/factory work until signal contract and mode semantics stabilize.
- [x] H3.6.1 slice completed: added explicit defence taxonomy + inventory (`signal`, `barrier`, `hybrid`) with ownership and dependency direction in `docs/module-boundaries.md`.
- [x] H3.6.2 slice completed: introduced `BotSignal`/`SignalAccumulator` in `src/signals/botness.rs` and rewired JS, GEO, and rate-pressure botness scoring paths in `src/lib.rs` to emit normalized signal contributions with no behavior change.
- [x] H3.6.3 slice completed: added explicit signal availability states (`active`, `disabled`, `unavailable`) across JS/GEO/rate signal emitters and botness assessment flow, with regression tests for non-silent disabled/unavailable handling.
- [x] H3.6.4 slice completed: split rate hybrid paths into `src/signals/rate_pressure.rs` (telemetry + pressure scoring signals) and `src/enforcement/rate.rs` (hard limit enforcement path), then rewired runtime botness flow accordingly.
- [x] H3.6.5 slice completed: added per-module composability modes (`off`, `signal`, `enforce`, `both`) for JS/GEO/rate with runtime signal/action gating and admin-config validation, preserving default behavior as `both`.
- [x] H3.6.6 slice completed: defined explicit mode semantics in runtime/config/admin surfaces, added effective-mode + warning payloads (`defence_modes_effective`, `defence_mode_warnings`), and validated invalid mode key/value combinations.
- [x] H3.6.7 slice completed: introduced `BotnessSignalContext` and split botness into contribution collection + score finalization (`collect_botness_contributions`, `compute_botness_assessment_from_contributions`) so runtime policy consumes normalized contributions rather than direct scoring internals.
- [x] H3.6.8 slice completed: locked pre-launch default-mode semantics with explicit config tests and added mode-matrix regression coverage for JS/GEO/rate signal paths (including rate hybrid signal behavior), while retaining early-route ordering integration guards.
- [x] H3.6.9 slice completed: added botness signal-state and effective defence-mode observability (`botness_signal_state_total`, `defence_mode_effective_total`) plus richer botness log outcomes (`signal_states`, `modes`) for maze/challenge decisions.
- [x] H3.6.10 slice completed: updated composability/tuning/operator docs (`docs/configuration.md`, `docs/features.md`, `docs/observability.md`, `docs/module-boundaries.md`) with effective-mode semantics and observability guidance.
- [x] H3.6.11 slice completed: kept implementation internal-only (no provider registry/factory introduced) and explicitly deferred external-provider wiring to H4.

### H4 Pluggable provider architecture (internal by default, external-capable)
Implementation rule: when internal feature work touches provider-managed capabilities, route changes through provider interfaces and registry paths (no new direct orchestration-path module calls).
- [x] Define provider traits for swappable capabilities:
  rate limiting,
  ban store/sync,
  challenge engine,
  maze/tarpit serving,
  fingerprint signal source.
- [x] Add a provider registry/factory that selects implementations from config (compile-time/runtime config, no behavior change by default).
- [x] Implement `Internal*` providers matching current behavior as the default path.
- [x] Define and document provider externalization matrix by deployment persona:
  `self_hosted_minimal` (default),
  `enterprise_akamai` (target managed-edge integration),
  with advisory-by-default and authoritative-optional edge signal precedence.
- [x] Add explicit `External*` adapter stubs/contracts for high-leverage capabilities first:
  `fingerprint_signal`,
  `rate_limiter`,
  `ban_store`,
  `challenge_engine`,
  with explicit unsupported handling for `maze_tarpit` until a stable external API target exists.
- [x] Add contract tests that every provider implementation must pass to guarantee semantic parity and explicit unavailability behavior (`active`/`disabled`/`unavailable`) for external signal sources.
- [x] Add observability tags/metrics identifying active provider implementation per capability and edge integration mode (`off`/`advisory`/`authoritative`).
- [x] Document provider selection, rollout, and rollback procedures in deployment docs (including Akamai advisory/authoritative guidance and fallback-to-internal behavior).
- [x] H4.1 slice completed: formalized provider capability contracts in `src/providers/contracts.rs` (`RateLimiterProvider`, `BanStoreProvider`, `ChallengeEngineProvider`, `MazeTarpitProvider`, `FingerprintSignalProvider`) with stable enum labels and default-behavior regression tests.
- [x] H4.2 slice completed: added config-backed provider backend selection (`provider_backends` + `SHUMA_PROVIDER_*` defaults), plus `src/providers/registry.rs` factory/registry mapping (`internal`/`external`) with default internal selection and no behavior change to request handling paths.
- [x] H4.3 slice completed: implemented `Internal*` provider adapters in `src/providers/internal.rs` and routed core request/policy flow through registry-selected provider interfaces in `src/lib.rs` and `src/runtime/policy_pipeline.rs` (default behavior preserved under `internal` backends).
- [x] H4.4.1 slice completed: added `edge_integration_mode` posture (`off`/`advisory`/`authoritative`) to config/defaults and threaded it through runtime decision metadata plus metrics export (`bot_defence_edge_integration_mode_total`) without changing enforcement precedence.
- [x] H4.4.2 slice completed: added explicit `external` provider adapters in `src/providers/external.rs`; `fingerprint_signal` now routes to an external stub contract, while `challenge_engine` and `maze_tarpit` remain explicit unsupported adapter paths with safe fallback semantics.
- [x] H4.4.3 slice completed: added provider implementation observability with capability/backend/implementation metrics (`bot_defence_provider_implementation_effective_total`) and runtime event-tag provider summaries (`providers=...`) wired from registry-selected implementations.
- [x] H4.4.4 slice completed: added fingerprint provider contract availability semantics (`active`/`disabled`/`unavailable`) across internal/external adapters plus registry tests enforcing explicit unavailability behavior when external fingerprint is selected but not configured.
- [x] H4.4.5 slice completed: documented deployment personas plus provider selection matrix and added Akamai-focused advisory/authoritative rollout + rollback runbook with explicit fallback-to-internal procedure in `docs/configuration.md`, `docs/deployment.md`, and `docs/observability.md`.
- [ ] H4.5 plan follow-up (`docs/plans/2026-02-13-provider-externalization-design.md` step 3): replace external fingerprint stub with an Akamai-first adapter that maps edge/Bot Manager outcomes into normalized fingerprint signals.
- [x] H4.6 plan follow-up (`docs/plans/2026-02-13-provider-externalization-design.md` step 4): implemented external `rate_limiter` and `ban_store` adapters with distributed state/sync semantics and retired unsupported-stub behavior for those capabilities.
- [x] H4.6.1 slice completed: external `rate_limiter` now uses Redis-backed distributed counting (`INCR` + TTL) with explicit internal fallback and provider implementation labeling (`external_redis_with_internal_fallback`).
- [x] H4.6.2 slice completed: external `ban_store` now uses Redis-backed distributed ban state (JSON + TTL) with explicit internal fallback, provider implementation labeling (`external_redis_with_internal_fallback`), and admin ban/unban/list provider routing.
- [ ] H4.7 plan follow-up (`docs/plans/2026-02-13-provider-externalization-design.md` step 5): add integration tests for advisory vs authoritative mode precedence and explicit downgrade-to-internal behavior when external providers are unavailable.

### H5 Execution and rollout discipline
- [ ] Execute this hardening work as small, test-backed slices (one boundary family at a time) to avoid broad regressions.
- [ ] ~~Require each structural slice to pass full verification (`cargo test`, integration smoke, dashboard smoke where relevant) before merge.~~
- [ ] Require each structural slice to pass full verification via Makefile (`make test`; includes unit + integration + dashboard e2e) before merge.
- [ ] Track and enforce "no net behavior change" for refactor-only slices unless explicitly scoped otherwise.
- [x] Define a cutover checklist for enabling any external provider in non-dev environments (staging soak, SLOs, rollback trigger).

## P3 Platform and Configuration Clarity
- [ ] Design runtime-agnostic architecture that keeps core detection logic portable while preserving Fermyon-first performance paths.
- [x] Define platform scope boundaries to avoid overreach by leaning on upstream bot managers (for example Akamai) for features better handled there.
- [x] Add non-secret runtime config export for deploy handoff (exclude secrets) so dashboard-tuned settings can be applied in immutable redeploys.
- [ ] Evaluate renaming `SHUMA_CHALLENGE_RISK_THRESHOLD` to `SHUMA_BOTNESS_CHALLENGE_THRESHOLD` to reflect botness semantics.
- [ ] Standardize terminology across code/UI/docs so `honeypot` and `maze` are used consistently instead of interchangeably.
- [ ] Initialize Ban IP pane duration controls from the current Admin Manual Ban default duration so Ban IP and Ban Durations panes stay consistent.
- [ ] Review dashboard implementation and evaluate migration to an ultra-light frontend framework (for example Lit) with explicit tradeoffs for bundle size, maintainability, DX, and migration cost.
- [ ] Document setup-time config bootstrapping clearly: how `make setup` creates/populates local env, how env-only vars are sourced, and how KV defaults are seeded and later overridden.
- [ ] Long-term option: integrate upstream identity/proxy auth (OIDC/SAML) for dashboard/admin instead of app-level key login.
- [x] P3.1 slice completed: documented Akamai-vs-Shuma platform scope ownership boundaries, non-goals, and decision rules in `docs/bot-defence.md` to keep edge-vs-app responsibilities explicit.

## Recurring Quality Gates
- [ ] Keep unit, integration, e2e, and CI flows passing; clean up defunct tests quickly.
- [ ] Identify and prioritize missing tests for new defence stages before implementation.
- [ ] Reassess data retention policy as event and metrics volume grows.
