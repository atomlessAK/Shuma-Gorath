# Defence Excellence Program

Date: 2026-02-13
Status: Proposed

## Purpose

This program expands the maze-excellence planning model to every major defence capability in Shuma-Gorath.

Each plan is structured to answer:

- what state-of-the-art research says,
- what Shuma should keep internal for `self_hosted_minimal`,
- where `enterprise_akamai` should be advisory vs authoritative,
- what execution sequence gets to production safely.

## Plan Set

1. `docs/plans/2026-02-13-fingerprint-excellence-plan.md`
2. `docs/plans/2026-02-13-js-verification-excellence-plan.md`
3. `docs/plans/2026-02-13-cdp-detection-excellence-plan.md`
4. `docs/plans/2026-02-13-pow-excellence-plan.md`
5. `docs/plans/2026-02-13-challenge-lite-excellence-plan.md`
6. `docs/plans/2026-02-13-puzzle-challenge-excellence-plan.md`
7. `docs/plans/2026-02-13-rate-limiting-excellence-plan.md`
8. `docs/plans/2026-02-13-http-tarpit-excellence-plan.md`
9. `docs/plans/2026-02-13-ip-range-policy-excellence-plan.md`
10. `docs/plans/2026-02-13-geo-fencing-excellence-plan.md`
11. `docs/plans/2026-02-13-ssh-tarpit-excellence-plan.md`

## Recommended Execution Order

1. Fingerprint
2. JS verification
3. CDP detection
4. Rate limiting
5. IP range policy
6. GEO fencing
7. Challenge Lite
8. Puzzle challenge
9. PoW
10. HTTP tarpit
11. SSH tarpit

## Why This Order

- Signal quality first: fingerprint + JS/CDP improve downstream policy precision.
- Baseline control next: rate limiting + IP/GEO reduce obvious abuse volume.
- Challenge depth after that: challenge-lite, puzzle, and PoW become better targeted.
- Cost-imposition last: tarpit features are safest once scoring and policy quality are mature.

## Cross-Plan Guardrails

- Preserve internal defaults for `self_hosted_minimal`.
- Keep Shuma as policy orchestrator in all personas.
- Prefer Akamai authoritative mode only where edge vantage materially improves correctness or cost.
- Keep fallback-to-internal behavior explicit and observable.
- Ensure every plan has rollback thresholds before enforcement expansion.
