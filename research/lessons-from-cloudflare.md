Shuma-Gorath can learn 6 high-impact things from Cloudflare’s AI Labyrinth:

1. Move from obvious trap paths to covert in-page decoys.
Shuma currently routes explicit /maze/ and /trap/ paths (lib.rs (line 604), maze.rs (line 257)) and even advertises trap routes in robots docs (maze.md (line 10)). Cloudflare’s model is hidden links only bots see.

2. Use deception-first for medium-confidence bots, not always immediate bans.
Shuma hard-bans on honeypot/rate trips (lib.rs (line 775), lib.rs (line 809)). Cloudflare emphasizes wasting scraper resources before revealing defenses.

3. Turn labyrinth hits into richer detection signals.
Shuma tracks maze hits and threshold bans (lib.rs (line 355), lib.rs (line 390)) plus basic counters/stats (metrics.rs (line 177), admin.rs (line 1543)). Cloudflare uses decoy-follow behavior as a feedback loop to improve detection patterns over time.

4. Make decoy content less fingerprintable.
Shuma’s maze pages are deterministic per path (maze.rs (line 64), maze.rs (line 450)). Cloudflare describes diverse, pre-generated decoy content integrated into real pages, which is harder for bots to pattern-match.

5. Keep decoys selective and invisible to real users.
Inference: I don’t see a response-body transformer in current request flow, so Shuma mostly relies on dedicated trap endpoints rather than selective hidden-link injection. Cloudflare only presents labyrinth links to suspected AI scrapers to minimize UX/SEO impact.

6. Add a stronger AI-bot policy layer (managed + behavioral).
Shuma has strong robots policy generation and static crawler lists (robots.rs (line 13), robots.rs (line 63)), plus botness scoring (lib.rs (line 261)), but it can evolve toward explicit AI-bot action tiers (allow/monitor/challenge/labyrinth/block) with clearer precedence.

Cloudflare source note: the AI Labyrinth docs page was last updated Jan 23, 2026.

Sources:

Cloudflare AI Labyrinth docs
Cloudflare AI Labyrinth technical blog
Cloudflare AI bots concept page



further insights from cloudflare's bot defences

1. [JA3/JA4 fingerprint](https://developers.cloudflare.com/bots/additional-configurations/ja3-ja4-fingerprint/)
Shuma takeaway: add transport-fingerprint signals to bot scoring, not just request/IP behavior.
Current gap: botness is JS+geo+rate only in `/Users/jamestindall/Projects/Shuma-Gorath/src/lib.rs:261`.
Design direction: add optional trusted fingerprint fields (from edge headers) into config weights (`/Users/jamestindall/Projects/Shuma-Gorath/src/config.rs:108`) and ban fingerprints.

2. [Detection IDs](https://developers.cloudflare.com/bots/additional-configurations/detection-ids/)
Shuma takeaway: introduce stable, named detection IDs for high-confidence heuristics.
Current gap: you have free-form `signals` strings, but no canonical ID taxonomy or policy layer.
Design direction: add internal IDs (for scraping/account-takeover patterns) and let rules trigger by ID, similar to how CDP tiering already classifies strength in `/Users/jamestindall/Projects/Shuma-Gorath/src/cdp.rs:53`.

3. [JavaScript detections](https://developers.cloudflare.com/cloudflare-challenges/challenge-types/javascript-detections/)
Shuma takeaway: separate “passive JS telemetry” from “active challenge.”
Current state: JS only runs when challenged (`/Users/jamestindall/Projects/Shuma-Gorath/src/js.rs:69`), then sets `js_verified` (`/Users/jamestindall/Projects/Shuma-Gorath/src/js.rs:49`).
Design direction: add lightweight always-on JS detection for HTML traffic, and make enforcement conditional via rules (not automatic).

4. [Sequence rules](https://developers.cloudflare.com/bots/additional-configurations/sequence-rules/)
Shuma takeaway: model request order/timing, not just rate.
Current gap: rate limit is per-IP bucket/window only (`/Users/jamestindall/Projects/Shuma-Gorath/src/rate.rs:23`, `/Users/jamestindall/Projects/Shuma-Gorath/src/ip.rs:13`).
Design direction: add signed sequence cookie + operation IDs and rule checks like “A must precede B within N ms”.

5. [Block AI bots](https://developers.cloudflare.com/bots/additional-configurations/block-ai-bots/)
Shuma takeaway: make AI-bot controls a first-class policy surface (allow/challenge/maze/block) instead of only robots directives.
Current state: strong robots policy exists (`/Users/jamestindall/Projects/Shuma-Gorath/src/robots.rs:63`), but enforcement is not explicitly AI-category driven in the main decision tree (`/Users/jamestindall/Projects/Shuma-Gorath/src/lib.rs:976`).

6. [Managed robots.txt](https://developers.cloudflare.com/bots/additional-configurations/managed-robots-txt/)
Shuma takeaway: treat robots policy as managed/configurable profiles, not just generated text.
Current state: Shuma already has dynamic generation and AI/search toggles (`/Users/jamestindall/Projects/Shuma-Gorath/src/robots.rs:63`).
Upgrade idea: add versioned policy presets + scheduled bot-list refresh + policy diff visibility in admin.

7. [Static resources](https://developers.cloudflare.com/bots/additional-configurations/static-resources/)
Shuma takeaway: skip expensive bot checks for likely static assets by default.
Current gap: only manual path whitelist bypass exists (`/Users/jamestindall/Projects/Shuma-Gorath/src/lib.rs:615`).
Design direction: add built-in extension/content-type static bypass mode to reduce latency and false positives.

Most impactful next 3, in order:
1. Add detection IDs + policy matching.
2. Add sequence engine.
3. Add static-resource bypass defaults.

Sources used:
- [JA3/JA4 fingerprint](https://developers.cloudflare.com/bots/additional-configurations/ja3-ja4-fingerprint/)
- [Detection IDs](https://developers.cloudflare.com/bots/additional-configurations/detection-ids/)
- [JavaScript detections](https://developers.cloudflare.com/cloudflare-challenges/challenge-types/javascript-detections/)
- [Sequence rules](https://developers.cloudflare.com/bots/additional-configurations/sequence-rules/)
- [Block AI bots](https://developers.cloudflare.com/bots/additional-configurations/block-ai-bots/)
- [Managed robots.txt](https://developers.cloudflare.com/bots/additional-configurations/managed-robots-txt/)
- [Static resources](https://developers.cloudflare.com/bots/additional-configurations/static-resources/)

If you want, I can turn this into a concrete staged design doc and then implement Phase 1.