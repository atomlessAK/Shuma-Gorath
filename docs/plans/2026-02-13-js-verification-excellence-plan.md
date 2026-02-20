# JS Verification Excellence Plan

Date: 2026-02-13
Status: Proposed

## Context

JS verification currently issues an IP-bound cookie (`js_verified`) after client script execution, with optional PoW and CDP reporting. This is functional but coarse, and it mixes gate-setting, telemetry collection, and anti-automation logic in one interstitial path.

## Goals

- Keep friction near zero for real browsers.
- Raise cost for scripted traffic without over-penalizing privacy-preserving clients.
- Improve replay resistance and flow integrity.
- Keep self-hosted defaults lightweight while supporting Akamai-backed enterprise policies.

## Non-goals

- Always-on heavy browser telemetry for all traffic.
- Hard dependency on third-party CAPTCHA services.

## State-of-the-art Signals

1. Modern anti-bot stacks use progressive verification and sequence checks.
2. Runtime behavior and consistency signals outperform single-step pass/fail gates.
3. Challenge systems are most effective when adaptively triggered by risk.

## Internal vs Akamai Ownership

- `self_hosted_minimal`:
  - Shuma-issued verification tokens and local JS interstitial flow.
- `enterprise_akamai`:
  - Akamai JS/challenge signals may short-circuit low-value repeats.
  - Shuma remains the final policy router and fallback verifier.

## Proposed Architecture

### A. Verification token hardening

- Move to short-lived, signed verification tokens with issue context:
  - ip bucket,
  - user-agent bucket,
  - issued-at and expiry,
  - risk tier.
- Rotate keys and enforce bounded replay windows.

### B. Progressive verification modes

- Low risk: lightweight JS check only.
- Medium risk: JS integrity + interaction sanity checks.
- High risk: JS + not-a-bot/puzzle/PoW escalation.

### C. Integrity and anti-bypass controls

- Add nonce-bound challenge pages.
- Validate event ordering and impossible timing patterns.
- Reject stale or duplicated challenge payloads.

### D. Separation of concerns

- Split JS gate rendering from CDP signal collection and PoW solve flow.
- Keep each capability independently toggleable and observable.

## Rollout Strategy

1. Introduce signed short-lived JS verification token format.
2. Enable progressive modes in advisory scoring only.
3. Route medium/high risk cohorts through step-up policy.
4. Tune thresholds and false-positive rates before broad enforcement.

## Structured Implementation TODOs

1. JS-1: Replace static cookie token model with signed short-lived verification token.
2. JS-2: Add nonce and replay protection for JS verification payloads.
3. JS-3: Add progressive JS mode selection by botness tier.
4. JS-4: Add event-order and timing plausibility validation primitives.
5. JS-5: Separate JS gate module from CDP and PoW orchestration.
6. JS-6: Add explicit no-JS fallback routing policy for high-risk requests.
7. JS-7: Add Akamai advisory/authoritative precedence hooks for enterprise profile.
8. JS-8: Add metrics for served/pass/fail/replay/latency by mode.
9. JS-9: Add integration tests for token expiry, replay, and downgrade paths.
10. JS-10: Document operational runbook and rollback thresholds.

## Enterprise Offering Snapshot (Akamai and Cloudflare)

- Akamai:
  - Bot Manager supports browser-side signal collection and configurable response strategy bands that can route requests to challenge/deny paths at the edge.
  - App and API Protector groups these controls with WAF/API protections for unified edge policy.
- Cloudflare:
  - JavaScript Detections can inject lightweight JS on HTML page views and issue a short-lived `cf_clearance` outcome that must still be enforced by WAF/custom rules.
  - Interstitial Challenge Pages provide managed challenge escalation.
  - Turnstile provides token-based challenge attestation and requires server-side Siteverify validation for security.
- Planning implication:
  - Keep Shuma JS verification token and replay model internal-first.
  - In enterprise mode, consume Akamai/Cloudflare outcomes as trigger modifiers and corroboration, not replacements for Shuma's policy and fallback routing.

## Source References

- https://developers.cloudflare.com/cloudflare-challenges/challenge-types/javascript-detections/
- https://developers.cloudflare.com/cloudflare-challenges/challenge-types/challenge-pages/
- https://developers.cloudflare.com/turnstile/get-started/server-side-validation/
- https://techdocs.akamai.com/cloud-security/docs/handle-adversarial-bots
- https://www.akamai.com/products/app-and-api-protector
- https://www.akamai.com/products/bot-manager
- https://arxiv.org/abs/2406.07647
- https://pmc.ncbi.nlm.nih.gov/articles/PMC7338186/
