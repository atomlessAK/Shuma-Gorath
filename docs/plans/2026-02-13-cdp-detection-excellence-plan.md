# CDP Detection Excellence Plan

Date: 2026-02-13
Status: Proposed

## Context

CDP detection exists as a client-side script and report endpoint, with score tiering and optional auto-ban on strong detections. Current design is useful but vulnerable to script fingerprinting, report suppression, and static-check evasion.

## Goals

- Improve resilience against anti-detection tooling.
- Keep CDP as one signal in a fused model, not a sole enforcement trigger.
- Reduce chances of brittle false positives.

## Non-goals

- Building an unstoppable one-shot CDP detector.
- Treating client-side JS signals as authoritative in isolation.

## State-of-the-art Signals

1. Static detector scripts are eventually fingerprinted and bypassed.
2. Multi-signal and consistency-based detection remains more robust than single probes.
3. Bot frameworks adapt quickly; detectors need rotation and layered checks.

## Internal vs Akamai Ownership

- `self_hosted_minimal`:
  - Internal CDP probes and scoring as advisory-to-enforcement input.
- `enterprise_akamai`:
  - Akamai bot signals can corroborate CDP findings.
  - Shuma retains local explainability and final scoring.

## Proposed Architecture

### A. Probe diversity and rotation

- Maintain multiple probe families and rotate subsets.
- Version detector payloads and track effectiveness over time.

### B. Report integrity

- Bind reports to signed challenge nonce and expiry.
- Reject detached reports with no corresponding active challenge context.

### C. Correlation layer

- Combine CDP tier with fingerprint inconsistencies and rate/sequence signals.
- Require corroboration for high-impact actions (for example auto-ban).

### D. Safe enforcement policy

- Keep strong CDP as escalation trigger first, direct ban second.
- Add explicit cool-down and revalidation logic.

## Rollout Strategy

1. Add nonce-bound reporting and detector version tags.
2. Use CDP in advisory mode only for initial cohorts.
3. Enable corroborated enforcement thresholds.
4. Periodically rotate and retire low-value probe variants.

## Structured Implementation TODOs

1. CDP-1: Add detector versioning and probe-family rotation support.
2. CDP-2: Bind CDP report payloads to signed challenge nonce.
3. CDP-3: Add report expiry and replay controls.
4. CDP-4: Add correlation rules with fingerprint and rate signals.
5. CDP-5: Require corroboration before auto-ban actions.
6. CDP-6: Add false-positive review workflow in admin logs.
7. CDP-7: Add metrics per probe family (`hit`, `bypass`, `false_positive`).
8. CDP-8: Add synthetic evasive test harness for regression checks.
9. CDP-9: Integrate Akamai bot-confidence as optional corroborating input.
10. CDP-10: Publish rotation cadence and rollback runbook.

## Enterprise Offering Snapshot (Akamai and Cloudflare)

- Akamai:
  - App and API Protector now includes Browser Impersonation Detection capabilities to identify tooling that mimics browsers.
  - Bot Manager already contributes browser fingerprinting and behavioral risk signals that can corroborate CDP findings.
- Cloudflare:
  - Cloudflare does not expose a dedicated "CDP detected" flag, but provides adjacent anti-automation surfaces (`js_detection.passed`, bot score, detection IDs, JA3/JA4) that are useful corroborating signals.
  - Managed challenge flows can enforce step-up when corroborated risk rises.
- Planning implication:
  - Keep CDP probe execution and scoring as Shuma-native logic.
  - Treat Akamai/Cloudflare outputs as corroboration inputs in the fused model, not as a CDP replacement signal.

## Source References

- https://rebrowser.net/blog/how-to-fix-runtime-enable-cdp-detection-of-puppeteer-playwright-and-other-automation-libraries
- https://kaliiiiiiiiii.github.io/brotector/
- https://doi.org/10.1007/978-3-030-29962-0_28
- https://arxiv.org/abs/2406.07647
- https://techdocs.akamai.com/cloud-security/docs/handle-adversarial-bots
- https://www.akamai.com/products/bot-manager
- https://www.akamai.com/newsroom/press-release/akamai-strengthens-app-and-api-protector-with-new-capabilities
- https://developers.cloudflare.com/bots/reference/bot-management-variables/
- https://developers.cloudflare.com/bots/additional-configurations/detection-ids/
- https://developers.cloudflare.com/cloudflare-challenges/challenge-types/challenge-pages/
