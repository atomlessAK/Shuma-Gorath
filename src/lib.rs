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

/// Main HTTP handler for the bot trap. This function is invoked for every HTTP request.
/// It applies a series of anti-bot checks in order of cost and effectiveness, returning early on block/allow.
#[http_component]

pub fn handle_bot_trap(req: Request) -> Response {
    // Try to open the default key-value store for persistent state (bans, rate, config, etc.)
    let store = match Store::open_default() {
        Ok(s) => Some(s),
        Err(e) => {
            // Log the error (if logging is available) and continue with safe defaults
            // eprintln!("[bottrap] Store unavailable: {e:?}");
            None
        }
    };
    let site_id = "default";
    let ip = req.header("x-forwarded-for").map(|v| v.as_str().unwrap_or("unknown")).unwrap_or("unknown");
    let path = req.path();
    let ua = req.header("user-agent").map(|v| v.as_str().unwrap_or("")).unwrap_or("");

    // --- Admin API: /admin endpoints ---
    if path.starts_with("/admin") {
        // Delegate to admin module for all admin endpoints
        return admin::handle_admin(&req);
    }

    // If store is unavailable, skip all KV-dependent logic and allow the request with a warning
    if store.is_none() {
        return Response::new(200, "OK (bot trap: store unavailable, all checks bypassed)");
    }
    let store = store.as_ref().unwrap();

    // Load config (from KV or defaults)
    let cfg = config::Config::load(store, site_id);

    // --- Bot trap logic: ordered by cost/likelihood ---

    // 1. Whitelist: allow known good IPs immediately (no KV access)
    if whitelist::is_whitelisted(ip, &cfg.whitelist) {
        return Response::new(200, "OK (whitelisted)");
    }
    // 2. Ban: block banned IPs (single KV read)
    if ban::is_banned(store, site_id, ip) {
        return Response::new(403, "Blocked: Banned");
    }
    // 3. Honeypot: ban if accessing honeypot endpoints (string match, then KV write)
    if honeypot::is_honeypot(path, &cfg.honeypots) {
        ban::ban_ip(store, site_id, ip, "honeypot", cfg.ban_duration);
        return Response::new(403, "Blocked: Honeypot");
    }
    // 4. Rate limiting: ban if exceeding allowed requests (KV read/write)
    if !rate::check_rate_limit(store, site_id, ip, cfg.rate_limit) {
        ban::ban_ip(store, site_id, ip, "rate", cfg.ban_duration);
        return Response::new(429, "Blocked: Rate limit");
    }
    // 5. JS verification: require JS challenge for suspicious clients (header/cookie parse)
    if js::needs_js_verification(&req, store, site_id, ip) {
        return js::inject_js_challenge(ip);
    }
    // 6. Outdated browser: ban if using old/unsupported browsers (user-agent parse)
    if browser::is_outdated_browser(ua, &cfg.browser_block) {
        ban::ban_ip(store, site_id, ip, "browser", cfg.ban_duration);
        return Response::new(403, "Blocked: Outdated browser");
    }
    // 7. Geo-based escalation: require JS challenge for high-risk geos (header parse)
    if geo::is_high_risk_geo(&req, &cfg.geo_risk) {
        return js::inject_js_challenge(ip);
    }

    // --- Passed all checks: allow request ---
    Response::new(200, "OK (passed bot trap)")
}
