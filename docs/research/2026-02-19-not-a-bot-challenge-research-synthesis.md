# Not-a-Bot Checkbox Challenge Research Synthesis

Date: 2026-02-19  
Status: Active implementation guidance (Round 2 refresh)

## Scope

Goal: identify the strongest lightweight "not-a-bot" checkbox implementation for medium-certainty traffic, positioned below puzzle difficulty, with low server cost and strong replay/abuse resistance.

## Evidence-backed findings

1. Server-side verification, short expiry, and single-use semantics are mandatory.
   - Turnstile and reCAPTCHA both require backend verification and document one-time, short-lived tokens.
   - Implication for Shuma: keep signed seed verification on submit, enforce expiry and replay rejection every time.

2. "Managed/non-interactive first" is now the dominant low-friction pattern.
   - Cloudflare positions managed challenge as adaptive by default, with interactive challenge only when needed.
   - Implication for Shuma: not-a-bot must remain a scored signal in an adaptive ladder, not a universal hard gate.

3. Replay resistance and token context binding are core protocol concerns.
   - Privacy Pass RFC guidance and Apple PAT guidance emphasize anti-replay and context-aware redemption.
   - Implication for Shuma: maintain one-time operation IDs, short TTLs, and request-binding checks (IP/UA/path class).

4. Accessibility-equivalent paths are required, not optional.
   - W3C guidance requires alternatives for CAPTCHA-style interactions.
   - Implication for Shuma: keyboard/touch flows must be pass-capable with equivalent anti-replay/expiry semantics.

5. CAPTCHA-only confidence has eroded due automation advances.
   - Recent papers show high solve rates for modern model-assisted CAPTCHA attacks.
   - Implication for Shuma: checkbox outcomes should never independently authorize sensitive actions.

6. Defender cost control is primarily a data-shape problem.
   - Best operational posture favors bounded telemetry schema, bounded cardinality, and minimal per-submit storage.
   - Implication for Shuma: keep compact telemetry, fixed scoring, and low-cardinality monitoring dimensions.

## Updated design requirements for Shuma

1. Keep not-a-bot as the medium-friction step in the botness ladder.
2. Preserve strict seed lifecycle rules: short-lived, single-use, signed, request-bound.
3. Keep telemetry compact and range-bounded; reject malformed payloads.
4. Keep external failures generic and non-oracular.
5. Add explicit operational controls (threshold, TTL, attempt caps) via runtime config.
6. Add monitoring parity for not-a-bot outcomes, solve-latency distribution, and abandonment estimate.
7. Keep room for optional privacy-preserving attestation inputs (PAT-like) as additive signal only.
8. Prefer one-step interaction for checkbox-style flows; avoid requiring an extra manual continue action when activation telemetry is already captured.
9. Keep accessibility pathways neutral in risk scoring (no direct penalty for keyboard/touch/assistive usage).

## Implementation mapping (this tranche)

- Add first-class config controls for not-a-bot routing/TTL/attempt behavior.
- Emit not-a-bot monitoring counters with bounded label vocab and latency buckets.
- Expose not-a-bot outcomes + abandonment on dashboard monitoring.
- Add dedicated browser e2e lifecycle coverage (serve, submit, replay rejection).

## Sources

- Turnstile server-side validation:
  - https://developers.cloudflare.com/turnstile/get-started/server-side-validation/
- Cloudflare challenge behavior and managed challenge:
  - https://developers.cloudflare.com/cloudflare-challenges/challenge-types/challenge-pages/
  - https://developers.cloudflare.com/cloudflare-challenges/concepts/how-challenges-work/
- Cloudflare Private Access Tokens:
  - https://developers.cloudflare.com/cloudflare-challenges/reference/private-access-tokens/
- Google reCAPTCHA verify:
  - https://developers.google.com/recaptcha/docs/verify
- RFC 9576 (Privacy Pass architecture), anti-replay concerns:
  - https://www.rfc-editor.org/rfc/rfc9576.html
- RFC 9577 (token issuance protocol):
  - https://www.rfc-editor.org/rfc/rfc9577
- Apple Private Access Tokens guidance:
  - https://developer.apple.com/videos/play/wwdc2022/10077/
- W3C CAPTCHA accessibility techniques/guidance:
  - https://www.w3.org/TR/WCAG20-TECHS/G143.html
  - https://www.w3.org/TR/WCAG20-TECHS/G144.html
- CAPTCHA attack evidence:
  - https://www.usenix.org/conference/usenixsecurity25/presentation/halligan
  - https://arxiv.org/abs/2307.12108
  - https://arxiv.org/abs/1903.01003
