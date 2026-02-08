# 🐙 Features & Roadmap

## 🐙 Current Features (Implemented)

- Honeypot endpoints (default: `/bot-trap`)
- Per-IP rate limiting
- Browser version blocking
- Geo-risk scoring via `X-Geo-Country`
- JavaScript challenge with signed cookie
- Proof-of-work (PoW) step before JS verification (edge-served)
- Optional browser whitelist to bypass JS challenge
- Link maze honeypot with auto-ban threshold
- CDP automation detection and reporting (`/cdp-report`)
- robots.txt generation and policy controls
- Admin API (ban/unban, analytics, events, config, maze, robots, CDP)
- Test mode (log-only, no enforcement)
- Event logging with retention (`SHUMA_EVENT_LOG_RETENTION_HOURS`)
- Prometheus metrics (`/metrics`)
- Web dashboard for analytics and admin control
- Makefile-based setup, build, and test workflows

## 🐙 Disabled or Experimental

- Challenge-based human verification (disabled in current flow; only POST handler exists)

## 🐙 Near-Term Roadmap

- Human verification research (challenge vs modern alternatives) and decision on re-enabling challenge flow
- Tarpit mode (slow responses to waste bot resources)
- Webhook notifications (Slack/Discord/PagerDuty)
- CSV/JSON export for events and analytics
- Additional geo/IP intelligence sources and fallbacks
- Expanded test coverage (edge cases + negative tests)
- Optional re-enable of challenge-on-ban feature

## 🐙 Longer-Term / Modern Threats

See `docs/akamai-bot-manager.md` for the Agentic AI & modern threat roadmap.
