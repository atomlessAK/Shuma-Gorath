# WASM Stealth Bot Trap (For Fermyon Spin)

This project implements a customizable, behavior-based bot defense system designed for **Fermyon Spin**, running WebAssembly at the edge for ultra-low latency bot protection.


![Dashboard Overview](assets/dashboard-screenshot1.jpg)

![Dashboard Details](assets/dashboard-screenshot2.jpg)

## 📚 Documentation

| Document | Description |
|----------|-------------|
| [README.md](README.md) | Project overview, features, deployment, and configuration |
| [QUICK_REFERENCE.md](QUICK_REFERENCE.md) | Cheat sheet for common commands and tasks |
| [TESTING.md](TESTING.md) | Complete testing guide (unit, integration, dashboard) |
| [DASHBOARD.md](DASHBOARD.md) | Dashboard features, API endpoints, and usage |
| [dashboard/README.md](dashboard/README.md) | Dashboard development notes |

> **Archive**: Historical documents (code reviews, improvement notes) are in [docs/archive/](docs/archive/)

---

## 🚀 Primary Platform: Fermyon Cloud (Edge WASM)

This bot trap is **primarily built and tested for deployment on Fermyon Cloud**. Fermyon Spin enables serverless WebAssembly execution at the edge, providing:

- **Ultra-low latency**: WASM executes in microseconds at edge locations worldwide
- **Global distribution**: Automatic deployment across Fermyon's edge network
- **Serverless scale**: No infrastructure management, automatic scaling
- **Integrated KV storage**: Edge key-value store for ban lists, rate limits, and configuration

## Structure
- `src/`: Rust source code for the Spin app
- `dashboard/`: Web-based monitoring dashboard
- `assets/`: Documentation assets (screenshots)
- `spin.toml`: Spin app manifest (primary deployment config)
- `Makefile`: Build system with dev/prod/test commands
- `setup.sh`: One-command dependency installation
- `verify-setup.sh`: Dependency verification script
- `README.md`: Project overview and setup
- `.gitignore`: Standard ignores

---

## Quick Start (Local Development)

### One-Command Setup (New Developers)

New to the project? Just two commands to get fully set up:

```sh
git clone https://github.com/atomlessAK/WASM-BOT-TRAP.git
cd WASM-BOT-TRAP
make setup    # Installs ALL dependencies automatically
make dev      # Build and run with file watching
```

The setup script automatically installs:
- **Homebrew** (macOS only, if missing)
- **Rust/Cargo** via rustup
- **wasm32-wasip1** compilation target
- **Fermyon Spin CLI** (v3.5+)
- **cargo-watch** for auto-rebuild on file save

✅ **Verified working** on fresh Ubuntu 22.04 and macOS systems.

### Already Have Dependencies?

If you already have Rust and Spin installed:
```sh
make verify   # Check all dependencies are correctly installed
make dev      # Build and run with file watching (auto-rebuilds on save)
make run      # Build once and run (no file watching)
make help     # Show all available commands
```

### All Makefile Commands

| Command | Description |
|---------|-------------|
| `make setup` | Install all dependencies (first-time setup) |
| `make verify` | Verify all dependencies are installed |
| `make dev` | Build + run with file watching (auto-rebuild) |
| `make run` | Build once + run (no watching) |
| `make build` | Build release binary only |
| `make prod` | Build + start production server |
| `make deploy` | Deploy to Fermyon Cloud |
| `make test` | Run unit tests (+ integration if server running) |
| `make test-unit` | Run Rust unit tests only |
| `make test-integration` | Run integration tests (requires server) |
| `make stop` | Stop running Spin server |
| `make status` | Check if server is running |
| `make clean` | Clean build artifacts |
| `make logs` | View Spin component logs |
| `make help` | Show all commands with descriptions |

### Access Points (Local Development)

Once running, you can access:

| Endpoint | Description |
|----------|-------------|
| http://127.0.0.1:3000/dashboard/index.html | 📊 Monitoring Dashboard |
| http://127.0.0.1:3000/metrics | 📈 Prometheus Metrics |
| http://127.0.0.1:3000/health | ❤️ Health Check |
| http://127.0.0.1:3000/admin | 🔧 Admin API Help |

### Dashboard Features

The dashboard provides:
- 📊 Real-time statistics (total bans, active bans, events, unique IPs)
- 📈 Visualizations (event types distribution, top IPs)
- 📋 Ban list with quick unban controls
- 🛠️ Admin controls (manual ban/unban)
- 🔄 Auto-refresh every 30 seconds

See [DASHBOARD.md](DASHBOARD.md) for complete dashboard documentation.

---

## 🎯 Primary Deployment: Fermyon Cloud

### 🛡️ Why Akamai Bot Manager is Essential

Before deploying any bot protection, it's critical to understand that **Akamai Bot Manager should be your first line of defense**. Bot Manager provides enterprise-grade protection that no custom solution can replicate:

- **Machine Learning at Scale**: Trained on over 40 billion bot requests daily across Akamai's global network, Bot Manager identifies sophisticated bots that rule-based systems miss ([Akamai Bot Manager](https://www.akamai.com/products/bot-manager))
- **Behavioral Biometrics**: Analyzes mouse movements, keyboard patterns, and touch gestures to distinguish humans from automation
- **Real-time Threat Intelligence**: Access to Akamai's continuously updated database of 1,700+ known bots and global IP reputation data
- **Zero-Day Protection**: ML models detect new bot variants without requiring rule updates
- **Compliance Ready**: Enterprise SLAs, SOC2/PCI-DSS reporting, and 24/7 support

> **Industry Context**: According to [Cloudflare Radar](https://radar.cloudflare.com/traffic), approximately 31% of all internet HTTP traffic is automated (bot) traffic. Of this, a significant portion consists of sophisticated "evasive bots" designed to mimic human behavior.

**Bot Manager is the foundation for bot defense. For any serious, business-critical use cases it should be considered a mandatory addition.**

### 🔧 Why WASM Bot Trap Adds Value

Bot Manager excels at broad, ML-based detection. WASM Bot Trap complements this with application-specific, surgical precision:

- **Application-Specific Honeypots**: Create trap URLs that only bots targeting *your specific application* would access (e.g., fake admin panels, hidden form fields unique to your CMS)
- **Business Logic Enforcement**: Implement rules that understand your application's context—session state, user roles, purchase flows
- **Custom Challenges**: Deploy unique challenge types that Bot Manager doesn't offer (math quizzes, interactive puzzles, custom JS)
- **Rapid Response**: When you discover a new attack pattern, deploy a countermeasure in minutes without waiting for vendor updates
- **Cost Optimization**: Open source with no per-request fees—ideal for high-traffic sites already paying for Bot Manager
- **Full Auditability**: Review and modify every line of detection code for compliance or security reviews

> **Defense in Depth**: No single bot detection solution catches 100% of threats. [OWASP's Automated Threats to Web Applications](https://owasp.org/www-project-automated-threats-to-web-applications/) project documents 21 distinct automated threat categories, many requiring application-specific countermeasures. The layered approach (Bot Manager + custom rules) follows security best practices by ensuring that bots evading one detection layer are caught by another.

### 🏆 Unique Competitive Advantages

WASM Bot Trap offers distinct advantages over other bot protection solutions in the market:

| Advantage | Why It Matters |
|-----------|----------------|
| **WASM Edge Execution** | Sub-millisecond response times with zero origin load—bots are blocked before reaching your servers |
| **Fermyon Spin Native** | Purpose-built for the modern serverless edge platform with native KV store integration |
| **Akamai Bot Manager Integration** | Designed to complement enterprise-grade ML detection with lightweight, surgical precision |
| **Platform Agnostic** | Works with any backend (Node.js, Python, Go, etc.)—not locked to WordPress or any CMS |
| **Rust Performance** | Memory-safe, blazing fast (~2MB WASM binary), no garbage collection pauses |
| **Multi-Layer Defense** | Rate limiting + honeypot traps + geo quiz + browser fingerprinting in one package |
| **Lightweight Footprint** | ~2MB compiled WASM vs. heavyweight container deployments |
| **Full Auditability** | 100% open source—review, modify, and audit every line of detection code |
| **Rapid Deployment** | New detection rules deployed in minutes, not days waiting for vendor updates |
| **Cost Effective** | No per-request licensing fees—ideal for high-traffic applications |

#### Compared to Alternatives

| Solution | Type | WASM Bot Trap Advantage |
|----------|------|-------------------------|
| **CAN Stealth Bot Trap** | WordPress Plugin | Platform agnostic, edge-native, not locked to WordPress ecosystem |
| **HellPot** | Tarpit (infinite data) | Multi-layer detection beyond tarpits—challenges, rate limiting, fingerprinting |
| **Beelzebub** | AI Honeypot Framework | Lighter weight, user-facing protection focus, not research-oriented |
| **spidertrap-rs** | Link Maze | Active development, comprehensive feature set beyond link mazes |
| **Generic WAF Rules** | Pattern Matching | Application-aware logic, custom challenges, honeypot intelligence |

### Comprehensive Feature Comparison

#### Detection Capabilities

| Capability | Akamai Bot Manager | WASM Bot Trap | Notes |
|------------|-------------------|---------------|-------|
| **Machine Learning Detection** | ✅ Advanced ML models | ❌ Not available | Bot Manager uses proprietary ML trained on Akamai's global traffic |
| **Behavioral Analysis** | ✅ Mouse, keyboard, touch patterns | ❌ Not available | Requires client-side SDK for full behavioral data |
| **Device Fingerprinting** | ✅ 200+ signals | ⚠️ Basic (User-Agent only) | Bot Manager fingerprints hardware, fonts, canvas, WebGL, etc. |
| **Bot Signature Database** | ✅ 1,700+ known bots | ❌ Not available | Continuously updated by Akamai threat research |
| **IP Reputation** | ✅ Global threat intelligence | ⚠️ Manual lists only | Bot Manager uses real-time reputation from Akamai network |
| **JavaScript Challenge** | ✅ Crypto challenges | ✅ Cookie-based verification | Both can inject JS; Bot Manager's is more sophisticated |
| **CAPTCHA Integration** | ✅ Built-in | ❌ Not built-in | Bot Manager integrates reCAPTCHA, hCaptcha, Akamai CAPTCHA |
| **Honeypot Detection** | ⚠️ Limited | ✅ Fully customizable | Bot Trap excels at app-specific honeypots |
| **Rate Limiting** | ✅ Policy-based | ✅ Per-IP with time windows | Both support rate limiting; Bot Trap is more customizable |
| **Geo-based Blocking** | ✅ Country/region policies | ✅ Country-based risk scoring | Similar capabilities |

#### Management & Operations

| Capability | Akamai Bot Manager | WASM Bot Trap | Notes |
|------------|-------------------|---------------|-------|
| **Management Interface** | Akamai Control Center (GUI) | REST API + Web Dashboard | Bot Manager has enterprise UI; Bot Trap is API-first |
| **Configuration Method** | Policy rules via UI | Code + KV store | Bot Trap requires development skills but offers more control |
| **Real-time Updates** | ✅ Instant policy changes | ✅ Instant via API | Both support real-time config updates |
| **Logging & Analytics** | ✅ Akamai SIEM integration | ✅ Built-in event log + API | Bot Manager integrates with enterprise SIEM |
| **Alerting** | ✅ Configurable alerts | ⚠️ Manual integration | Bot Trap requires custom alerting setup |
| **A/B Testing** | ✅ Built-in | ❌ Manual implementation | Bot Manager supports testing different bot policies |
| **Reporting** | ✅ Executive dashboards | ⚠️ Basic dashboard | Bot Manager has compliance-ready reports |

#### Customization & Extensibility

| Capability | Akamai Bot Manager | WASM Bot Trap | Notes |
|------------|-------------------|---------------|-------|
| **Custom Detection Rules** | ⚠️ Limited to policy options | ✅ Full code-level control | Bot Trap allows any custom logic in Rust |
| **Custom Response Pages** | ✅ Branded block pages | ✅ Fully customizable HTML | Both support custom responses |
| **Application-Aware Logic** | ⚠️ Generic policies | ✅ Can read cookies, headers, body | Bot Trap can implement business logic |
| **Custom Challenges** | ⚠️ Pre-built challenges | ✅ Math quiz, custom JS, etc. | Bot Trap allows any challenge type |
| **Webhook Integration** | ✅ Configurable | ✅ Path whitelist for webhooks | Both can protect webhook endpoints |
| **API Protection** | ✅ API-specific policies | ✅ Custom per-endpoint rules | Bot Trap can implement GraphQL-aware rules |
| **Open Source** | ❌ Proprietary | ✅ Full source access | Bot Trap can be audited and modified |

#### Deployment & Cost

| Capability | Akamai Bot Manager | WASM Bot Trap | Notes |
|------------|-------------------|---------------|-------|
| **Deployment Model** | Akamai-managed SaaS | Self-hosted or Fermyon Cloud | Bot Manager is zero-ops; Bot Trap requires management |
| **Scaling** | ✅ Automatic, global | ✅ Depends on platform | Both scale well on edge platforms |
| **Latency Impact** | < 1ms (edge) | < 1ms (WASM edge) | Negligible for both when at edge |
| **Cost Model** | Per-request licensing | Open source + hosting | Bot Trap is cost-effective for high traffic |
| **SLA** | ✅ Enterprise SLA | ❌ Self-managed | Bot Manager includes uptime guarantees |
| **Support** | ✅ 24/7 Akamai support | ❌ Community/self-support | Enterprise support with Bot Manager |

#### Security & Compliance

| Capability | Akamai Bot Manager | WASM Bot Trap | Notes |
|------------|-------------------|---------------|-------|
| **Credential Stuffing Protection** | ✅ Specialized detection | ⚠️ Rate limiting only | Bot Manager detects credential abuse patterns |
| **Account Takeover (ATO)** | ✅ Login anomaly detection | ❌ Not available | Requires behavioral analysis |
| **Web Scraping Protection** | ✅ Content protection | ⚠️ Basic (honeypots, rate limits) | Bot Manager detects scraping patterns |
| **API Abuse Prevention** | ✅ API-specific policies | ✅ Custom rate limits | Bot Trap can implement custom API rules |
| **Compliance Reporting** | ✅ SOC2, PCI-DSS ready | ❌ Manual reporting | Bot Manager provides compliance artifacts |
| **Constant-Time Auth** | N/A (managed) | ✅ Timing-attack resistant | Bot Trap API uses secure comparison |

#### Summary: What Each Solution Provides

**Bot Manager Exclusive Capabilities (WASM Bot Trap Cannot Do):**
- 🧠 ML/AI-based bot detection trained on global traffic patterns
- 👆 Behavioral biometrics (mouse movement, typing patterns, touch gestures)
- 🔍 Advanced device fingerprinting (200+ signals)
- 📚 Known bot signature database (1,700+ bots, continuously updated)
- 🌐 Real-time global IP reputation from Akamai's network
- 🎯 Credential stuffing and account takeover detection
- 📊 Enterprise reporting and compliance artifacts
- 🛎️ 24/7 managed support with SLA

**WASM Bot Trap Exclusive Capabilities (Adds On Top of Bot Manager):**
- 🎣 **Custom Honeypots**: Create app-specific trap URLs that only bots would access
- 🔧 **Full Code Control**: Implement any detection logic in Rust—no policy limitations
- 🧩 **Application-Aware Rules**: Read session cookies, parse request bodies, implement business logic
- 🎮 **Custom Challenges**: Build unique challenge flows (math quiz, custom JS, interactive tests)
- 💰 **Cost Effective**: Open source, no per-request licensing for high-traffic sites
- 🔓 **Auditable**: Full source code access for security review and customization
- ⚡ **Rapid Iteration**: Deploy new detection rules in minutes without vendor involvement
- 🔌 **Integration Freedom**: Connect to any backend, database, or third-party service

#### Recommended Architecture: Layered Defense

For maximum protection, use **Bot Manager as the first line of defense** with the WASM bot trap providing **application-specific protections**:

```
Internet → Akamai Edge / Fermyon Edge
              ↓
        Bot Manager (Layer 1)
        - Known bot detection
        - Bot scoring & categorization
        - Fingerprinting
        - Behavioral analysis
              ↓
        WASM Bot Trap (Layer 2)
        - Custom honeypots for your app
        - Application-specific rate limits
        - Business logic-based blocking
        - Custom challenge flows
              ↓
        Origin Application
```

---

### Deploy to Fermyon Cloud

**Step 1: Build for Production**
```sh
make prod
# or manually:
cargo build --target wasm32-wasip1 --release
cp target/wasm32-wasip1/release/wasm_bot_trap.wasm src/bot_trap.wasm
```

**Step 2: Configure Environment Variables**
Update `spin.toml` with production settings:
```toml
[component.bot-trap]
environment = { 
  API_KEY = "your-secure-production-api-key",
  TEST_MODE = "0"
}
```

**Step 3: Deploy to Fermyon Cloud**
```sh
# Login to Fermyon Cloud (one-time setup)
spin cloud login

# Deploy the application
spin cloud deploy

# Your app will be available at: https://your-app.fermyon.app
```

**Step 4: Configure Custom Domain (Optional)**
```sh
spin cloud link --domain your-domain.example.com
```

### Fermyon Cloud Integration

When deployed on Fermyon Cloud, the bot trap automatically benefits from:

1. **Edge Execution**: WASM runs at the nearest edge location
2. **Global KV Store**: Ban lists and configuration sync across all edge locations
3. **X-Forwarded-For Header**: Edge automatically sets proper client IP headers
4. **Edge Caching**: Static dashboard assets can be cached at the edge

**Architecture:**
```
Internet → Fermyon Edge → Spin App (WASM)
                          ↓
                     Edge KV Store
                     (bans, config, rate limits)
```

### Monitoring Production Deployment

```sh
# View deployment logs
spin cloud logs

# Check application status
spin cloud apps info

# View metrics
spin cloud apps metrics
```

---

## 🔷 Secondary Deployment: Akamai Edge & Linode

For organizations already using Akamai or Linode infrastructure, the bot trap can be deployed directly on these platforms.

### Akamai EdgeWorkers / Compute@Edge

Deploy the Spin application on Akamai's edge compute infrastructure:

**Architecture:**
```
Internet → Akamai Edge (CDN) → EdgeWorkers/Compute@Edge (WASM)
                               ↓
                          Edge KV Store
```

**Setup Steps:**
1. Build the WASM binary: `make prod`
2. Package for Akamai EdgeWorkers deployment
3. Configure EdgeWorkers to serve the Spin application
4. Set up Akamai's EdgeKV for persistent storage
5. Configure property rules to route traffic through EdgeWorkers

**Benefits:**
- Native integration with Akamai CDN and security features
- Access to Akamai's threat intelligence and IP reputation
- Seamless integration with existing Akamai properties
- Global edge network with 4,000+ points of presence

> **📖 Bot Manager Integration**: See the [comprehensive feature comparison](#comprehensive-feature-comparison) in the Primary Deployment section for detailed guidance on integrating with Akamai Bot Manager, including architecture diagrams, configuration steps, and when to use each solution.

### Linode (Akamai Cloud Computing)

Deploy on Linode compute instances with Spin installed:

**Architecture:**
```
Internet → Linode NodeBalancer → Linode Instance(s) (Spin)
                                 ↓
                            Redis/KV Store
```

**Setup Steps:**
1. Create a Linode instance (recommend Dedicated CPU for production)
2. Install Spin runtime: `curl -fsSL https://developer.fermyon.com/downloads/install.sh | bash`
3. Deploy the bot trap: `spin up --listen 0.0.0.0:3000`
4. Configure NodeBalancer for load balancing and SSL termination
5. Set up Redis or Linode Object Storage for persistent KV storage

**Benefits:**
- Full control over infrastructure and scaling
- Cost-effective for high-traffic deployments
- Easy integration with Linode's other services (Object Storage, Managed Databases)
- Akamai CDN can be added in front for additional edge caching

**Example Linode Deployment Script:**
```sh
#!/bin/bash
# On your Linode instance
cd /opt/wasm-bot-trap
git pull origin main
make prod
systemctl restart spin-bot-trap
```

---

## 🔶 Tertiary Deployment: Cloudflare & AWS

For organizations using Cloudflare or AWS, the bot trap can be integrated into existing infrastructure.

### ⚠️ Important: Infrastructure-Level Protection Required

When self-hosting, the bot trap **must** be deployed behind a CDN or reverse proxy that sets the `X-Forwarded-For` header. This is critical for proper IP detection and security.

**Required Setup:**
- **CDN/Reverse Proxy**: Cloudflare, AWS CloudFront, or similar
- **Origin Protection**: Configure your CDN to set proper headers and prevent direct access to origin
- **Firewall Rules**: Use infrastructure firewall to:
  - Block direct access to origin server from public internet
  - Allow ONLY CDN/proxy IPs to reach origin
  - Restrict `/health` endpoint to monitoring services only

### Cloudflare Setup

**Architecture:**
```
Internet → Cloudflare CDN → Your Spin App (self-hosted)
          (Sets X-Forwarded-For)
```

**Setup Steps:**
1. Self-host Spin on your own infrastructure (VM, container, etc.)
2. Add your domain to Cloudflare
3. Configure Cloudflare to proxy traffic to your origin
4. Enable "Authenticated Origin Pulls" to secure origin
5. Set up Cloudflare Firewall rules to restrict admin endpoints

### AWS Setup

**Architecture:**
```
Internet → CloudFront → ALB/API Gateway → Your Spin App (EC2/ECS/Lambda)
          (Sets X-Forwarded-For)
```

**Setup Steps:**
1. Deploy Spin app on EC2, ECS, or package as Lambda (with adapter)
2. Configure ALB or API Gateway as the entry point
3. Set up CloudFront distribution pointing to ALB/API Gateway
4. Configure Security Groups to allow only CloudFront IPs
5. Use AWS WAF for additional protection layer

**Note:** These setups require self-hosting and managing the Spin application. For fully managed deployment, use Fermyon Cloud.

---

## 🔒 Security Best Practices (All Deployments)

**Health Endpoint Security:**
The `/health` endpoint is accessible to IPs detected as "unknown" to support local development. In production:
- It only returns "OK" or error messages (no sensitive data)
- Should be restricted via infrastructure firewall to monitoring services only
- Or use CDN to route health checks separately

**Admin API Security:**
- Change the default API key (`changeme-supersecret`) immediately
- Store API key in environment variable, not in code
- Restrict `/admin/*` endpoints via CDN rules to admin IPs only
- Use HTTPS in production (handled by CDN/Fermyon Cloud)
- Consider adding additional authentication layers (OAuth, JWT, etc.)

**Why X-Forwarded-For Matters:**
- Without a reverse proxy, client IPs will be detected as "unknown"
- The bot trap is designed to work with `X-Forwarded-For` header for accurate IP detection
- Direct origin access bypasses CDN protection and may allow attackers to hide their real IP

---


## Usage

### As a Site Owner
- Deploy the app to your edge environment (Fermyon Cloud or compatible platform).
- Configure honeypot URLs, rate limits, browser blocklist, geo risk countries, and whitelist via the admin API.
- Monitor and manage bans and analytics via the admin API.


### Endpoints: Browser & Curl Access

#### Browser-accessible pages

- `http://127.0.0.1:3000/`  
	Main entry: triggers bot trap logic. You may see the block page, math quiz, or JS challenge depending on your status.
- `http://127.0.0.1:3000/bot-trap`  
	Honeypot: triggers a ban and then shows the block page.
- `http://127.0.0.1:3000/quiz`  
	Math quiz page (if enabled for non-blocked users).
- `http://127.0.0.1:3000/admin`  
	Admin API help page (most actions require Authorization header).

#### Curl-accessible endpoints

- Health check (requires header):
	```sh
	curl -H "X-Forwarded-For: 127.0.0.1" http://127.0.0.1:3000/health
	```
- Root endpoint (simulate IP):
	```sh
	curl -H "X-Forwarded-For: 1.2.3.4" http://127.0.0.1:3000/
	```
- Honeypot (simulate ban):
	```sh
	curl -H "X-Forwarded-For: 1.2.3.4" http://127.0.0.1:3000/bot-trap
	```
- Admin unban (requires Authorization header):
	```sh
	curl -H "Authorization: Bearer changeme-supersecret" "http://127.0.0.1:3000/admin/unban?ip=1.2.3.4"
	```
- Admin analytics/events (requires Authorization header):
	```sh
	curl -H "Authorization: Bearer changeme-supersecret" http://127.0.0.1:3000/admin/analytics
	curl -H "Authorization: Bearer changeme-supersecret" http://127.0.0.1:3000/admin/events
	```
- Admin ban IP (requires Authorization header):
  ```sh
  curl -X POST -H "Authorization: Bearer changeme-supersecret" -H "Content-Type: application/json" \
    -d '{"ip":"1.2.3.4","reason":"admin_ban","duration":21600}' \
    http://127.0.0.1:3000/admin/ban
  ```
All endpoints require an `Authorization: Bearer <API_KEY>` header. The API key is configurable via the `API_KEY` environment variable (see below).

#### Endpoints

- `GET /admin/ban` — List all current bans (JSON: IP, reason, expiry)
- `POST /admin/ban` — Manually ban an IP (JSON body: `{"ip": "1.2.3.4", "reason": "admin_ban", "duration": 21600}`)
- `POST /admin/unban?ip=...` — Unban a specific IP (removes ban immediately)
- `GET /admin/analytics` — Get ban count analytics
- `GET /admin/events?hours=N` — Query recent security events, top IPs, and event statistics for dashboarding (see below)
- `GET /admin` — Usage help

#### `/admin/events` — Activity & Analytics API

Returns a JSON object with:

- `recent_events`: up to 100 most recent events (ban, unban, challenge, block, admin actions)
- `event_counts`: count of each event type in the time window
- `top_ips`: top 10 IPs by event count

Query params:
- `hours=N` — How many hours of history to include (default: 24)

**Example response:**
```json
{
	"recent_events": [
		{
			"ts": 1769577600,
			"event": "Ban",
			"ip": "203.0.113.42",
			"reason": "honeypot",
			"outcome": "banned",
			"admin": null
		},
		{
			"ts": 1769577700,
			"event": "Unban",
			"ip": "203.0.113.42",
			"reason": "admin_unban",
			"outcome": "unbanned",
			"admin": "admin"
		}
		// ...
	],
	"event_counts": {
		"Ban": 12,
		"Unban": 2,
		"Challenge": 8,
		"Block": 0,
		"AdminAction": 5
	},
	"top_ips": [
		["203.0.113.42", 7],
		["198.51.100.10", 3]
	]
}
```

**Example curl queries:**

List all bans:
```sh
curl -s -H "Authorization: Bearer $API_KEY" http://localhost:3000/admin/ban | jq
```

Manually ban an IP:
```sh
curl -X POST -H "Authorization: Bearer $API_KEY" -H "Content-Type: application/json" \
  -d '{"ip":"203.0.113.42","reason":"suspicious_activity","duration":3600}' \
  http://localhost:3000/admin/ban | jq
```

Unban an IP:
```sh
curl -X POST -H "Authorization: Bearer $API_KEY" "http://localhost:3000/admin/unban?ip=203.0.113.42"
```

Get ban count analytics:
```sh
curl -s -H "Authorization: Bearer $API_KEY" http://localhost:3000/admin/analytics | jq
```

Get recent events and statistics (last 24 hours):
```sh
curl -s -H "Authorization: Bearer $API_KEY" "http://localhost:3000/admin/events?hours=24" | jq
```

Get API usage/help:
```sh
curl -s -H "Authorization: Bearer $API_KEY" http://localhost:3000/admin
```

#### API Key Configuration
- The admin API key is set via the `API_KEY` environment variable in your Spin manifest or deployment environment. If not set, it defaults to `changeme-supersecret` for development.
- Example (in `spin.toml`):
	```toml
	[component.bot-trap]
	environment = { API_KEY = "changeme-supersecret" }
	```

### Prometheus Metrics Endpoint

The bot trap exports Prometheus-compatible metrics for integration with Grafana, Datadog, or any monitoring system that supports the Prometheus text format.

#### Endpoint

```
GET /metrics
```

This endpoint requires no authentication (for Prometheus scraper compatibility) and returns metrics in Prometheus text format.

#### Available Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `bot_trap_requests_total` | Counter | Total requests processed |
| `bot_trap_bans_total{reason="..."}` | Counter | Bans issued by reason (honeypot, rate_limit, browser, admin) |
| `bot_trap_blocks_total` | Counter | Requests blocked (403 responses) |
| `bot_trap_challenges_total` | Counter | JS challenges served |
| `bot_trap_whitelisted_total` | Counter | Requests bypassed via whitelist |
| `bot_trap_test_mode_actions_total` | Counter | Actions logged in test mode |
| `bot_trap_active_bans` | Gauge | Current number of active bans |
| `bot_trap_test_mode_enabled` | Gauge | Whether test mode is enabled (0/1) |

#### Example Response

```
# HELP bot_trap_requests_total Total requests processed by the bot trap
# TYPE bot_trap_requests_total counter
bot_trap_requests_total 1523

# HELP bot_trap_bans_total Total bans issued
# TYPE bot_trap_bans_total counter
bot_trap_bans_total{reason="honeypot"} 42
bot_trap_bans_total{reason="rate_limit"} 18
bot_trap_bans_total{reason="browser"} 5
bot_trap_bans_total{reason="admin"} 3

# HELP bot_trap_blocks_total Total requests blocked
# TYPE bot_trap_blocks_total counter
bot_trap_blocks_total 68

# HELP bot_trap_active_bans Current number of active bans
# TYPE bot_trap_active_bans gauge
bot_trap_active_bans 12

# HELP bot_trap_test_mode_enabled Whether test mode is enabled
# TYPE bot_trap_test_mode_enabled gauge
bot_trap_test_mode_enabled 0
```

#### Grafana Integration

Add the bot trap as a Prometheus scrape target:

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'wasm-bot-trap'
    static_configs:
      - targets: ['your-bot-trap-domain:3000']
    metrics_path: /metrics
    scrape_interval: 15s
```

Create Grafana dashboards to visualize:
- Request rate and block rate over time
- Ban reasons breakdown (pie chart)
- Active bans trend
- Test mode status


### Interactive Quiz for Banned Users

**NOTE:** The interactive quiz feature is currently disabled. Banned users now see a block page directly instead of a quiz. The quiz code is preserved in `src/quiz.rs` for potential future use if quiz-on-ban is re-enabled.

When re-enabled, the quiz would provide:
- **Randomized question types**: Addition, subtraction, and multiplication
- **User-friendly HTML**: Styled, accessible, and mobile-friendly
- **Automatic unban**: Correct answer removes the ban and restores access
- **Security**: Quiz answers are stored securely per IP

This feature helps reduce false positives and allows legitimate users to regain access easily.

### Web Dashboard

A web-based admin dashboard is available in the `dashboard/` directory with:
- **Real-time analytics**: Ban counts, event statistics, top IPs
- **Admin controls**: Ban/unban IPs directly from the UI
- **Event log viewer**: Browse recent security events with filtering
- **API key authentication**: Secure access to admin functions

To use the dashboard:
1. Open `dashboard/index.html` in your browser
2. Enter your API endpoint (default: `http://127.0.0.1:3000`)
3. Enter your API key (default: `changeme-supersecret`)
4. View analytics and manage bans

**Note:** Static file serving for the dashboard at `/dashboard/*` is configured in `spin_static_dashboard.toml` but may require additional Spin configuration depending on your version.


### Configuration
- Ban duration, rate limit, honeypot URLs, browser blocklist, **browser whitelist for JS challenge bypass**, geo risk, whitelist (with CIDR and comments), path-based whitelist for integrations/webhooks, and test mode are stored in edge KV and can be managed via future admin endpoints or direct KV updates.

#### Whitelist Features
- **IP/CIDR support:** Whitelist entries can be single IPs (e.g., `1.2.3.4`) or CIDR ranges (e.g., `192.168.0.0/24`).
- **Inline comments:** Entries can include comments after a `#` (e.g., `10.0.0.0/8 # corp network`).
- **Path-based whitelisting:** The `path_whitelist` config allows you to specify exact paths (e.g., `/webhook/stripe`) or wildcard prefixes (e.g., `/api/integration/*`) that should always bypass bot protections. Useful for trusted webhooks and integrations.


#### Browser Whitelist for JS Challenge Bypass

You can specify browsers (by name and minimum version) that should bypass the JS challenge. This is useful for trusted automation, monitoring, or integrations that cannot solve JS challenges.

Add a `browser_whitelist` array to your config, e.g.:
```json
{
	"browser_whitelist": [
		["Chrome", 120],
		["MyAutomationBot", 1]
	],
	"whitelist": ["1.2.3.4", "192.168.0.0/24 # office", "10.0.0.0/8 # corp"],
	"path_whitelist": ["/webhook/stripe", "/api/integration/* # trusted integrations"]
}
```

- Each entry is `[browser_name, min_version]`.
- If the User-Agent matches and version is >= min_version, the JS challenge is skipped for that request.
- Example: Chrome/120+ or any version of MyAutomationBot will bypass JS challenge.

#### Test Mode (Safe Deployment/Tuning)

Test mode allows you to safely deploy and tune the bot trap in production without impacting real users. When enabled, all block/ban/challenge actions are logged but not enforced—users are always allowed through. This is ideal for initial rollout, tuning, and validation.

**How to enable (choose one):**

1. **Via Dashboard** (recommended): Use the Test Mode toggle in the Admin Controls section
2. **Via API**: 
   ```bash
   curl -X POST -H "Authorization: Bearer YOUR_API_KEY" \
     -H "Content-Type: application/json" \
     -d '{"test_mode": true}' \
     http://127.0.0.1:3000/admin/config
   ```
3. **Via environment variable** (requires restart):
   ```toml
   [component.bot-trap]
   environment = { TEST_MODE = "1" }
   ```
4. **Via KV store**: Set `"test_mode": true` in the config object

**When enabled:**
- All actions (ban, block, challenge) are logged with a `[TEST MODE]` prefix in event log
- No user is actually blocked, banned, or challenged
- Dashboard shows prominent warning banner
- Events appear in the event log with `[TEST MODE]` suffix for easy filtering
- Useful for safe validation and tuning in production

**Check current status:**
```bash
curl -H "Authorization: Bearer YOUR_API_KEY" http://127.0.0.1:3000/admin/config
# Returns: {"test_mode": true/false, "ban_durations": {...}, ...}
```

**Disable test mode** via dashboard toggle or API to enforce real blocking/ban logic.

---

### Configurable Ban Durations

Different offense types can have different ban durations. This allows proportional responses: honeypot access (severe) gets longer bans than rate limiting (temporary).

**Default durations:**
| Ban Type | Duration | Description |
|----------|----------|-------------|
| `honeypot` | 24 hours (86400s) | Accessing trap URLs - severe offense |
| `rate_limit` | 1 hour (3600s) | Exceeding rate limits - temporary |
| `browser` | 6 hours (21600s) | Outdated/suspicious browser |
| `admin` | 6 hours (21600s) | Default for manual bans |

**Configure via Dashboard:**
Use the "Ban Durations" panel in Admin Controls to adjust per-type durations.

**Configure via API:**
```bash
curl -X POST -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"ban_durations": {"honeypot": 172800, "rate_limit": 1800}}' \
  http://127.0.0.1:3000/admin/config
```

---



## Testing

**⚠️ Three Test Layers for Complete Coverage**

This project uses three distinct test environments, each optimized for its purpose:

### Quick Test Commands

```sh
# Run all backend tests (unit + integration)
make test

# Run individual test suites
make test-unit          # Rust unit tests (13 tests)
make test-integration   # Spin integration tests (5 scenarios)
make test-dashboard     # Dashboard testing instructions
```

### 1. Unit Tests (Native Rust - 13 tests)

Fast, isolated logic testing:

```sh
cargo test              # Run all unit tests
cargo test ban          # Run ban-related tests
cargo test whitelist    # Run whitelist tests
```

**Tests:** IP banning, whitelist matching, quiz generation, helper functions

### 2. Integration Tests (Spin Environment - 5 scenarios)

Full HTTP/KV stack testing:

```sh
# Terminal 1: Start server
spin up

# Terminal 2: Run tests
./test_spin_colored.sh
```

**Tests:** Health endpoint, honeypot, admin ban/unban, root endpoint

### 3. Dashboard Tests (Browser - Manual)

UI and visualization testing:

```sh
# Start server and open dashboard
make local
open http://127.0.0.1:3000/dashboard/index.html
```

**Tests:** Charts, forms, API integration, user interactions

See [TESTING.md](TESTING.md) for complete testing guide with checklist.
```sh
make local    # Runs spin up in background, then test_spin_colored.sh
```

**Integration tests cover:**
- Health check endpoint (GET /health)
- Root endpoint behavior (GET /)
- Honeypot ban detection (POST /_wp-admin.php)
- Admin API ban/unban (POST /admin/ban, POST /admin/unban)
- End-to-end ban/unban flow

**Integration tests = 5 scenarios** (run in Spin environment ONLY)

### Run All Tests

```sh
./test_all_colored.sh    # Runs BOTH unit tests (cargo test) + integration tests (shell script)
make test                # Same as above
```

### Why Two Environments?

- **Unit tests** run in native Rust and test logic in isolation
- **Integration tests MUST run in Spin** because they require:
  - HTTP server and routing
  - Spin key-value store
  - Real HTTP headers (cookies, user-agent, x-forwarded-for)
  - Authentication and API endpoints

**⚠️ Important:** The `tests/bot_trap.rs` file exists only to prevent cargo warnings. It does NOT contain real integration tests. All integration tests are in `test_spin_colored.sh`.

### Build System Notes

All test scripts automatically run `cargo clean` before building or testing. This ensures the correct crate-type is set for each build mode (native or WASM), preventing build/test errors due to stale build artifacts.

**How crate-type switching works:**
- When building for native (unit tests), the crate-type is set to `["rlib"]`
- When building for WASM (Spin), the crate-type is set to `["cdylib"]`
- This is handled automatically by `build.rs` based on the build target

If you see errors about missing crates or WASM output, ensure you are using the provided scripts or run `cargo clean` before switching build modes.

### Manual Testing: Replicating Integration Tests in Browser

**⚠️ IMPORTANT: IP Detection When Testing Locally**

When accessing the bot trap from your local browser or curl without headers, your IP is detected as **"unknown"** (because there's no X-Forwarded-For header). This means:
- If you get banned, you need to unban IP: **"unknown"**
- In production with a reverse proxy/CDN, real client IPs are properly detected

**Before starting:** Ensure Spin is running in background:
```bash
cd /Users/jtindall/SOASTA_MP/wasm_bot_trap
nohup spin up --listen 127.0.0.1:3000 > /tmp/spin_bot_trap.log 2>&1 &
# Verify it's running:
curl http://127.0.0.1:3000/health
# Should return: OK
```

#### Test 1: Health Check Endpoint ✅
**Integration Test Equivalent:** `GET /health`

**Browser Steps:**
1. Open: http://127.0.0.1:3000/health
2. **Expected:** Plain text "OK" response
3. **Confirms:** Basic server functionality

**curl Command:**
```bash
curl http://127.0.0.1:3000/health
```

#### Test 2: Root Endpoint - JS Challenge ✅
**Integration Test Equivalent:** `GET /` (first visit without js_verified cookie)

**Browser Steps:**
1. Open browser in **Incognito/Private mode** (or clear cookies for 127.0.0.1)
2. Open Developer Tools (F12) → Network tab
3. Visit: http://127.0.0.1:3000/
4. **Expected:** 
   - Page with JavaScript that sets `js_verified` cookie
   - Auto-reload after cookie is set
   - After reload, you should pass through successfully
5. **Confirms:** JS challenge injection works

**curl Command:**
```bash
curl -v http://127.0.0.1:3000/
# Look for: js_verified cookie and JavaScript code
```

#### Test 3: Honeypot Ban Detection ✅
**Integration Test Equivalent:** `POST /bot-trap` then verify ban

**Browser Steps:**
1. Visit: http://127.0.0.1:3000/bot-trap
2. **Expected:** "Access Blocked" page with ban message
3. Now visit: http://127.0.0.1:3000/
4. **Expected:** Still see "Access Blocked" page (you're banned)
5. **To unban yourself (your IP is "unknown" locally):**
   ```bash
   curl -X POST -H "Authorization: Bearer changeme-supersecret" \
     "http://127.0.0.1:3000/admin/unban?ip=unknown"
   ```
6. Refresh http://127.0.0.1:3000/ - should now work again
7. **Confirms:** Honeypot triggers ban correctly and unban restores access

**curl Commands:**
```bash
# Trigger honeypot ban for a specific IP
curl -H "X-Forwarded-For: 1.2.3.4" http://127.0.0.1:3000/bot-trap

# Verify that IP is banned
curl -H "X-Forwarded-For: 1.2.3.4" http://127.0.0.1:3000/
# Should see "Access Blocked"

# Unban that specific IP
curl -X POST -H "Authorization: Bearer changeme-supersecret" \
  "http://127.0.0.1:3000/admin/unban?ip=1.2.3.4"
```

#### Test 4: Admin API - Manual Ban ✅
**Integration Test Equivalent:** `POST /admin/ban`

**Browser Steps (using curl):**
```bash
# Ban an IP address
curl -X POST -H "Authorization: Bearer changeme-supersecret" \
  -H "Content-Type: application/json" \
  -d '{"ip":"10.20.30.40","reason":"manual_test","duration":3600}' \
  http://127.0.0.1:3000/admin/ban

# Verify ban by listing all bans
curl -H "Authorization: Bearer changeme-supersecret" \
  http://127.0.0.1:3000/admin/ban | jq

# Try accessing from banned IP (should be blocked)
curl -H "X-Forwarded-For: 10.20.30.40" http://127.0.0.1:3000/
```

**Using the Web Dashboard:**
1. Open: http://127.0.0.1:3000/dashboard/index.html (if static serving enabled)
2. Enter API Key: `changeme-supersecret`
3. Enter API URL: `http://127.0.0.1:3000`
4. Click "Load Analytics"
5. In the "Ban IP" section, enter IP: `10.20.30.40`
6. Click "Ban IP" button
7. **Expected:** See ban confirmation, IP appears in ban list

#### Test 5: Admin API - Unban ✅
**Integration Test Equivalent:** `POST /admin/unban`

**Browser Steps (using curl):**
```bash
# Unban a specific IP
curl -X POST -H "Authorization: Bearer changeme-supersecret" \
  "http://127.0.0.1:3000/admin/unban?ip=1.2.3.4"

# Verify access restored (should see JS challenge, not ban page)
curl -H "X-Forwarded-For: 1.2.3.4" http://127.0.0.1:3000/
```

**Using the Web Dashboard:**
1. In the dashboard, view the ban list
2. Find the IP you want to unban
3. Click "Unban" button next to that IP
4. **Expected:** IP removed from ban list, access restored

#### Complete Manual Test Sequence
Run all 5 tests in order to fully replicate the integration test suite:

```bash
# Start Spin in background
nohup spin up --listen 127.0.0.1:3000 > /tmp/spin_bot_trap.log 2>&1 &
sleep 2

# Test 1: Health
echo "Test 1: Health Check"
curl http://127.0.0.1:3000/health
echo ""

# Test 2: Root (JS Challenge)
echo "Test 2: JS Challenge"
curl -s http://127.0.0.1:3000/ | grep -o "js_verified"
echo ""

# Test 3: Honeypot Ban (with proper X-Forwarded-For header)
echo "Test 3: Honeypot Ban"
curl -s -H "X-Forwarded-For: 1.2.3.4" http://127.0.0.1:3000/bot-trap > /dev/null
curl -s -H "X-Forwarded-For: 1.2.3.4" http://127.0.0.1:3000/ | grep -o "Access Blocked"
echo ""

# Test 4: Admin Ban
echo "Test 4: Admin Ban"
curl -X POST -H "Authorization: Bearer changeme-supersecret" \
  -H "Content-Type: application/json" \
  -d '{"ip":"10.20.30.40","reason":"test","duration":3600}' \
  http://127.0.0.1:3000/admin/ban
echo ""

# Test 5: Admin Unban  
echo "Test 5: Admin Unban"
curl -X POST -H "Authorization: Bearer changeme-supersecret" \
  "http://127.0.0.1:3000/admin/unban?ip=1.2.3.4"
curl -s -H "X-Forwarded-For: 1.2.3.4" http://127.0.0.1:3000/ | head -5
echo ""
```

#### Troubleshooting Local Testing

**Problem:** "I visited /bot-trap in my browser and got banned, but unban doesn't work"  
**Solution:** When testing locally without X-Forwarded-For header, your IP is "unknown". Unban with:
```bash
curl -X POST -H "Authorization: Bearer changeme-supersecret" \
  "http://127.0.0.1:3000/admin/unban?ip=unknown"
```

**Problem:** "Health endpoint returns Forbidden"  
**Solution:** This was fixed. Make sure you rebuild and restart Spin:
```bash
cargo build --target wasm32-wasip1 --release
cp target/wasm32-wasip1/release/wasm_bot_trap.wasm src/bot_trap.wasm
pkill -f "spin up"
spin up
```

**Problem:** "How do I see what IP the bot trap detected for me?"  
**Solution:** Check the ban list:
```bash
curl -H "Authorization: Bearer changeme-supersecret" \
  http://127.0.0.1:3000/admin/ban | jq
```
Look for your IP in the ban list - it's likely "unknown" if testing locally.

Beyond the integration tests, you can also manually test:

1. **Whitelist**: Add your IP to the whitelist config via admin API
2. **Rate Limit**: Send many requests quickly with a script to trigger auto-ban
3. **Outdated Browser**: Use custom User-Agent header (e.g., `Chrome/50`)
4. **Geo Risk**: Set `X-Geo-Country` header to a blocked country code

**Tip:** Use browser Developer Tools (F12) to inspect:
- Network tab: See headers, cookies, redirects
- Application tab: View cookies (look for `js_verified`)
- Console: Check for JavaScript errors

---

- Modular Rust code: see `src/` for ban, rate, JS, browser, geo, whitelist, honeypot, admin, and interactive quiz logic.
- Integration test script: see `test_spin_colored.sh` for automated end-to-end tests.
- Unit tests: see `src/ban_tests.rs` for ban logic tests.
- Logging: Security events and ban actions are logged using Spin's logging macros.
- Performance: Early returns, minimal KV access, lightweight parsing, and optimized WASM build.

## Performance Checklist
- Early returns: Whitelist and ban checks short-circuit further logic
- Minimal key-value store reads/writes per request
- Lightweight header/cookie parsing
- Fixed time window for rate limiting
- No large in-memory state; all persistent state in edge KV
- Build with `--release` for optimized WASM

---

## Architecture & Code Quality

### Project Structure
```
src/
├── lib.rs              # Main handler and request routing
├── admin.rs            # Admin API endpoints (/admin/*)
├── auth.rs             # API key authentication with constant-time comparison
├── ban.rs              # Ban management (ban_ip, unban_ip, is_banned)
├── block_page.rs       # HTML block page templates
├── browser.rs          # Browser version detection
├── config.rs           # Configuration loading and defaults
├── geo.rs              # Geolocation-based risk detection
├── honeypot.rs         # Honeypot URL detection
├── js.rs               # JavaScript challenge injection
├── metrics.rs          # Prometheus metrics endpoint
├── quiz.rs             # Math quiz (currently unused)
├── rate.rs             # Rate limiting with time windows
├── whitelist.rs        # IP/CIDR and path whitelisting
├── *_tests.rs          # Unit tests for each module
└── bot_trap.wasm       # Compiled WASM binary
```

### Design Principles
- **Early returns**: Whitelist and ban checks short-circuit further processing
- **Minimal KV access**: Efficient key-value operations with caching where appropriate
- **Modular design**: Each feature is isolated in its own module with clear interfaces
- **Testability**: Trait-based abstractions (KeyValueStore) allow easy unit testing
- **Security-first**: Constant-time auth comparison, input validation, sanitization

### Security Hardening
- **API key authentication**: All admin endpoints require Bearer token
- **Constant-time comparison**: Prevents timing attacks on API key validation
- **Input validation**: Path sanitization prevents traversal attacks
- **HMAC token verification**: JS challenge tokens are cryptographically signed
- **Rate limiting**: Automatic ban on rate limit exceed
- **Event logging**: All admin actions and security events are logged

### Performance Considerations
- **WASM optimization**: Built with `--release` for production
- **Fixed time windows**: Rate limiting uses 1-minute buckets for efficiency
- **No large state**: All persistent data in edge KV store
- **Minimal allocations**: String operations optimized where possible

---

## Security
- All admin endpoints require API key authentication.
- Input validation and sanitization for all admin operations.
- JS challenge uses a secure, tamper-proof token tied to the visitor's IP.

---

## Roadmap

### ✅ Recently Completed
- **Prometheus/OpenTelemetry Metrics**: `/metrics` endpoint for Grafana integration
- **Test Mode (Dry-Run)**: Safe deployment/tuning without blocking users
- **Configurable Ban Durations**: Per-ban-type TTLs (honeypot, rate_limit, browser, admin)
- **Browser Version Detection**: Flag outdated browsers as suspicious
- **One-Command Setup**: `make setup` installs all dependencies automatically
- **File Watching**: `make dev` auto-rebuilds on save via cargo-watch

### Near-term
- Tarpit mode: Optional slow infinite responses to waste bot resources
- Link maze honeypot: Fake link trees that trap crawlers in infinite loops
- Webhook notifications: Slack/Discord/PagerDuty alerts for bans
- Dual GeoIP providers: Fallback when primary fails (MaxMind + IP-API)
- Expand admin API for full configuration management (update config via API)
- Add CSV/JSON export for event logs and analytics
- Integrate with additional edge geo/IP intelligence sources
- Add more unit and integration tests for edge cases
- Implement re-enable option for quiz-on-ban feature

### Agentic AI & Modern Threat Detection
The coming world of agentic swarms presents new challenges for bot detection. Future enhancements to prepare:

#### Behavioral Analysis
- **Request pattern fingerprinting**: Detect repetitive, mechanical patterns typical of AI agents
- **Timing analysis**: Identify non-human response times and consistency patterns
- **Session behavior tracking**: Monitor navigation patterns, mouse/touch events, scroll behavior
- **API abuse detection**: Track GraphQL/REST API usage patterns characteristic of automated agents

#### AI Agent Detection
- **LLM fingerprinting**: Detect common AI agent user-agent patterns and headers
- **Tool usage detection**: Identify requests from known AI agent frameworks (LangChain, AutoGPT, etc.)
- **Capability probing**: Detect agents testing endpoints systematically
- **Context window analysis**: Monitor for requests that suggest large context window processing

#### Adaptive Defense
- **ML-based anomaly detection**: Train models on legitimate traffic patterns
- **Dynamic challenge escalation**: Increase challenge difficulty for suspicious patterns
- **Swarm coordination detection**: Identify coordinated attacks from distributed agent networks
- **Rate limit adaptation**: Dynamically adjust limits based on threat level

#### Integration & Intelligence
- **Threat intelligence feeds**: Integrate with known bad actor databases
- **Reputation scoring**: Build IP/user-agent reputation over time
- **Cross-site intelligence**: Share threat data across multiple protected sites
- **API for external ML models**: Allow custom detection models to plug in

#### Privacy-Preserving Verification
- **Zero-knowledge proofs**: Verify humanity without revealing identity
- **Attestation protocols**: Integrate with device/browser attestation APIs
- **Decentralized identity**: Support DIDs and verifiable credentials

---

See `src/` for implementation details and extend as needed.
