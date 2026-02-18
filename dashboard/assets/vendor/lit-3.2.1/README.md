# Lit Runtime Vendor Bundle

This directory contains a pinned local Lit runtime snapshot for no-build-step
dashboard execution.

## Version

- `lit@3.2.1`
- `lit-html@3.3.2`
- `lit-element@4.2.2`
- `@lit/reactive-element@2.1.2`

## Source

Copied from local package manager install (`pnpm`) on 2026-02-17:

- `node_modules/.pnpm/lit@3.2.1/node_modules/lit`
- `node_modules/.pnpm/lit-html@3.3.2/node_modules/lit-html`
- `node_modules/.pnpm/lit-element@4.2.2/node_modules/lit-element`
- `node_modules/.pnpm/@lit+reactive-element@2.1.2/node_modules/@lit/reactive-element`

## Rationale

- Eliminate runtime CDN dependency for Lit.
- Keep dashboard runtime deterministic and auditable in production.
- Preserve no-build-step serving model.

## Entry Integrity (SHA-256)

- `npm/lit/index.js`: `b1993a57ee9b162bc5af6b2f4bc44623c0bd3496be5119c83d5325b04093b65c`
- `npm/lit-html/lit-html.js`: `5b38623fdb1513e0a559fc25d516f669027f937c86d4451f7e704d37c298e061`
- `npm/lit-element/lit-element.js`: `ee09e38303bb39c36959a961fb0a52663c798ee44493914472c3a61e84e55e3c`
- `npm/@lit/reactive-element/reactive-element.js`: `76e9815ec67d16684bff1444224bf62532d6318dbd8f25e7c1e9546158b1335f`
