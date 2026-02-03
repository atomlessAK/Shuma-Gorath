# Shuma-Gorath Dashboard

Real-time dashboard for monitoring and managing Shuma-Gorath (Many-Angled Bot Defence).

## Features

- **Live Analytics**: Charts for blocked requests, ban rates, and traffic patterns
- **Event Log**: Real-time stream of bot detection events
- **Ban Management**: Manual IP ban/unban controls with quick-unban buttons
- **Test Mode Toggle**: Enable/disable test mode from the dashboard
- **Top IPs**: View most frequently flagged IP addresses
- **CDP Detection Status**: Monitor automation detection stats
- **Link Maze Stats**: Track crawler trap statistics
- **robots.txt Configuration**: Manage AI crawler opt-out policies
- **Auto-refresh**: Data updates every 30 seconds

## Files

| File | Purpose |
|------|---------|
| `index.html` | Main dashboard page with UI layout |
| `dashboard.js` | Chart rendering and data updates |
| `admin.js` | API helper functions for admin operations |
| `style.css` | Dashboard styling and responsive layout |

---

## Admin API Endpoints

All admin endpoints require `Authorization: Bearer <API_KEY>` header.

### Ban Management

#### `GET /admin/ban`
List all active bans for the site.

**Response:**
```json
{
  "bans": [
    {"ip": "1.2.3.4", "reason": "honeypot", "expires": 1706000000}
  ]
}
```

#### `POST /admin/ban`
Manually ban an IP address.

**Request Body:**
```json
{
  "ip": "1.2.3.4",
  "reason": "manual_ban",
  "duration": 3600
}
```

**Response:**
```json
{"status": "banned", "ip": "1.2.3.4"}
```

#### `POST /admin/unban?ip=<IP>`
Remove a ban for a specific IP.

**Response:** `Unbanned`

---

### Configuration

#### `GET /admin/config`
Get current server configuration.

**Response:**
```json
{
  "test_mode": false,
  "ban_duration": 21600,
  "ban_durations": {
    "honeypot": 86400,
    "rate_limit": 3600,
    "browser": 7200,
    "admin": 21600,
    "cdp": 43200
  },
  "rate_limit": 80,
  "honeypots": ["/bot-trap"],
  "maze_enabled": true,
  "maze_auto_ban": true,
  "maze_auto_ban_threshold": 5,
  "robots_enabled": true,
  "robots_block_ai_training": true,
  "robots_block_ai_search": false,
  "robots_allow_search_engines": true,
  "robots_crawl_delay": 0,
  "cdp_detection_enabled": false,
  "cdp_auto_ban": true,
  "cdp_detection_threshold": 0.7
}
```

#### `POST /admin/config`
Update server configuration. Supports partial updates.

**Request Body (example - toggle test mode):**
```json
{"test_mode": true}
```

**Request Body (example - update ban durations):**
```json
{
  "ban_durations": {
    "honeypot": 172800,
    "rate_limit": 1800
  }
}
```

**Request Body (example - CDP detection settings):**
```json
{
  "cdp_detection_enabled": true,
  "cdp_auto_ban": true,
  "cdp_detection_threshold": 0.8
}
```

**Request Body (example - robots.txt settings):**
```json
{
  "robots_enabled": true,
  "robots_block_ai_training": true,
  "robots_block_ai_search": false,
  "robots_crawl_delay": 10
}
```

---

### Analytics & Events

#### `GET /admin/analytics`
Get ban statistics and test mode status.

**Response:**
```json
{
  "ban_count": 42,
  "test_mode": false
}
```

#### `GET /admin/events?hours=<N>`
Query event log for recent events (default: 24 hours).

**Response:**
```json
{
  "recent_events": [
    {"ts": 1706000000, "event": "Ban", "ip": "1.2.3.4", "reason": "honeypot"}
  ],
  "event_counts": {"Ban": 10, "Unban": 2, "Challenge": 50},
  "top_ips": [["1.2.3.4", 15], ["5.6.7.8", 8]]
}
```

---

### Link Maze Honeypot

#### `GET /admin/maze`
Get link maze crawler trap statistics.

**Response:**
```json
{
  "total_hits": 150,
  "unique_crawlers": 12,
  "maze_auto_bans": 5,
  "deepest_crawler": {"ip": "1.2.3.4", "hits": 25},
  "top_crawlers": [
    {"ip": "1.2.3.4", "hits": 25},
    {"ip": "5.6.7.8", "hits": 18}
  ]
}
```

---

### robots.txt Management

#### `GET /admin/robots`
Get robots.txt configuration and preview.

**Response:**
```json
{
  "config": {
    "enabled": true,
    "block_ai_training": true,
    "block_ai_search": false,
    "allow_search_engines": true,
    "crawl_delay": 0
  },
  "content_signal_header": "noai, noimageai",
  "ai_training_bots": ["GPTBot", "Google-Extended", "CCBot", ...],
  "ai_search_bots": ["ChatGPT-User", "PerplexityBot", ...],
  "search_engine_bots": ["Googlebot", "Bingbot", ...],
  "preview": "# AI Training Bots - Blocked\nUser-agent: GPTBot\nDisallow: /\n..."
}
```

---

### CDP (Chrome DevTools Protocol) Detection

#### `GET /admin/cdp`
Get CDP automation detection configuration and statistics.

**Response:**
```json
{
  "config": {
    "enabled": false,
    "auto_ban": true,
    "detection_threshold": 0.7
  },
  "stats": {
    "total_detections": 0,
    "auto_bans": 0
  },
  "detection_methods": [
    "Error stack timing analysis (Runtime.Enable leak)",
    "navigator.webdriver property check",
    "Automation-specific window properties",
    "Chrome object consistency verification",
    "Plugin array anomaly detection"
  ]
}
```

#### `POST /cdp-report` (Public endpoint)
Submit a CDP detection report from client-side JavaScript.

**Request Body:**
```json
{
  "cdp_detected": true,
  "score": 0.85,
  "checks": ["webdriver", "automation_props", "cdp_timing"]
}
```

---

## Test Mode

When test mode is enabled:
- All bot trap logic runs normally
- Events are logged with `[TEST MODE]` suffix
- **No requests are actually blocked**
- Dashboard shows amber "TEST MODE ACTIVE" banner

Toggle test mode via:
1. Dashboard toggle switch in Admin Controls
2. API: `POST /admin/config` with `{"test_mode": true}`
3. Environment variable: `TEST_MODE=true`

---

## Usage

1. Start the Spin server: `make dev`
2. Open browser to: `http://127.0.0.1:3000/dashboard/index.html`
3. Configure API endpoint and key
4. Click "Refresh" to load data

## Development

The dashboard runs entirely client-side. To modify:

1. Edit HTML/JS/CSS files directly
2. Refresh browser to see changes
3. No build step required

---

## Configurable Ban Durations

Different ban types can have different expiry times:

| Ban Type | Default Duration | Config Key |
|----------|------------------|------------|
| Honeypot | 24 hours | `ban_durations.honeypot` |
| Rate Limit | 1 hour | `ban_durations.rate_limit` |
| Browser Block | 2 hours | `ban_durations.browser` |
| Admin Manual | 6 hours | `ban_durations.admin` |
| CDP Detection | 12 hours | `ban_durations.cdp` |

Update via API:
```bash
curl -X POST -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"ban_durations": {"honeypot": 172800, "rate_limit": 1800}}' \
  http://127.0.0.1:3000/admin/config
```
