mod block_page;
#[cfg(test)]
mod quiz_tests;
#[cfg(test)]
mod ban_tests;
#[cfg(test)]
mod whitelist_tests;
#[cfg(test)]
mod whitelist_path_tests;
mod auth;
// src/lib.rs
// Entry point for the WASM Stealth Bot Trap Spin app

use spin_sdk::http::{Request, Response};
use spin_sdk::http_component;
use spin_sdk::key_value::Store;

mod ban;         // Ban logic (IP, expiry, reason)
mod config;      // Config loading and defaults
mod rate;        // Rate limiting
mod js;          // JS challenge/verification
mod browser;     // Browser version checks
mod geo;         // Geo-based risk
mod whitelist;   // Whitelist logic
mod honeypot;    // Honeypot endpoint logic
mod admin;       // Admin API endpoints
mod quiz;        // Interactive math quiz for banned users
mod metrics;     // Prometheus metrics
mod maze;        // Link maze honeypot
mod robots;      // robots.txt generation
mod cdp;         // CDP (Chrome DevTools Protocol) automation detection

/// Main HTTP handler for the bot trap. This function is invoked for every HTTP request.
/// It applies a series of anti-bot checks in order of cost and effectiveness, returning early on block/allow.



/// Extract the best available client IP from the request.
fn extract_client_ip(req: &Request) -> String {
    // Prefer X-Forwarded-For (may be a comma-separated list)
    if let Some(h) = req.header("x-forwarded-for") {
        let val = h.as_str().unwrap_or("");
        // Take the first IP in the list
        if let Some(ip) = val.split(',').next() {
            let ip = ip.trim();
            if !ip.is_empty() && ip != "unknown" {
                return ip.to_string();
            }
        }
    }
    // Fallback: X-Real-IP
    if let Some(h) = req.header("x-real-ip") {
        let val = h.as_str().unwrap_or("");
        if !val.is_empty() && val != "unknown" {
            return val.to_string();
        }
    }
    // Fallback: remote_addr (Spin SDK may not expose this, but placeholder for future)
    // If available: req.remote_addr().unwrap_or("")

    // Last resort:
    "unknown".to_string()
}

/// Main handler logic, testable as a plain Rust function.
pub fn handle_bot_trap_impl(req: &Request) -> Response {
    let store = match Store::open_default() {
        Ok(s) => Some(s),
        Err(_e) => None,
    };
    let path = req.path();

    // Health check endpoint (accessible from localhost/browser)
    if path == "/health" {
        let allowed = ["127.0.0.1", "::1", "unknown"];
        let ip = extract_client_ip(req);
        if !allowed.contains(&ip.as_str()) {
            return Response::new(403, "Forbidden");
        }
        if let Ok(store) = Store::open_default() {
            let test_key = "health:test";
            let _ = store.set(test_key, b"ok");
            let ok = store.get(test_key).is_ok();
            let _ = store.delete(test_key);
            if ok {
                return Response::new(200, "OK");
            }
        }
        return Response::new(500, "Key-value store error");
    }

    // Quiz POST handler
    if path == "/quiz" && *req.method() == spin_sdk::http::Method::Post {
        if let Ok(store) = Store::open_default() {
            return quiz::handle_quiz_submit(&store, req);
        }
        return Response::new(500, "Key-value store error");
    }

    // Prometheus metrics endpoint
    if path == "/metrics" {
        if let Ok(store) = Store::open_default() {
            return metrics::handle_metrics(&store);
        }
        return Response::new(500, "Key-value store error");
    }

    // robots.txt - configurable AI crawler blocking
    if path == "/robots.txt" {
        if let Ok(store) = Store::open_default() {
            let cfg = config::Config::load(&store, "default");
            if cfg.robots_enabled {
                metrics::increment(&store, metrics::MetricName::RequestsTotal, Some("robots_txt"));
                let content = robots::generate_robots_txt(&cfg);
                let content_signal = robots::get_content_signal_header(&cfg);
                return Response::builder()
                    .status(200)
                    .header("Content-Type", "text/plain; charset=utf-8")
                    .header("Content-Signal", content_signal)
                    .header("Cache-Control", "public, max-age=3600")
                    .body(content)
                    .build();
            }
        }
        // If disabled or store error, return 404
        return Response::new(404, "Not Found");
    }

    // Link Maze Honeypot - trap bots in infinite loops
    if maze::is_maze_path(path) {
        // Get store to log event and track metrics
        if let Ok(store) = Store::open_default() {
            let ip = extract_client_ip(req);
            metrics::increment(&store, metrics::MetricName::MazeHits, None);
            
            // Log maze access event
            crate::admin::log_event(&store, &crate::admin::EventLogEntry {
                ts: crate::admin::now_ts(),
                event: crate::admin::EventType::Challenge,
                ip: Some(ip.clone()),
                reason: Some("maze_trap".to_string()),
                outcome: Some("maze_page_served".to_string()),
                admin: None,
            });
            
            // Check if this IP has hit too many maze pages (potential crawler)
            let maze_key = format!("maze_hits:{}", ip);
            let hits: u32 = store.get(&maze_key)
                .ok()
                .flatten()
                .and_then(|v| String::from_utf8(v).ok())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            
            // Increment maze hit counter for this IP
            let _ = store.set(&maze_key, (hits + 1).to_string().as_bytes());
            
            // If they've hit the threshold, they're definitely a bot - ban them
            let cfg = config::Config::load(&store, "default");
            if hits >= cfg.maze_auto_ban_threshold && cfg.maze_auto_ban {
                ban::ban_ip(&store, "default", &ip, "maze_crawler", cfg.get_ban_duration("honeypot"));
                metrics::increment(&store, metrics::MetricName::BansTotal, Some("maze_crawler"));
                crate::admin::log_event(&store, &crate::admin::EventLogEntry {
                    ts: crate::admin::now_ts(),
                    event: crate::admin::EventType::Ban,
                    ip: Some(ip.clone()),
                    reason: Some("maze_crawler".to_string()),
                    outcome: Some(format!("banned_after_{}_maze_pages", cfg.maze_auto_ban_threshold)),
                    admin: None,
                });
            }
        }
        
        return maze::handle_maze_request(path);
    }

    let site_id = "default";
    let ip = extract_client_ip(req);
    let ua = req.header("user-agent").map(|v| v.as_str().unwrap_or("")).unwrap_or("");

    // Admin API
    if path.starts_with("/admin") {
        return admin::handle_admin(req);
    }
    if store.is_none() {
        return Response::new(200, "OK (bot trap: store unavailable, all checks bypassed)");
    }
    let store = store.as_ref().unwrap();

    // Increment request counter
    metrics::increment(store, metrics::MetricName::RequestsTotal, None);

    let cfg = config::Config::load(store, site_id);

    // Path-based whitelist (for webhooks/integrations)
    if whitelist::is_path_whitelisted(path, &cfg.path_whitelist) {
        metrics::increment(store, metrics::MetricName::WhitelistedTotal, None);
        return Response::new(200, "OK (path whitelisted)");
    }
    // IP/CIDR whitelist
    if whitelist::is_whitelisted(&ip, &cfg.whitelist) {
        metrics::increment(store, metrics::MetricName::WhitelistedTotal, None);
        return Response::new(200, "OK (whitelisted)");
    }
    // Test mode: log and allow all actions (no blocking, banning, or challenging)
    if cfg.test_mode {
        if honeypot::is_honeypot(path, &cfg.honeypots) {
            println!("[TEST MODE] Would ban IP {ip} for honeypot");
            metrics::increment(store, metrics::MetricName::TestModeActions, None);
            crate::admin::log_event(store, &crate::admin::EventLogEntry {
                ts: crate::admin::now_ts(),
                event: crate::admin::EventType::Block,
                ip: Some(ip.clone()),
                reason: Some("honeypot [TEST MODE]".to_string()),
                outcome: Some("would_block".to_string()),
                admin: None,
            });
            return Response::new(200, "TEST MODE: Would block (honeypot)");
        }
        if !rate::check_rate_limit(store, site_id, &ip, cfg.rate_limit) {
            println!("[TEST MODE] Would ban IP {ip} for rate limit");
            metrics::increment(store, metrics::MetricName::TestModeActions, None);
            crate::admin::log_event(store, &crate::admin::EventLogEntry {
                ts: crate::admin::now_ts(),
                event: crate::admin::EventType::Block,
                ip: Some(ip.clone()),
                reason: Some("rate_limit [TEST MODE]".to_string()),
                outcome: Some("would_block".to_string()),
                admin: None,
            });
            return Response::new(200, "TEST MODE: Would block (rate limit)");
        }
        if ban::is_banned(store, site_id, &ip) {
            println!("[TEST MODE] Would serve quiz to banned IP {ip}");
            metrics::increment(store, metrics::MetricName::TestModeActions, None);
            crate::admin::log_event(store, &crate::admin::EventLogEntry {
                ts: crate::admin::now_ts(),
                event: crate::admin::EventType::Block,
                ip: Some(ip.clone()),
                reason: Some("banned [TEST MODE]".to_string()),
                outcome: Some("would_serve_quiz".to_string()),
                admin: None,
            });
            return Response::new(200, "TEST MODE: Would serve quiz");
        }
        if path != "/health" && js::needs_js_verification(req, store, site_id, &ip) {
            println!("[TEST MODE] Would inject JS challenge for IP {ip}");
            metrics::increment(store, metrics::MetricName::TestModeActions, None);
            crate::admin::log_event(store, &crate::admin::EventLogEntry {
                ts: crate::admin::now_ts(),
                event: crate::admin::EventType::Challenge,
                ip: Some(ip.clone()),
                reason: Some("js_verification [TEST MODE]".to_string()),
                outcome: Some("would_challenge".to_string()),
                admin: None,
            });
            return Response::new(200, "TEST MODE: Would inject JS challenge");
        }
        if browser::is_outdated_browser(ua, &cfg.browser_block) {
            println!("[TEST MODE] Would ban IP {ip} for outdated browser");
            metrics::increment(store, metrics::MetricName::TestModeActions, None);
            crate::admin::log_event(store, &crate::admin::EventLogEntry {
                ts: crate::admin::now_ts(),
                event: crate::admin::EventType::Block,
                ip: Some(ip.clone()),
                reason: Some("browser [TEST MODE]".to_string()),
                outcome: Some("would_block".to_string()),
                admin: None,
            });
            return Response::new(200, "TEST MODE: Would block (outdated browser)");
        }
        if geo::is_high_risk_geo(req, &cfg.geo_risk) {
            println!("[TEST MODE] Would inject JS challenge for geo-risk IP {ip}");
            metrics::increment(store, metrics::MetricName::TestModeActions, None);
            crate::admin::log_event(store, &crate::admin::EventLogEntry {
                ts: crate::admin::now_ts(),
                event: crate::admin::EventType::Challenge,
                ip: Some(ip.clone()),
                reason: Some("geo_risk [TEST MODE]".to_string()),
                outcome: Some("would_challenge".to_string()),
                admin: None,
            });
            return Response::new(200, "TEST MODE: Would inject JS challenge (geo-risk)");
        }
        return Response::new(200, "TEST MODE: Would allow (passed bot trap)");
    }
    // Honeypot: ban and hard block
    if honeypot::is_honeypot(path, &cfg.honeypots) {
        ban::ban_ip(store, site_id, &ip, "honeypot", cfg.get_ban_duration("honeypot"));
        metrics::increment(store, metrics::MetricName::BansTotal, Some("honeypot"));
        metrics::increment(store, metrics::MetricName::BlocksTotal, None);
        // Log ban event
        crate::admin::log_event(store, &crate::admin::EventLogEntry {
            ts: crate::admin::now_ts(),
            event: crate::admin::EventType::Ban,
            ip: Some(ip.clone()),
            reason: Some("honeypot".to_string()),
            outcome: Some("banned".to_string()),
            admin: None,
        });
        return Response::new(403, block_page::render_block_page(block_page::BlockReason::Honeypot));
    }
    // Rate limit: ban and hard block
    if !rate::check_rate_limit(store, site_id, &ip, cfg.rate_limit) {
        ban::ban_ip(store, site_id, &ip, "rate", cfg.get_ban_duration("rate"));
        metrics::increment(store, metrics::MetricName::BansTotal, Some("rate_limit"));
        metrics::increment(store, metrics::MetricName::BlocksTotal, None);
        // Log ban event
        crate::admin::log_event(store, &crate::admin::EventLogEntry {
            ts: crate::admin::now_ts(),
            event: crate::admin::EventType::Ban,
            ip: Some(ip.clone()),
            reason: Some("rate".to_string()),
            outcome: Some("banned".to_string()),
            admin: None,
        });
        return Response::new(429, block_page::render_block_page(block_page::BlockReason::RateLimit));
    }
    // Ban: always show block page if banned (no quiz)
    if ban::is_banned(store, site_id, &ip) {
        metrics::increment(store, metrics::MetricName::BlocksTotal, None);
        // Log block event
        crate::admin::log_event(store, &crate::admin::EventLogEntry {
            ts: crate::admin::now_ts(),
            event: crate::admin::EventType::Ban,
            ip: Some(ip.clone()),
            reason: Some("banned".to_string()),
            outcome: Some("block page".to_string()),
            admin: None,
        });
        return Response::new(403, block_page::render_block_page(block_page::BlockReason::Honeypot));
    }
    // JS verification (bypass for browser whitelist)
    if path != "/health" && js::needs_js_verification_with_whitelist(req, store, site_id, &ip, &cfg.browser_whitelist) {
        metrics::increment(store, metrics::MetricName::ChallengesTotal, None);
        // Log challenge event
        crate::admin::log_event(store, &crate::admin::EventLogEntry {
            ts: crate::admin::now_ts(),
            event: crate::admin::EventType::Challenge,
            ip: Some(ip.clone()),
            reason: Some("js_verification".to_string()),
            outcome: Some("js challenge".to_string()),
            admin: None,
        });
        return js::inject_js_challenge(&ip);
    }
    // Outdated browser
    if browser::is_outdated_browser(ua, &cfg.browser_block) {
        ban::ban_ip(store, site_id, &ip, "browser", cfg.get_ban_duration("browser"));
        metrics::increment(store, metrics::MetricName::BansTotal, Some("browser"));
        metrics::increment(store, metrics::MetricName::BlocksTotal, None);
        // Log ban event
        crate::admin::log_event(store, &crate::admin::EventLogEntry {
            ts: crate::admin::now_ts(),
            event: crate::admin::EventType::Ban,
            ip: Some(ip.clone()),
            reason: Some("browser".to_string()),
            outcome: Some("banned".to_string()),
            admin: None,
        });
        return Response::new(403, block_page::render_block_page(block_page::BlockReason::OutdatedBrowser));
    }
    // Geo-based escalation
    if geo::is_high_risk_geo(req, &cfg.geo_risk) {
        metrics::increment(store, metrics::MetricName::ChallengesTotal, None);
        // Log challenge event
        crate::admin::log_event(store, &crate::admin::EventLogEntry {
            ts: crate::admin::now_ts(),
            event: crate::admin::EventType::Challenge,
            ip: Some(ip.clone()),
            reason: Some("geo_risk".to_string()),
            outcome: Some("js challenge".to_string()),
            admin: None,
        });
        return js::inject_js_challenge(&ip);
    }
    Response::new(200, "OK (passed bot trap)")
}

#[http_component]
pub fn spin_entrypoint(req: Request) -> Response {
    handle_bot_trap_impl(&req)
}
