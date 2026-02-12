# 🐙 Features & Roadmap

## 🐙 Current Features (Implemented)

- Honeypot endpoints (default: `/instaban`)
- Per-IP rate limiting
- Browser version blocking
- GEO scoring + policy routing (`allow/challenge/maze/block`) via trusted `X-Geo-Country`
- JS challenge with signed cookie
- Puzzle challenge step-up with single-use seeds
- Proof-of-work (PoW) step before JS verification (edge-served)
- Optional browser whitelist to bypass JS challenge
- maze crawler trap with auto-ban threshold
- CDP automation detection and reporting (`/cdp-report`)
- robots.txt generation and policy controls
- Admin API (ban/unban, analytics, events, config, maze, robots, CDP)
- Test mode (log-only, no enforcement)
- Event logging with retention (`SHUMA_EVENT_LOG_RETENTION_HOURS`)
- Prometheus metrics (`/metrics`)
- Composable defence modes per module (`off` / `signal` / `enforce` / `both`) for `rate`, `geo`, and `js`
- Effective-mode and signal-state observability for botness decisions
- Web dashboard for analytics and admin control
- Makefile-based setup, build, and test workflows

## 🐙 Near-Term Roadmap
- Human verification tuning (usability vs abuse resistance) and accessibility path
- Tarpit mode (slow responses to waste bot resources)
- Webhook notifications (Slack/Discord/PagerDuty)
- CSV/JSON export for events and analytics
- Additional geo/IP intelligence sources and fallbacks
- Expanded test coverage (edge cases + negative tests)
- Optional re-enable of challenge-on-ban feature

## 🐙 Longer-Term / Modern Threats

See `docs/bot-defence.md` for Shuma-Gorath layering strategy with managed edge bot defences (including Akamai Bot Manager).
