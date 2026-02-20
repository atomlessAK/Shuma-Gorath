# IP-Range Policy Runbook

Date: 2026-02-20  
Owner: Bot defence operations

## Scope

This runbook covers rollout, rollback, false-positive mitigation, feed freshness, and cost controls for:

- `ip_range_policy_mode`
- `ip_range_emergency_allowlist`
- `ip_range_custom_rules`
- `ip_range_managed_policies`

Managed built-in sets currently supported:

- `openai_gptbot`
- `openai_oai_searchbot`
- `openai_chatgpt_user`
- `github_copilot`

DeepSeek managed sets are intentionally unavailable until an official machine-readable source exists.

## Safe rollout sequence

1. Start in `off` and configure target custom/managed rules.
2. Switch to `advisory` and monitor `ip_range_policy_advisory` outcomes.
3. Validate no material false-positive traffic for the target ranges.
4. Switch to `enforce` only after advisory evidence is clean.
5. Prefer lower-friction actions first (`rate_limit` or `maze`) before hard blocking where feasible.

## False-positive mitigation

1. Add impacted CIDRs to `ip_range_emergency_allowlist` immediately.
2. Keep the allowlist entry in place while investigating source/rule mismatch.
3. If blast radius is unclear, set `ip_range_policy_mode=off` for immediate stop.
4. Remove or narrow offending custom/managed policy entries.
5. Re-enter `advisory` before returning to `enforce`.

## Rollback

Fast rollback order:

1. `ip_range_policy_mode=off` (global stop).
2. Disable affected managed/custom rules.
3. Add temporary emergency allowlist coverage for known good traffic.
4. Re-enable in `advisory` only after validation.

## Managed-set refresh and staleness policy

Refresh catalog from official sources:

```bash
make ip-range-catalog-update
```

Updater guardrails:

- HTTPS only and strict host allowlist.
- Source schema validation.
- CIDR validation + broad-prefix rejection.
- Per-set entry caps + growth-delta guard.

Recommended cadence:

- Refresh at least daily.
- In `enforce` mode, stale managed catalogs are automatically bypassed when catalog age exceeds `ip_range_managed_max_staleness_hours`.
- Keep `ip_range_allow_stale_managed_enforce=false` by default; set it true only as a short-lived emergency override with explicit risk acceptance.
- If catalog freshness cannot be restored quickly, degrade global mode to `advisory` or `off` until refreshed.

## Cost and efficiency controls

- Keep custom CIDR lists minimal and specific; avoid broad prefixes.
- Use deterministic precedence:
  emergency allowlist > custom rules > managed policies.
- Prefer `advisory` for broad exploratory policies to avoid expensive remediation from false positives.
- Keep `ip_range_custom_rules` and `ip_range_managed_policies` bounded; matching is optimized but still linear over active rule CIDRs.

## Security controls

- Treat managed catalog updates as controlled changes (review diff before merge).
- Never ingest unofficial, scraped, or user-submitted list sources directly into managed sets.
- For unknown providers, use explicit custom rules with ownership and expiry notes until official provenance exists.
