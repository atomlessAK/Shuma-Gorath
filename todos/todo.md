# TODO Roadmap

Last updated: 2026-02-18

This is the active work queue.
`todos/security-review.md` tracks security finding validity and closure status.
Completed items are archived in `todos/completed-todo-history.md`.

## Direction Snapshot (for next implementation stages)
- [ ] Follow internal-first delivery policy: harden Shuma-native capability paths before completing external-provider parity for the same capability; use enterprise/Akamai patterns to inform design, not as baseline dependencies.

## P0 Priority Override (Highest Priority Queue)
- [x] Complete the remaining SvelteKit migration work (`DSH-SVLT-NEXT1.*`, `DSH-SVLT-NEXT2.*`, `DSH-SVLT-NEXT3.*`, `DSH-SVLT-TEST1.*`, `DSH-SVLT-TEST2.*`) before non-critical roadmap work.
- [x] Treat all non-blocking research/backlog items below as lower priority until the Svelte-native dashboard path replaces the bridge path.

## P1 Research Dossiers (Paper-by-Paper TODOs)
Completion rule for every paper TODO below: capture key findings, map to `self_hosted_minimal` vs `enterprise_akamai` ownership, and propose concrete Shuma TODO updates.

### Fingerprinting, JS Verification, and CDP-Adjacent Detection
- [x] Strengthen fingerprinting by hardening internal baseline signals first, then ingesting trusted upstream edge signals (JA3/JA4 and similar) with provenance checks and explicit internal fallback when edge headers are absent or untrusted.
- Completed research tranche (`R-FP-01`..`R-FP-09`) archived in `docs/research/2026-02-16-fingerprinting-research-synthesis.md` and `todos/completed-todo-history.md`.

#### Phase 1: Foundation and guardrails
- [x] Normalize fingerprint signals with provenance/confidence metadata for rule evaluation.
- [x] FP-R11 Add feature-family entropy budgeting and per-family confidence caps (avoid over-weighting high-cardinality unstable attributes).
- [x] FP-R20 Add fingerprint data-minimization and retention controls (TTL/pseudonymization/export visibility) plus operator documentation.

#### Phase 2: Core internal detection quality
- [x] FP-R15 Expand cross-layer inconsistency rules: UA, client hints, runtime/browser APIs, and transport-level fingerprints.
- [x] Add mismatch heuristics (for example UA/client-hint versus transport fingerprint anomalies).
- [x] FP-R12 Add temporal coherence modeling with per-attribute churn classes and impossible-transition detection IDs.
- [x] FP-R16 Add flow-centric fingerprint telemetry extraction and bounded per-flow aggregation windows.

#### Phase 3: Hardening and controlled signal expansion
- [x] FP-R13 Add JS/CDP detector-surface rotation support (versioned probe families + staged rollout + rollback controls).
- [x] Add trusted-header ingestion for transport fingerprints supplied by CDN/proxy.
- [x] FP-R14 Add multi-store persistence-abuse signals (cookie/localStorage/sessionStorage/IndexedDB recovery patterns) as suspicious automation features.
- [x] FP-R17 Add optional challenge-bound, short-lived device-class marker path (Picasso-inspired) for replay-resistant continuity checks.
- [x] FP-R18 Add optional low-friction behavioral micro-signals in challenge contexts (mouse/timing), with privacy guardrails and conservative weighting.

#### Phase 4: Operatorization and validation
- [x] Add fingerprint-centric admin visibility for investigations and tuning.
- [x] FP-R19 Add evasive-regression coverage for detector fingerprinting, temporal drift, and inconsistency-bypass attempts.
- [ ] Run a Finch comparison spike to see if Shuma might benefit from enabling enhancing its internal capabilities with allowing users to integrate finch alongside it(no direct dependency in core runtime).

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
- [ ] OUT-1 Add explicit deployment guardrails that fail when `provider_backends.rate_limiter=external` or `provider_backends.ban_store=external` but required Redis outbound hosts are not allowlisted in `spin.toml` `allowed_outbound_hosts`.
- [ ] OUT-2 Add a provider-to-outbound-requirements matrix in public docs (internal vs external backend, required host capabilities, required outbound host allowlists, fallback behavior).
- [ ] OUT-3 Add integration verification that exercises external Redis provider selection under restricted outbound policy and confirms safe fallback/guardrail behavior is deterministic.
- [ ] OUT-5 Before implementing non-stub `challenge_engine=external` and `maze_tarpit=external`, complete design work for their external transport path through Spin host capabilities or sidecar/adapter boundary, with rollback and security posture defined.
- [ ] Build Shuma-native bounded slow-drip tarpit behavior in Rust/Spin; treat external projects (for example Finch/Sarracenia/caddy-defender) as design references, not runtime dependencies.
- [ ] TP-C1: Reuse shared deception token primitives from maze scope (`MZ-2`) for tarpit progression; do not introduce a tarpit-only token format.
- [ ] TP-C2: Reuse shared budget/fallback primitives from maze scope (`MZ-7`) for tarpit limits and deterministic fallback; do not fork budget logic by mode.
- [ ] Implement `maze_plus_drip` mode with configurable byte rate and hard timeout using shared primitives.
- [ ] Enforce strict tarpit budgets (global concurrent streams and per-IP-bucket caps) via shared budget governor.
- [ ] Add deterministic fallback action when tarpit budget is exhausted (`maze` or `block`) via shared fallback matrix.
- [ ] Add tarpit metrics/admin visibility for activation, saturation, duration, bytes sent, and escalation outcomes.
- [ ] Escalate persistent tarpit clients to ban/block with guardrails to minimize false positives.
- [ ] Integrate tarpit budgets/counters with distributed state work for multi-instance consistency.
- [ ] (Enterprise/hybrid track) Extend distributed-state monitoring with ban sync-lag metrics (rate-limiter fallback/drift monitoring is implemented).

#### SSH Tarpit and Honeypot Evasion Resistance
- [ ] R-SSH-02 Review Bythwood et al., "Fingerprinting Bots in a Hybrid Honeypot" (IEEE SoutheastCon 2023) and assess hybrid interaction design implications for SSH deception tiers. https://doi.org/10.1109/SoutheastCon51012.2023.10115143
- [ ] R-SSH-03 Review Vetterl et al., "A Comparison of an Adaptive Self-Guarded Honeypot with Conventional Honeypots" (Applied Sciences 2022) and evaluate adaptive risk-vs-observability controls for Shuma SSH tarpit mode. https://doi.org/10.3390/app12105224
- [ ] R-SSH-04 Review Cordeiro/Vasilomanolakis, "Towards agnostic OT honeypot fingerprinting" (TMA 2025) and extract transport-stack realism requirements applicable to SSH tarpit surfaces. https://doi.org/10.23919/TMA66427.2025.11097018

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

## P1 Distributed State and Limiter Correctness
- [ ] (Enterprise/hybrid track; non-blocking for `self_hosted_minimal`) Design strategy for syncing bans/unbans across global edge instances. (architecture, ops)

### P1 Outbound Capability and External Provider Constraints
- [ ] OUT-4 Create an ADR for non-Redis external integrations (for example webhook notifications or cross-service sync) that defines the approved pattern in Spin (`allowed_outbound_hosts` expansion vs sidecar/bridge service).


### Stage 2.6 follow-up: Maze test coverage closure
- [ ] MZ-T1 Add Spin integration coverage for live opaque maze traversal across multiple hops: entry -> tokenized link follow -> checkpoint submit -> `<maze_path_prefix>issue-links` progression -> fallback/escalation branches, with assertions for deterministic fallback action/reason semantics.
- [ ] MZ-T2 Add browser E2E coverage for live maze behavior (not just dashboard config): JS-enabled and JS-disabled cohorts, checkpoint/micro-PoW flow, replay rejection, and high-confidence escalation outcomes under real HTTP/session behavior.
- [ ] MZ-T3 Add concurrency/soak coverage for maze state/budget primitives (replay keys, checkpoint keys, global/per-bucket budget caps) to detect contention/regression under burst traversal and verify bounded host-write behavior.
- [ ] MZ-T4 Wire the new maze integration + E2E + soak tests into canonical Makefile/CI verification paths (`make test`, focused rerun targets, and CI failure gates) so maze behavior regressions fail fast before merge.

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


### H4 Pluggable provider architecture (internal by default, external-capable)
Implementation rule: when internal feature work touches provider-managed capabilities, route changes through provider interfaces and registry paths (no new direct orchestration-path module calls).
### H5 Execution and rollout discipline
- [ ] Execute this hardening work as small, test-backed slices (one boundary family at a time) to avoid broad regressions.
- [ ] Require each structural slice to pass full verification via Makefile (`make test`; includes unit + integration + dashboard e2e) before merge.
- [ ] Track and enforce "no net behavior change" for refactor-only slices unless explicitly scoped otherwise.
## P3 Platform and Configuration Clarity
- [ ] Design runtime-agnostic architecture that keeps core detection logic portable while preserving Fermyon-first performance paths.
- [ ] Evaluate renaming `SHUMA_CHALLENGE_PUZZLE_RISK_THRESHOLD` to `SHUMA_BOTNESS_CHALLENGE_PUZZLE_THRESHOLD` to reflect botness semantics.
- [ ] Standardize terminology across code/UI/docs so `honeypot` and `maze` are used consistently instead of interchangeably.
- [ ] Initialize Ban IP pane duration controls from the current Admin Manual Ban default duration so Ban IP and Ban Durations panes stay consistent.
- [x] Dashboard modernization now follows SvelteKit full cutover (`DSH-SVLT-*`) with static adapter output served via Spin (`dist/dashboard`), superseding the prior framework migration direction.
- [ ] Document setup-time config bootstrapping clearly: how `make setup` creates/populates local env, how env-only vars are sourced, and how KV defaults are seeded and later overridden.
- [ ] Long-term option: integrate upstream identity/proxy auth (OIDC/SAML) for dashboard/admin instead of app-level key login.

### P3 Monitoring Signal Expansion (Dashboard + Telemetry)
- [x] DSH-MON-1 Add a `Honeypot Hits` monitoring section (mirroring maze summary style) with: total hits, unique crawler buckets, top crawler buckets, and top honeypot paths hit.
- [x] DSH-MON-2 Add a `Challenge Failures` monitoring section with time-windowed totals and reason breakdown (`incorrect`, `expired/replay`, `sequence_violation`, `invalid_output`, `forbidden`), plus trend chart.
- [x] DSH-MON-3 Add a `PoW Failures` monitoring section with time-windowed totals and reason breakdown (`invalid_proof`, `missing_seed/nonce`, `sequence_violation`, `expired/replay`, `binding/timing mismatch`), plus trend chart.
- [x] DSH-MON-4 Add a `Rate Limiting Violations` monitoring section with totals, unique offender buckets, top offender buckets, and enforcement outcomes (`limited`, `banned`, `fallback_allow`, `fallback_deny`).
- [x] DSH-MON-5 Add a `GEO Violations` monitoring section with totals by route/action (`block`, `challenge`, `maze`) and top country codes causing policy actions.
- [x] DSH-MON-6 Add a Monitoring-page helper panel that explains how to export/scrape the same signals in Prometheus format (`/metrics`) for external visualization platforms (for example Prometheus/Grafana), including copyable scrape examples.
- [ ] DSH-MON-7 Deliberate Prometheus parity scope for Monitoring: audit each Monitoring widget/signal as `already_exported`, `derivable_from_existing_series`, or `missing_export`; then define a prioritized add-list with cardinality/cost guardrails before implementing new metric series.
- [x] MON-TEL-1 Add structured honeypot hit telemetry (KV/metric counters by IP bucket and path key) so dashboard can report path-level honeypot activity without relying on free-form event text parsing.
- [x] MON-TEL-2 Add challenge-submit failure telemetry with explicit counters and optional event records for failure classes that currently only increment coarse counters (enable top-offender and reason panels).
- [ ] MON-TEL-3 Add explicit PoW verify outcome telemetry (success + failure classes) since invalid-proof and malformed-request paths are not currently surfaced as dashboard-ready counters/events.
- [ ] MON-TEL-3.a Add PoW verify success-class telemetry and decide whether Monitoring should expose success/fail ratio or keep failures-only.
- [ ] MON-TEL-4 Add rate-limit violation summary endpoint (or equivalent aggregation contract) that returns filtered offender/top-path/top-window data without requiring expensive client-side filtering over generic event feeds.
- [x] MON-TEL-5 Add GEO enforcement telemetry keyed by action + country (bounded cardinality, ISO country normalization) so GEO monitoring panels are robust and not dependent on outcome-string parsing.
- [x] MON-TEL-6 Add admin API surface for these monitoring summaries (`/admin/honeypot`, `/admin/challenge`, `/admin/pow`, `/admin/rate`, `/admin/geo` or consolidated endpoint) with strict response schema + docs.
- [ ] MON-TEL-7 Add tests for telemetry correctness and dashboard rendering states (empty/loading/error/data) for each new monitoring section, including cardinality guardrails and retention-window behavior.
- [ ] MON-TEL-7.a Extend dashboard automated tests to assert new monitoring cards/tables/charts across empty/loading/error/data states, not just adapter contracts.

### P1 Dashboard SvelteKit Excellence Round 5 (State Convergence + Functionalization)
- [ ] DSH-SVLT-EX22 Codify and enforce the pre-launch policy stance: no backward DOM-ID compatibility layer, no multi-instance runtime guarantees for now, and prioritize behavior/outcome contracts over legacy structural test contracts.
- [ ] DSH-SVLT-EX23 Break up `dashboard/src/lib/runtime/dashboard-native-runtime.js` into focused runtime modules (session, refresh, config wiring, DOM binding lifecycle) and reduce coordinator hotspot size materially.
- [ ] DSH-SVLT-EX24 Converge on one dashboard state source of truth by removing duplicate runtime snapshot/session/status state paths and routing tab/session/snapshot updates through a single store contract.
- [ ] DSH-SVLT-EX25 Remove dead/unsafe native runtime event-controller leftovers (including undeclared `dashboardEventAbortController` helpers) and add regression guardrails preventing undeclared runtime globals.
- [ ] DSH-SVLT-EX26 Move primary Monitoring rendering from imperative ID-driven DOM mutation/string HTML paths to Svelte reactive component state + declarative templates.
- [ ] DSH-SVLT-EX27 Replace Ban table full rebuild + per-refresh rebinding with stable row patching and delegated action handling to reduce DOM/listener churn.
- [ ] DSH-SVLT-EX28 Refactor chart orchestration to instance-scoped runtime services owned by mount lifecycle (no module-level chart singletons), while retaining the shared chart runtime loader adapter.
- [ ] DSH-SVLT-EX29 Standardize dashboard static asset resolution on SvelteKit base-aware paths and remove hard-coded absolute asset references from route/component templates.
- [ ] DSH-SVLT-EX30 Remove superseded/unused dashboard controller abstractions (for example unused feature-controller wrapper paths) and add dead-code guard checks to module tests.
- [ ] DSH-SVLT-EX31 Add architecture/perf gates for the refactor: coordinator LOC budget, duplicate-state path regression checks, and remount/listener leak checks across decomposed runtime modules.
- [ ] DSH-SVLT-EX32 Publish an ADR that locks the current dashboard runtime policy (single-instance pre-launch, no backward DOM-ID compatibility shims, no bridge flag matrix) and align implementation/tests to that scope.

## Recurring Quality Gates
- [ ] Keep unit, integration, e2e, and CI flows passing; clean up defunct tests quickly.
- [ ] Identify and prioritize missing tests for new defence stages before implementation.
- [ ] Reassess data retention policy as event and metrics volume grows.
