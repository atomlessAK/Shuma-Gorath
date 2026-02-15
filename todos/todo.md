# TODO Roadmap

Last updated: 2026-02-15

This is the active work queue.
`todos/security-review.md` tracks security finding validity and closure status.
Completed items are archived in `todos/completed-todo-history.md`.

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
- [ ] R-RL-03 Review Veroff et al., "Evaluation of a low-rate DoS attack against application servers" (Computers & Security 2008) and capture queue/resource-starvation mitigation patterns. https://doi.org/10.1016/j.cose.2008.07.004
- [ ] R-RL-05 Review Srivatsa et al., "Mitigating application-level denial of service attacks on Web servers" (ACM TWEB 2008) and assess admission/congestion control patterns for Shuma policy pipeline. https://research.ibm.com/publications/mitigating-application-level-denial-of-service-attacks-on-web-servers-a-client-transparent-approach
- [ ] R-RL-06 Review Lemon, "Resisting SYN flood DoS attacks with a SYN cache" (BSDCon 2002) and capture edge-vs-origin queue protection lessons relevant to Akamai authoritative mode. https://www.usenix.org/legacy/publications/library/proceedings/bsdcon02/full_papers/lemon/lemon_html/index.html
- [ ] R-RL-07 Review Chen et al., "SMARTCOOKIE" (USENIX Security 2024) and evaluate split-proxy edge-cookie architecture fit for enterprise Akamai deployments. https://collaborate.princeton.edu/en/publications/smartcookie-blocking-large-scale-syn-floods-with-a-split-proxy-de/

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
- [ ] R-SSH-02 Review Bythwood et al., "Fingerprinting Bots in a Hybrid Honeypot" (IEEE SoutheastCon 2023) and assess hybrid interaction design implications for SSH deception tiers. https://doi.org/10.1109/SoutheastCon51012.2023.10115143
- [ ] R-SSH-03 Review Vetterl et al., "A Comparison of an Adaptive Self-Guarded Honeypot with Conventional Honeypots" (Applied Sciences 2022) and evaluate adaptive risk-vs-observability controls for Shuma SSH tarpit mode. https://doi.org/10.3390/app12105224
- [ ] R-SSH-04 Review Cordeiro/Vasilomanolakis, "Towards agnostic OT honeypot fingerprinting" (TMA 2025) and extract transport-stack realism requirements applicable to SSH tarpit surfaces. https://doi.org/10.23919/TMA66427.2025.11097018

## Modularization Sprint (Active)
## P0 Immediate (ops and abuse containment)
## P1 Distributed State and Limiter Correctness
- [ ] (Enterprise/hybrid track; non-blocking for `self_hosted_minimal`) Design strategy for syncing bans/unbans across global edge instances. (architecture, ops)
- [ ] (Enterprise/hybrid track) Extend distributed-state monitoring with ban sync-lag metrics (rate-limiter fallback/drift monitoring is implemented).
## P1 Staged Adaptive Defence (maze to slow-drip)

### Stage 1: Policy and signal prerequisites
- Completed items archived to `todos/completed-todo-history.md` on 2026-02-15.

### Stage 2: Maze excellence execution (Cloudflare-inspired, Shuma-native)
- Completed items archived to `todos/completed-todo-history.md` on 2026-02-15.
- [ ] ~~Inject covert decoy links into eligible HTML responses for medium-confidence suspicious traffic.~~ Superseded by MZ-4.
- [ ] ~~Keep decoys invisible to normal users and compliant crawlers; avoid UX/SEO regressions.~~ Superseded by MZ-4 acceptance criteria.
- [ ] ~~Increase maze entropy (template diversity, fake static assets, path diversity) to reduce fingerprintability.~~ Superseded by MZ-1 and MZ-3.
- [ ] ~~Add pluggable maze content-seed providers (default static corpus + operator-supplied dynamic seeds) to reduce hard-coded vocabulary.~~ Superseded by MZ-3.1.
- [ ] ~~Add a manual/scheduled seed-refresh tool for operator-provided URLs/feeds (for example homepage headlines) with robots/compliance guardrails, caching, and rate limits.~~ Superseded by MZ-3.2.
- [ ] ~~Prefer metadata/keyword extraction over copying article bodies to minimize legal risk, bandwidth, and fingerprintability.~~ Superseded by MZ-3.3.
- [ ] ~~Feed maze interaction behavior back into botness scoring and detection IDs.~~ Superseded by MZ-9.

### Stage 2 follow-up: Operator-safe Maze Preview
- Completed items archived to `todos/completed-todo-history.md` on 2026-02-15.

### Stage 2.5 follow-up: Maze excellence shortfall closure (research-first)
- Completed items archived to `todos/completed-todo-history.md` on 2026-02-15.

### Stage 2.6 follow-up: Maze test coverage closure
- [ ] MZ-T1 Add Spin integration coverage for live `/maze/*` traversal across multiple hops: entry -> tokenized link follow -> checkpoint submit -> `/maze/issue-links` progression -> fallback/escalation branches, with assertions for deterministic fallback action/reason semantics.
- [ ] MZ-T2 Add browser E2E coverage for live maze behavior (not just dashboard config): JS-enabled and JS-disabled cohorts, checkpoint/micro-PoW flow, replay rejection, and high-confidence escalation outcomes under real HTTP/session behavior.
- [ ] MZ-T3 Add concurrency/soak coverage for maze state/budget primitives (replay keys, checkpoint keys, global/per-bucket budget caps) to detect contention/regression under burst traversal and verify bounded host-write behavior.
- [ ] MZ-T4 Wire the new maze integration + E2E + soak tests into canonical Makefile/CI verification paths (`make test`, focused rerun targets, and CI failure gates) so maze behavior regressions fail fast before merge.

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
### H2 Test layout modernization (Rust idiomatic split)
### H3 Domain directory deepening (beyond first modularization pass)
### H3.6 Composable Defence + Signal Foundation (internal-first)
### H4 Pluggable provider architecture (internal by default, external-capable)
Implementation rule: when internal feature work touches provider-managed capabilities, route changes through provider interfaces and registry paths (no new direct orchestration-path module calls).
### H5 Execution and rollout discipline
- [ ] Execute this hardening work as small, test-backed slices (one boundary family at a time) to avoid broad regressions.
- [ ] ~~Require each structural slice to pass full verification (`cargo test`, integration smoke, dashboard smoke where relevant) before merge.~~
- [ ] Require each structural slice to pass full verification via Makefile (`make test`; includes unit + integration + dashboard e2e) before merge.
- [ ] Track and enforce "no net behavior change" for refactor-only slices unless explicitly scoped otherwise.
## P3 Platform and Configuration Clarity
- [ ] Design runtime-agnostic architecture that keeps core detection logic portable while preserving Fermyon-first performance paths.
- [ ] Evaluate renaming `SHUMA_CHALLENGE_RISK_THRESHOLD` to `SHUMA_BOTNESS_CHALLENGE_THRESHOLD` to reflect botness semantics.
- [ ] Standardize terminology across code/UI/docs so `honeypot` and `maze` are used consistently instead of interchangeably.
- [ ] Initialize Ban IP pane duration controls from the current Admin Manual Ban default duration so Ban IP and Ban Durations panes stay consistent.
- [ ] Decomposed into `DSH-*` dashboard modernization items below (frameworkless-first with explicit Lit decision gate).
- [ ] Document setup-time config bootstrapping clearly: how `make setup` creates/populates local env, how env-only vars are sourced, and how KV defaults are seeded and later overridden.
- [ ] Long-term option: integrate upstream identity/proxy auth (OIDC/SAML) for dashboard/admin instead of app-level key login.

### P3 Dashboard Architecture Modernization (Tabbed SPA, Frameworkless-First)

#### Baseline and decision gate
- [x] DSH-R1 Baseline current dashboard architecture and runtime costs (JS/CSS bytes, startup time, memory, polling cadence, bundle provenance, current e2e coverage) and publish a short decision memo in `docs/plans/`. (`docs/plans/2026-02-15-dashboard-architecture-modernization.md`)
- [x] DSH-R2 Evaluate two implementation tracks against Shuma constraints: (A) frameworkless modular SPA + JSDoc typing, (B) ultra-light framework (Lit) with equivalent tab shell; include explicit tradeoffs for maintenance, DX, runtime weight, and migration risk. (`docs/plans/2026-02-15-dashboard-architecture-modernization.md`)
- [x] DSH-R3 Define framework-adoption gate criteria (for example: unresolved lifecycle complexity, repeated DOM/state bugs, unacceptable change lead time after frameworkless refactor); default to no framework unless gate is tripped. (`docs/plans/2026-02-15-dashboard-architecture-modernization.md`)

#### Tabbed SPA shell and structure (frameworkless path)
- [x] DSH-1 Implement tabbed SPA shell in `dashboard/index.html` + `dashboard/dashboard.js` with canonical tabs: `Monitoring`, `IP Bans`, `Status`, `Config`, `Tuning`.
- [x] DSH-2 Add URL-backed tab routing (`#monitoring`, `#ip-bans`, `#status`, `#config`, `#tuning`) with refresh-safe deep links and history navigation.
- [x] DSH-3 Refactor monolithic dashboard orchestration into tab-scoped controllers/modules with clear lifecycle (`init`, `mount`, `unmount`, `refresh`) and no cross-tab hidden coupling.
- [ ] DSH-4 Introduce a shared dashboard API client layer (typed request/response adapters, centralized error handling, auth/session helpers) to eliminate ad-hoc per-tab fetch patterns.
- [ ] DSH-5 Introduce shared state primitives (single source of truth for session/config/status snapshots, explicit invalidation rules, and tab-local derived state) without adding runtime framework dependencies.

#### Safety, reliability, and UX hardening
- [ ] DSH-6 Replace CDN-loaded Chart.js with a pinned local asset strategy under `dashboard/assets/` (versioned artifact + integrity/provenance note in docs) to reduce supply-chain/runtime variability.
- [ ] DSH-7 Scope polling/refresh loops by active tab visibility (for example Monitoring-only chart/event polling), with deterministic suspend/resume behavior and bounded timer count.
- [ ] DSH-8 Add accessibility and keyboard-navigation guarantees for tabs (ARIA roles, focus management, visible selected state, screen-reader labels) and document expected behavior.
- [ ] DSH-9 Add progressive type safety without runtime cost (`// @ts-check` + JSDoc typedefs across dashboard modules; optional follow-up TypeScript compile step only if needed).
- [ ] DSH-10 Add robust empty/error/loading states per tab to remove implicit DOM assumptions and reduce silent UI failure modes.

#### Verification, docs, and rollout
- [ ] DSH-11 Expand dashboard e2e coverage for tabbed SPA flows (route persistence, tab switching, Monitoring charts/events, IP ban actions, Config/Tuning saves, error-state handling, and session continuity).
- [ ] DSH-12 Add focused unit-style tests for shared dashboard utilities/modules where practical (state reducers/selectors, response adapters, tab router helpers).
- [ ] DSH-13 Update public docs (`docs/dashboard.md`, `README.md`, `docs/testing.md`) with the new tab model, routing behavior, and contributor workflow for dashboard changes.
- [ ] DSH-14 Add migration/rollback notes for operators and contributors (how to disable/roll back tab shell safely if regressions appear).

#### Conditional Lit follow-up (only if DSH-R3 gate trips)
- [ ] DSH-G1 If frameworkless acceptance criteria are not met, run a constrained Lit pilot on one tab (recommended: `Monitoring`) and compare measured bundle/runtime/maintainability deltas before any full migration decision.
## Recurring Quality Gates
- [ ] Keep unit, integration, e2e, and CI flows passing; clean up defunct tests quickly.
- [ ] Identify and prioritize missing tests for new defence stages before implementation.
- [ ] Reassess data retention policy as event and metrics volume grows.
