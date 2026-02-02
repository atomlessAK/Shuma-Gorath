# WASM Stealth Bot Trap (Fermyon Spin)

This project implements a customizable, behavior-based bot defense system designed for **Fermyon Spin**, running WebAssembly at the edge for ultra-low latency bot protection.

## 🚀 Primary Platform: Fermyon Cloud (Edge WASM)

This bot trap is **primarily built and tested for deployment on Fermyon Cloud**. Fermyon Spin enables serverless WebAssembly execution at the edge, providing:

- **Ultra-low latency**: WASM executes in microseconds at edge locations worldwide
- **Global distribution**: Automatic deployment across Fermyon's edge network
- **Serverless scale**: No infrastructure management, automatic scaling
- **Integrated KV storage**: Edge key-value store for ban lists, rate limits, and configuration

## Structure
- `src/`: Rust source code for the Spin app
- `spin.toml`: Spin app manifest (primary deployment config)
- `README.md`: Project overview and setup
- `.gitignore`: Standard ignores

---

## Quick Start (Local Development)

### Prerequisites
1. Install [Spin](https://developer.fermyon.com/spin/install)
2. Install Rust with wasm32-wasip1 target: `rustup target add wasm32-wasip1`

### Build and Run (Makefile)
The easiest way to build and run locally:
```sh
make local    # Build, clean, and start local server
make prod     # Build for production and start server
make clean    # Clean build artifacts
```

### Manual Build and Run
```sh
cargo build --target wasm32-wasip1 --release
cp target/wasm32-wasip1/release/wasm_bot_trap.wasm src/bot_trap.wasm
spin up --listen 127.0.0.1:3000
```

### Access Dashboard
Once running, open the monitoring dashboard at:
```
http://127.0.0.1:3000/dashboard/index.html
```

The dashboard provides:
- 📊 Real-time statistics (total bans, active bans, events, unique IPs)
- 📈 Visualizations (event types distribution, top IPs)
- 📋 Ban list with quick unban controls
- 🛠️ Admin controls (manual ban/unban)
- 🔄 Auto-refresh every 30 seconds

See [DASHBOARD.md](DASHBOARD.md) for complete dashboard documentation.

---

## 🎯 Primary Deployment: Fermyon Cloud

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

### 🤖 Integration with Akamai Bot Manager

**Akamai Bot Manager** is Akamai's enterprise-grade bot detection and mitigation solution that uses machine learning, behavioral analysis, and fingerprinting to identify and manage bot traffic. When deploying the WASM bot trap on Akamai infrastructure, you should understand how these two solutions can work together.

#### Understanding the Roles

| Feature | Akamai Bot Manager | WASM Bot Trap |
|---------|-------------------|---------------|
| **Detection Method** | ML/AI, fingerprinting, behavioral analysis | Rule-based, honeypots, rate limiting |
| **Bot Categories** | Known bots, unknown bots, bot scores | Binary (allow/block) with custom rules |
| **Management** | Akamai Control Center | Self-hosted admin API/dashboard |
| **Customization** | Policy-based configuration | Full code-level customization |
| **Cost Model** | Licensed per traffic volume | Open source, self-managed |

#### Recommended Architecture: Layered Defense

For maximum protection, use **Bot Manager as the first line of defense** with the WASM bot trap providing **application-specific protections**:

```
Internet → Akamai Edge
              ↓
        Bot Manager (Layer 1)
        - Known bot detection
        - Bot scoring & categorization
        - Fingerprinting
        - Behavioral analysis
              ↓
        EdgeWorkers / WASM Bot Trap (Layer 2)
        - Custom honeypots for your app
        - Application-specific rate limits
        - Business logic-based blocking
        - Custom challenge flows
              ↓
        Origin Application
```

#### When to Use Each Solution

**Use Akamai Bot Manager for:**
- Detecting known commercial bots (scrapers, SEO tools, etc.)
- Identifying sophisticated bots using ML-based behavioral analysis
- Managing bot traffic with granular policies (allow, block, throttle, challenge)
- Protecting against credential stuffing and account takeover (ATO)
- Compliance with bot management SLAs and reporting

**Use WASM Bot Trap for:**
- Custom honeypot URLs specific to your application
- Application-aware rate limiting (e.g., per-endpoint limits)
- Custom JavaScript challenges tailored to your site
- Rapid prototyping of new detection rules
- Protecting specific high-value endpoints with custom logic
- Cost-effective protection for smaller properties

**Use Both Together for:**
- Defense in depth with multiple detection layers
- Catching bots that evade one layer with the other
- Custom application logic that Bot Manager can't handle
- Extending Bot Manager with application-specific rules

#### Configuration: Bot Manager + WASM Bot Trap

**Step 1: Configure Bot Manager First**

In Akamai Control Center:
1. Enable Bot Manager on your property
2. Configure bot detection policies (e.g., block known bad bots)
3. Set up conditional actions based on bot score
4. Pass bot detection headers to origin/EdgeWorkers

**Step 2: Forward Bot Manager Headers to WASM Bot Trap**

Bot Manager can pass detection results via headers. Configure your property to forward:

```
Akamai-Bot-Score: 85
Akamai-Bot-Category: AUTOMATED_TOOL
Akamai-Bot-Action: MONITOR
```

**Step 3: Use Bot Manager Data in WASM Bot Trap (Future Enhancement)**

The WASM bot trap can be extended to read these headers and make decisions:

```rust
// Example: Trust Bot Manager's high-confidence detections
fn check_bot_manager_headers(req: &Request) -> Option<BotManagerResult> {
    let score = req.header("Akamai-Bot-Score")?.parse::<u8>().ok()?;
    let category = req.header("Akamai-Bot-Category")?;
    
    if score > 90 && category == "MALICIOUS_BOT" {
        return Some(BotManagerResult::Block);
    }
    None
}
```

#### When NOT to Integrate

You may choose to run them independently if:
- You want the WASM bot trap to catch what Bot Manager misses (no header sharing)
- Bot Manager is configured in monitor-only mode for analytics
- You're using different Akamai properties for different protection layers
- Your application has unique requirements that don't fit Bot Manager's model

#### Migration Path

If you're currently using only Bot Manager and want to add custom protections:
1. Deploy WASM bot trap in **test mode** (`TEST_MODE=1`) behind Bot Manager
2. Monitor logs to see what additional traffic would be blocked
3. Tune honeypot URLs and rate limits based on your application
4. Gradually enable blocking for specific rules
5. Use Bot Manager for broad protection, WASM bot trap for surgical precision

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

**How to enable:**
- Set the environment variable `TEST_MODE=1` or `TEST_MODE=true` in your deployment (e.g., in `spin.toml`):
	```toml
	[component.bot-trap]
	environment = { TEST_MODE = "1" }
	```
- Or set `"test_mode": true` in the config KV object.

**When enabled:**
- All actions (ban, block, challenge) are logged with a `[TEST MODE]` prefix
- No user is actually blocked, banned, or challenged
- Useful for safe validation and tuning in production

**Disable test mode** to enforce real blocking/ban logic.

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
make test-dashboard     # Dashboard manual tests (instructions)
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

### Near-term
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
