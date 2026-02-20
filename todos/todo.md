# TODO Roadmap

Last updated: 2026-02-19

This is the active work queue.
`todos/security-review.md` tracks security finding validity and closure status.
Completed items are archived in `todos/completed-todo-history.md`.

## P1 Privacy and Data-Protection Follow-up
- [ ] SEC-GDPR-2 Enforce deterministic cleanup/expiry for stale fingerprint state keys (`fp:state:*`, `fp:flow:*`, `fp:flow:last_bucket:*`) aligned to configured fingerprint TTL/window controls.
- [ ] SEC-GDPR-3 Add an optional event-log IP minimization mode (raw vs masked/pseudonymized) for privacy-sensitive deployments, with explicit tradeoff documentation.
- [ ] SEC-GDPR-4 Add a deployer-ready privacy/cookie disclosure template in docs (lawful basis, retention table, storage inventory, and rights-handling workflow).

## P1 Research Dossiers (Paper-by-Paper TODOs)
Completion rule for every paper TODO below: capture key findings, map to `self_hosted_minimal` vs `enterprise_akamai` ownership, and propose concrete Shuma TODO updates.

### Fingerprinting, JS Verification, and CDP-Adjacent Detection
- Completed research tranche (`R-FP-01`..`R-FP-09`) archived in `docs/research/2026-02-16-fingerprinting-research-synthesis.md` and `todos/completed-todo-history.md`.
- [ ] Run a Finch comparison spike to see if Shuma might benefit from enabling enhancing its internal capabilities with allowing users to integrate finch alongside it(no direct dependency in core runtime).

### Challenges: PoW, Not-a-Bot, and Puzzle Escalation
- [ ] R-CH-01 Review Dwork/Naor, "Pricing via Processing or Combatting Junk Mail" (CRYPTO 1992) and extract adaptive requester-cost principles for modern web bot defence. https://www.microsoft.com/en-us/research/publication/pricing-via-processing-or-combatting-junk-mail/
- [ ] R-CH-02 Review Juels/Brainard, "Client Puzzles" (NDSS 1999) and define stateless verification patterns for Shuma PoW endpoints. https://www.ndss-symposium.org/ndss1999/cryptographic-defense-against-connection-depletion-attacks/
- [ ] R-CH-03 Review Adam Back, "Hashcash: A Denial of Service Counter-Measure" (2002) and assess modern browser-side PoW cost tuning constraints. https://nakamotoinstitute.org/library/hashcash/
- [ ] R-CH-04 Review von Ahn et al., "CAPTCHA: Using Hard AI Problems for Security" (EUROCRYPT 2003) and capture challenge-design principles still valid for the Not-a-Bot checkbox step. https://doi.org/10.1007/3-540-39200-9_18
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
- [x] R-IP-01 Review Ramanathan et al., "BLAG: Improving the Accuracy of Blacklists" (NDSS 2020) and derive feed-aggregation + false-positive controls for managed CIDR sets. https://doi.org/10.14722/ndss.2020.24232
- [x] R-IP-02 Review Sheng et al., "An Empirical Analysis of Phishing Blacklists" (2009) and extract freshness/latency requirements for update cadence and rollout safety. https://kilthub.cmu.edu/articles/journal_contribution/An_Empirical_Analysis_of_Phishing_Blacklists/6469805
- [x] R-IP-03 Review Oest et al., "PhishTime" (USENIX Security 2020) and map continuous quality-measurement methodology to Shuma feed validation. https://www.usenix.org/conference/usenixsecurity20/presentation/oest-phishtime
- [x] R-IP-04 Review Li et al., "HADES Attack" (NDSS 2025) and define anti-poisoning controls for any external blocklist ingestion pipeline. https://doi.org/10.14722/ndss.2025.242156
- [x] R-IP-05 Review Deri/Fusco, "Evaluating IP Blacklists effectiveness" (FiCloud 2023) and identify practical precision/recall limits for aggressive edge enforcement. https://research.ibm.com/publications/evaluating-ip-blacklists-effectiveness
- Research synthesis recorded in `docs/research/2026-02-20-ip-range-policy-research-synthesis.md` (includes source mapping and implementation implications).
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

## P2 Challenge Roadmap
- [x] NAB-0 Research and policy synthesis: keep `docs/research/2026-02-19-not-a-bot-challenge-research-synthesis.md`, `docs/plans/2026-02-13-not-a-bot-excellence-plan.md`, and `todos/not-a-bot-spec.md` aligned as the single implementation source.
- [x] NAB-1 Implement Not-a-Bot checkbox (`/challenge/not-a-bot-checkbox`) per `todos/not-a-bot-spec.md` with signed short-lived single-use nonce and IP-bucket binding.
- [x] NAB-2 Implement Not-a-Bot telemetry capture/validation and deterministic scoring model (`0..10`) with threshold routing (`pass`, `escalate_puzzle`, `maze_or_block`).
- [x] NAB-3 Add Not-a-Bot verification marker/token issuance after pass and enforce it in routing flow.
- [x] NAB-4 Add Not-a-Bot routing integration so medium-certainty traffic hits Not-a-Bot before puzzle escalation, with deterministic maze/block fallback.
- [x] NAB-5 Add Not-a-Bot admin visibility/config controls for thresholds, TTL, and attempt caps (read-only defaults plus optional mutability controls).
- [x] NAB-6 Add Not-a-Bot monitoring parity (`served`, `pass`, `escalate`, `fail`, `replay`, solve-latency buckets, abandonment estimate) and dashboard exposure.
- [x] NAB-7 Add dedicated e2e browser coverage for Not-a-Bot lifecycle and replay/abuse paths (unit + integration coverage is now in place).
- [x] NAB-8 Add operator docs and threshold tuning guidance aligned to low-friction managed-first routing.
- [x] NAB-9 Align Not-a-Bot control behavior to one-step state-of-the-art UX (checkbox-like control + auto-progress on activation).
- [x] NAB-10 Explicitly document the very-low-certainty managed/invisible path mapping (passive + JS/PoW) and keep Not-a-Bot medium-certainty only.
- [x] NAB-11 Preserve accessibility-neutral scoring policy: keyboard/touch flows remain pass-capable; assistive paths are never direct negative signals.
- [ ] NAB-12 Evaluate optional PAT-style private attestation signal ingestion as additive evidence only (non-blocking).

## P2 GEO Defence Maturity
- [ ] Add ASN/network dimensions in GEO policy logic (not just country list). (`src/signals/geo/mod.rs`, `src/config/mod.rs`, `src/admin/api.rs`)
- [ ] Add GEO/ASN observability and alerting (metrics, dashboard panels, docs). (`src/observability/metrics.rs`, dashboard, docs)

## P2 Modularization and Future Repository Boundaries
- [ ] Write objective criteria for future repo splits (API stability, release cadence, ownership, operational coupling).
## P3 Platform and Configuration Clarity
- [ ] Design runtime-agnostic architecture that keeps core detection logic portable while preserving Fermyon-first performance paths.
- [ ] Evaluate renaming `SHUMA_CHALLENGE_PUZZLE_RISK_THRESHOLD` to `SHUMA_BOTNESS_CHALLENGE_PUZZLE_THRESHOLD` to reflect botness semantics.
- [ ] Standardize terminology across code/UI/docs so `honeypot` and `maze` are used consistently instead of interchangeably.
- [ ] Document setup-time config bootstrapping clearly: how `make setup` creates/populates local env, how env-only vars are sourced, and how KV defaults are seeded and later overridden.
- [ ] Long-term option: integrate upstream identity/proxy auth (OIDC/SAML) for dashboard/admin instead of app-level key login.
