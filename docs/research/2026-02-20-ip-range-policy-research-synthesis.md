# IP Range Policy Research Synthesis

Date: 2026-02-20  
Status: Completed research tranche (`R-IP-01`..`R-IP-05`)  
Scope: CIDR/range blocking excellence for Shuma (`self_hosted_minimal` and `enterprise_akamai`)

## Research Corpus

- Ramanathan et al., BLAG (NDSS 2020): [doi.org/10.14722/ndss.2020.24232](https://doi.org/10.14722/ndss.2020.24232)
- Sheng et al., phishing blacklist analysis (CEAS 2009): [people.cs.vt.edu/gangwang/.../Sheng_Ceas09_Blacklist.pdf](https://people.cs.vt.edu/gangwang/publications/Sheng_Ceas09_Blacklist.pdf)
- Oest et al., PhishTime (USENIX Security 2020): [usenix.org/conference/usenixsecurity20/presentation/oest-phishtime](https://www.usenix.org/conference/usenixsecurity20/presentation/oest-phishtime)
- Li et al., HADES Attack (NDSS 2025): [ndss-symposium.org/wp-content/uploads/2025-2156-paper.pdf](https://www.ndss-symposium.org/wp-content/uploads/2025-2156-paper.pdf)
- Deri/Fusco, IP blacklist effectiveness (FiCloud 2023): [research.ibm.com/publications/evaluating-ip-blacklists-effectiveness](https://research.ibm.com/publications/evaluating-ip-blacklists-effectiveness)
- Cloudflare Lists docs: [developers.cloudflare.com/waf/tools/lists/](https://developers.cloudflare.com/waf/tools/lists/)
- Akamai Network Lists API docs: [techdocs.akamai.com/application-security/reference/get-network-lists](https://techdocs.akamai.com/application-security/reference/get-network-lists)
- OpenAI crawler docs + official IP lists:
  - [openai.com/gptbot](https://openai.com/gptbot)
  - [openai.com/chatgpt-user](https://openai.com/chatgpt-user)
  - [openai.com/searchbot](https://openai.com/searchbot)
  - [openai.com/gptbot.json](https://openai.com/gptbot.json)
  - [openai.com/chatgpt-user.json](https://openai.com/chatgpt-user.json)
  - [openai.com/searchbot.json](https://openai.com/searchbot.json)
- GitHub Meta API (official machine-readable service CIDRs incl. `copilot`): [api.github.com/meta](https://api.github.com/meta)

## Key Findings

1. Blacklists are inherently noisy and decay quickly.
- BLAG and subsequent work show precision/recall is highly sensitive to data quality and context; raw list membership alone is weak.
- Practical implication: IP-range policies should be confidence-tiered and action-tiered (advisory/challenge/maze before hard deny where possible).

2. Freshness is critical and staleness is a core failure mode.
- Sheng (2009) and PhishTime (2020) both show meaningful detection loss from lag; many abuse campaigns are short-lived.
- Practical implication: each managed CIDR set needs explicit freshness metadata, max staleness window, and deterministic expiry behavior.

3. External list poisoning is a real attacker strategy.
- HADES demonstrates that blocklist ecosystems can be manipulated through forged abuse observations/reporting paths.
- Practical implication: ingestion must enforce provenance constraints, schema/range sanity checks, and source trust tiering; never auto-promote unknown feeds to hard-block.

4. Single-source authority is operationally brittle.
- Cross-paper theme: a single upstream feed can be wrong, stale, or manipulated.
- Practical implication: Shuma should separate:
  - official-provider managed sets (high trust, machine-readable),
  - operator custom sets (explicit ownership/risk),
  - optional third-party reputation feeds (quarantined until quality evidence).

5. Enterprise edge products treat IP sets as reusable policy artifacts.
- Cloudflare/Akamai expose list abstractions with explicit API lifecycles and attach those lists to enforcement rules.
- Practical implication: Shuma should model managed sets and custom sets as first-class policy objects with version/provenance metadata, not ad-hoc string arrays.

## Official Source Availability for AI-Service Traffic

High-confidence machine-readable feeds currently available:

- OpenAI: GPTBot, ChatGPT-User, OAI-SearchBot JSON CIDR endpoints.
- GitHub: Meta API includes `copilot` CIDRs.

Observed gap:

- No equivalent official DeepSeek machine-readable CIDR list was identified in this research pass.
- Policy implication: do not ship an unverified DeepSeek managed set. Track as a standing research TODO until an official provenance source exists.

## What Shuma Needs for IP-Range Excellence

## 1) Deterministic policy model and precedence

- Global mode: `off | advisory | enforce`.
- Hard precedence:
  - emergency allowlist,
  - operator custom rules,
  - managed set policies,
  - default pipeline.
- First-match semantics within custom rules; stable ordering and explicit rule IDs.

## 2) Action matrix with safe fallbacks

Required actions:

- `403_forbidden`
- `custom_message`
- `drop_connection` (best-effort in HTTP runtime)
- `redirect_308`
- `rate_limit`
- `honeypot`
- `maze`
- `tarpit`

If an action is unavailable (for example tarpit provider not implemented), fallback must be deterministic and observable.

## 3) Managed set lifecycle (cost + safety)

- Versioned managed-set snapshot in-repo with metadata:
  - `set_id`, `provider`, `source_url`, `source_timestamp`, `snapshot_timestamp`, `version_hash`.
- Update tooling with strict ingestion guardrails:
  - allowlisted source domains only,
  - HTTPS only,
  - schema validation,
  - CIDR parse validation,
  - broad-prefix guards (`/0` style mistakes),
  - hard caps on entry count.
- Expiry/age policy:
  - stale managed sets should auto-disable for enforce mode unless explicitly overridden.

## 4) Runtime efficiency requirements

- Compile CIDR rules once per config revision (avoid per-request parse/deserialize loops).
- Cap max rule counts and CIDR counts to bound matching time and KV serialization size.
- Keep telemetry dimensions bounded (set IDs + actions), avoid raw CIDR logging on hot paths.

## 5) Observability + rollback

- Metrics:
  - match totals by source (`custom`/`managed`) and action,
  - advisory vs enforced decision counts,
  - fallback action counts.
- Event logs:
  - include set/rule ID, action, and mode.
- Rollback:
  - one-step disable of managed set and one-step global mode flip to `off`.

## Ownership Mapping

`self_hosted_minimal`
- Internal policy engine + local managed snapshot + operator custom rules.
- No required external provider integration.

`enterprise_akamai`
- Same internal precedence model remains authoritative for app-context decisions.
- Optional ingestion/sync from Akamai/Cloudflare list ecosystems as additional sources, never bypassing Shuma precedence and safety gates.

## Resulting Implementation Direction

Immediate engineering priorities:

1. Implement typed IP-range policy config and runtime evaluator with strict precedence and dry-run/advisory mode.
2. Add managed-set snapshot + update tooling for official OpenAI and GitHub Copilot sources.
3. Add full action matrix execution with deterministic fallback behavior.
4. Add admin API/config export, docs, and tests for precedence/action routing.
5. Keep DeepSeek as a research-tracked gap until official provenance data exists.
