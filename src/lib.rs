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
    let cfg = config::Config::load(store, site_id);

    // Path-based whitelist (for webhooks/integrations)
    if whitelist::is_path_whitelisted(path, &cfg.path_whitelist) {
        return Response::new(200, "OK (path whitelisted)");
    }
    // IP/CIDR whitelist
    if whitelist::is_whitelisted(&ip, &cfg.whitelist) {
        return Response::new(200, "OK (whitelisted)");
    }
    // Test mode: log and allow all actions (no blocking, banning, or challenging)
    if cfg.test_mode {
        if honeypot::is_honeypot(path, &cfg.honeypots) {
            println!("[TEST MODE] Would ban IP {ip} for honeypot");
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
