# GEO Fencing Excellence Plan

Date: 2026-02-13
Status: Proposed

## Context

GEO controls currently rely on trusted edge country headers and country-list routing (`allow`, `challenge`, `maze`, `block`). ASN and confidence dimensions are still missing.

## Goals

- Improve geolocation decision quality with confidence-aware policy.
- Add ASN/network dimensions and reduce over-reliance on country-only routing.
- Keep fallback behavior predictable when geo trust is unavailable.

## Non-goals

- City-level hard enforcement without confidence modeling.
- Treating untrusted geo headers as valid inputs.

## State-of-the-art Signals

1. IP geolocation has measurable error; confidence matters.
2. Country-only policies are too coarse for many abuse patterns.
3. Network/ASN context can materially improve policy precision.

## Internal vs Akamai Ownership

- `self_hosted_minimal`:
  - Internal country-based logic remains baseline.
- `enterprise_akamai`:
  - Akamai geo/network attribution can be authoritative.
  - Shuma keeps explicit policy precedence and fallback behavior.

## Proposed Architecture

### A. Trust-aware geo model

- Preserve existing trusted-header gate.
- Add confidence score and source-provenance to geo signals.

### B. ASN/network policy extension

- Add ASN and network policy lists with explicit precedence.
- Support combined country + ASN policy rules.

### C. Confidence-based routing

- High confidence: allow direct policy action.
- Low confidence: use geo as scoring signal only.

### D. Observability and safety

- Add metrics for geo source availability and route outcomes.
- Include false-positive review support in admin views.

## Rollout Strategy

1. Add ASN schema and confidence fields without behavior change.
2. Enable ASN scoring in advisory mode.
3. Enable combined geo/asn enforcement for narrow cohorts.
4. Expand cautiously after false-positive validation.

## Structured Implementation TODOs

1. GEO-1: Add ASN/network inputs to geo assessment model.
2. GEO-2: Add geo signal confidence and provenance fields.
3. GEO-3: Add combined country+ASN policy evaluation.
4. GEO-4: Add confidence-based signal-vs-enforce routing rules.
5. GEO-5: Add admin configuration for ASN lists and actions.
6. GEO-6: Add observability for geo/asn availability and outcomes.
7. GEO-7: Add tests for precedence and confidence fallback behavior.
8. GEO-8: Add enterprise Akamai authoritative geo mapping support.
9. GEO-9: Add dry-run tooling for geo/asn enforcement changes.
10. GEO-10: Publish GEO policy safety and rollback runbook.

## Enterprise Offering Snapshot (Akamai and Cloudflare)

- Akamai:
  - EdgeScape provides edge geolocation/network attributes (including ASN context) as request metadata.
  - Application Security IP/Geo Firewall and Request Control Cloudlet provide edge-native country/geo control surfaces.
- Cloudflare:
  - Cloudflare sends country and optional visitor-location headers (for enterprise geolocation use cases) and exposes geo fields in WAF expressions.
  - WAF custom rules can directly enforce country/ASN-based policy actions.
- Planning implication:
  - Treat edge-provided geo/ASN as high-value provenance signals in enterprise mode.
  - Keep Shuma confidence modeling, precedence, and fallback logic internal to avoid hard coupling to one provider's geo semantics.

## Source References

- https://doi.org/10.1145/2398776.2398790
- https://doi.org/10.1145/2835776.2835820
- https://doi.org/10.1016/j.comnet.2017.02.006
- https://arxiv.org/abs/2105.13389
- https://techdocs.akamai.com/cloud-security/docs/handle-adversarial-bots
- https://techdocs.akamai.com/edge-diagnostics/docs/data-in-edgescape-and-cloudlets-headers
- https://techdocs.akamai.com/application-security/reference/get-policy-ip-geo-firewall
- https://techdocs.akamai.com/cloudlets/docs/request-control
- https://developers.cloudflare.com/network/ip-geolocation/
- https://developers.cloudflare.com/waf/custom-rules/use-cases/block-by-geographical-location/
