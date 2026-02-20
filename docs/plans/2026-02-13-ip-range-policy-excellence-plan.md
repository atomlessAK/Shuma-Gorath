# IP Range Policy Excellence Plan

Date: 2026-02-13
Status: Delivered for P1 scope (2026-02-20; includes managed-catalog staleness enforcement controls)

## Context

IP range policy controls are now implemented for the planned P1 scope (CIDR matching, managed sets, custom lists, action routing, advisory mode, admin wiring, and test coverage).

Extended research and source mapping are captured in:

- `docs/research/2026-02-20-ip-range-policy-research-synthesis.md`

## Goals

- Add reliable CIDR-based policy routing with safe precedence.
- Support managed feed-backed range sets with anti-poisoning controls.
- Keep operator-defined policy override clear and auditable.

## Non-goals

- Blindly trusting external feeds without validation.
- Irreversible broad block actions without dry-run capability.

## State-of-the-art Signals

1. Blacklist quality varies; freshness and validation are essential.
2. Feed poisoning and stale entries are practical risks.
3. Policy safety depends on confidence labels, rollout controls, and rollback paths.

## Internal vs Akamai Ownership

- `self_hosted_minimal`:
  - Internal CIDR evaluation and managed-set updates.
- `enterprise_akamai`:
  - Edge range enforcement may be authoritative.
  - Shuma keeps app-context routing and override controls.

## Proposed Architecture

### A. CIDR evaluation engine

- Add normalized CIDR parser and matcher.
- Support explicit action mapping by range set.
- Use deterministic precedence with first-match semantics in custom rules.

### B. Managed set lifecycle

- Versioned managed sets with provenance metadata.
- Signed/validated ingestion path and staged activation.
- Start with official machine-readable sources:
  - OpenAI (`gptbot`, `searchbot`, `chatgpt-user`)
  - GitHub `copilot` from `api.github.com/meta`
- Keep DeepSeek as unresolved/research-tracked until an official provenance endpoint exists.

### C. Policy precedence

- Explicit precedence model:
  - emergency allowlist,
  - operator custom policy,
  - managed set policy,
  - default routing.

### D. Safety tooling

- Dry-run mode with hit counters.
- Rollback to previous set version in one action.
- Broad-prefix and cardinality guardrails to prevent accidental over-blocking.
- Staleness policy for managed-set snapshots.

### E. Runtime efficiency

- Compile CIDRs once per config revision (avoid per-request parse loops).
- Cap rule and CIDR counts to bound request-time matching cost.

## Rollout Strategy

1. Ship matcher and dry-run mode first.
2. Introduce managed sets in advisory mode.
3. Enable action routing for high-confidence sets.
4. Add edge-authoritative integration for enterprise deployments.

## Structured Implementation TODOs

1. IP-1: Implement typed IP-range config model and strict admin/API validation.
2. IP-2: Implement CIDR parser/matcher with precedence:
   emergency allowlist > custom rules > managed sets.
3. IP-3: Implement action matrix:
   `403_forbidden`, `custom_message`, `drop_connection`, `redirect_308`, `rate_limit`, `honeypot`, `maze`, `tarpit`.
4. IP-4: Add advisory/dry-run mode with complete telemetry and event logging.
5. IP-5: Ship managed-set snapshot catalog with provenance/version metadata.
6. IP-6: Add update tooling with anti-poisoning guardrails and schema/cidr validation.
7. IP-7: Add managed-set stale snapshot handling and safe default behavior.
8. IP-8: Add integration tests for precedence/action routing and managed/custom overrides.
9. IP-9: Add enterprise Akamai/Cloudflare mapping hooks without bypassing Shuma precedence.
10. IP-10: Publish operational runbook (rollout, rollback, false-positive controls, staleness policy, cost controls).

## Enterprise Offering Snapshot (Akamai and Cloudflare)

- Akamai:
  - Application Security exposes Network Lists and IP/Geo Firewall APIs for reusable CIDR and location controls in edge policy.
  - Request Control Cloudlet provides production edge routing controls for IP/geo allow/deny/challenge style patterns.
- Cloudflare:
  - Cloudflare Lists support reusable IP/ASN/hostname sets for WAF custom rules.
  - IP Access Rules support quick allow/block/challenge actions by IP, ASN, country, or user agent, while Cloudflare recommends custom WAF rules for finer policy control.
- Planning implication:
  - Keep Shuma precedence and dry-run/rollback controls internal so policy safety is consistent across deployments.
  - Use enterprise list ingestion as an optional signal/enforcement source, but preserve explicit Shuma override ordering.

## Source References

- https://doi.org/10.14722/ndss.2020.24232
- https://www.usenix.org/conference/usenixsecurity20/presentation/oest-phishtime
- https://doi.org/10.14722/ndss.2025.242156
- https://research.ibm.com/publications/evaluating-ip-blacklists-effectiveness
- https://techdocs.akamai.com/application-security/reference/get-network-lists
- https://techdocs.akamai.com/application-security/reference/get-policy-ip-geo-firewall
- https://techdocs.akamai.com/cloudlets/docs/request-control
- https://developers.cloudflare.com/waf/tools/lists/
- https://developers.cloudflare.com/waf/tools/ip-access-rules/
