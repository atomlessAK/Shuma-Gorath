mod block_page;
#[cfg(test)]
mod challenge_tests;
#[cfg(test)]
mod config_tests;
#[cfg(test)]
mod risk_tests;
#[cfg(test)]
mod ban_tests;
#[cfg(test)]
mod whitelist_tests;
#[cfg(test)]
mod whitelist_path_tests;
#[cfg(test)]
mod cdp_tests;
mod auth;
// src/lib.rs
// Entry point for the WASM Stealth Bot Trap Spin app

use spin_sdk::http::{Request, Response};
use spin_sdk::http_component;
use spin_sdk::key_value::Store;
use std::env;

mod ban;         // Ban logic (IP, expiry, reason)
mod config;      // Config loading and defaults
mod rate;        // Rate limiting
mod ip;          // IP bucketing helpers
mod js;          // JS challenge/verification
mod browser;     // Browser version checks
mod geo;         // Geo-based risk
mod whitelist;   // Whitelist logic
mod honeypot;    // Honeypot endpoint logic
mod admin;       // Admin API endpoints
mod challenge;   // Interactive math challenge for banned users
mod metrics;     // Prometheus metrics
mod maze;        // Link maze honeypot
mod robots;      // robots.txt generation
mod cdp;         // CDP (Chrome DevTools Protocol) automation detection
mod pow;         // Proof-of-work verification

/// Main HTTP handler for the bot trap. This function is invoked for every HTTP request.
/// It applies a series of anti-bot checks in order of cost and effectiveness, returning early on block/allow.



/// Returns true if forwarded IP headers should be trusted for this request.
/// If FORWARDED_IP_SECRET is set, require a matching X-Shuma-Forwarded-Secret header.
fn forwarded_ip_trusted(req: &Request) -> bool {
    match env::var("FORWARDED_IP_SECRET") {
        Ok(secret) => req
            .header("x-shuma-forwarded-secret")
            .and_then(|v| v.as_str())
            .map(|v| v == secret)
            .unwrap_or(false),
        Err(_) => true,
    }
}

/// Extract the best available client IP from the request.
pub(crate) fn extract_client_ip(req: &Request) -> String {
    // Prefer X-Forwarded-For (may be a comma-separated list) when trusted
    if forwarded_ip_trusted(req) {
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
    }
    // Fallback: remote_addr (Spin SDK may not expose this, but placeholder for future)
    // If available: req.remote_addr().unwrap_or("")

    // Last resort:
    "unknown".to_string()
}

/// Return the configured fail mode: "open" (default) or "closed".
fn shuma_fail_mode() -> String {
    env::var("SHUMA_FAIL_MODE").unwrap_or_else(|_| "open".to_string()).to_lowercase()
}

fn rate_proximity_score(rate_count: u32, rate_limit: u32) -> u8 {
    if rate_limit == 0 {
        return 0;
    }
    let ratio = rate_count as f32 / rate_limit as f32;
    if ratio >= 0.8 {
        2
    } else if ratio >= 0.5 {
        1
    } else {
        0
    }
}

pub(crate) fn compute_risk_score(js_needed: bool, geo_risk: bool, rate_count: u32, rate_limit: u32) -> u8 {
    let mut score = 0u8;
    if js_needed {
        score += 1;
    }
    if geo_risk {
        score += 2;
    }
    score += rate_proximity_score(rate_count, rate_limit);
    score
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
        let allowed = ["127.0.0.1", "::1"];
        let ip = extract_client_ip(req);
        if !allowed.contains(&ip.as_str()) {
            return Response::new(403, "Forbidden");
        }
        let mode = shuma_fail_mode();
        if let Ok(store) = Store::open_default() {
            let test_key = "health:test";
            let _ = store.set(test_key, b"ok");
            let ok = store.get(test_key).is_ok();
            let _ = store.delete(test_key);
            if ok {
                return Response::builder()
                    .status(200)
                    .header("X-KV-Status", "available")
                    .header("X-Shuma-Fail-Mode", mode.as_str())
                    .body("OK")
                    .build();
            }
        }
        println!("[KV OUTAGE] Key-value store unavailable; SHUMA_FAIL_MODE={}", mode);
        return Response::builder()
            .status(500)
            .header("X-KV-Status", "unavailable")
            .header("X-Shuma-Fail-Mode", mode.as_str())
            .body("Key-value store error")
            .build();
    }

    // Challenge POST handler
    if path == "/challenge" && *req.method() == spin_sdk::http::Method::Post {
        if let Ok(store) = Store::open_default() {
            return challenge::handle_challenge_submit(&store, req);
        }
        return Response::new(500, "Key-value store error");
    }
    if path == "/challenge" && *req.method() == spin_sdk::http::Method::Get {
        if let Ok(store) = Store::open_default() {
            let cfg = config::Config::load(&store, "default");
            return challenge::serve_challenge_page(req, cfg.test_mode);
        }
        return Response::new(500, "Key-value store error");
    }

    // CDP Report endpoint - receives automation detection reports from client-side JS
    if path == "/cdp-report" && *req.method() == spin_sdk::http::Method::Post {
        if let Ok(store) = Store::open_default() {
            return cdp::handle_cdp_report(&store, req);
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

    let site_id = "default";
    let ip = extract_client_ip(req);
    let ua = req.header("user-agent").map(|v| v.as_str().unwrap_or("")).unwrap_or("");

    // Admin API
    if path.starts_with("/admin") {
        return admin::handle_admin(req);
    }
    if store.is_none() {
        let mode = shuma_fail_mode();
        println!("[KV OUTAGE] Store unavailable during request handling; SHUMA_FAIL_MODE={}", mode);
        if mode == "closed" {
            return Response::builder()
                .status(500)
                .header("X-KV-Status", "unavailable")
                .header("X-Shuma-Fail-Mode", mode.as_str())
                .body("Key-value store error (fail-closed)")
                .build();
        }
        return Response::builder()
            .status(200)
            .header("X-KV-Status", "unavailable")
            .header("X-Shuma-Fail-Mode", mode.as_str())
            .body("OK (bot trap: store unavailable, all checks bypassed)")
            .build();
    }
    let store = store.as_ref().unwrap();

    let cfg = config::Config::load(store, site_id);

    // Link Maze Honeypot - trap bots in infinite loops (only if enabled)
    if maze::is_maze_path(path) {
        if !cfg.maze_enabled {
            return Response::new(404, "Not Found");
        }
        metrics::increment(store, metrics::MetricName::MazeHits, None);

        // Log maze access event
        crate::admin::log_event(store, &crate::admin::EventLogEntry {
            ts: crate::admin::now_ts(),
            event: crate::admin::EventType::Challenge,
            ip: Some(ip.clone()),
            reason: Some("maze_trap".to_string()),
            outcome: Some("maze_page_served".to_string()),
            admin: None,
        });

        // Check if this IP has hit too many maze pages (potential crawler)
        // Bucket the IP to reduce KV cardinality and avoid per-IP explosion
        let maze_bucket = crate::ip::bucket_ip(&ip);
        let maze_key = format!("maze_hits:{}", maze_bucket);
        let hits: u32 = store.get(&maze_key)
            .ok()
            .flatten()
            .and_then(|v| String::from_utf8(v).ok())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        // Increment maze hit counter for this IP
        let _ = store.set(&maze_key, (hits + 1).to_string().as_bytes());

        // If they've hit the threshold, they're definitely a bot - ban them
        if hits >= cfg.maze_auto_ban_threshold && cfg.maze_auto_ban {
            ban::ban_ip(store, "default", &ip, "maze_crawler", cfg.get_ban_duration("honeypot"));
            metrics::increment(store, metrics::MetricName::BansTotal, Some("maze_crawler"));
            crate::admin::log_event(store, &crate::admin::EventLogEntry {
                ts: crate::admin::now_ts(),
                event: crate::admin::EventType::Ban,
                ip: Some(ip.clone()),
                reason: Some("maze_crawler".to_string()),
                outcome: Some(format!("banned_after_{}_maze_pages", cfg.maze_auto_ban_threshold)),
                admin: None,
            });
        }

        return maze::handle_maze_request(path);
    }

    // Increment request counter
    metrics::increment(store, metrics::MetricName::RequestsTotal, None);

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
        if path.starts_with("/pow") {
            return Response::new(200, "TEST MODE: PoW bypassed");
        }
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
            println!("[TEST MODE] Would serve challenge to banned IP {ip}");
            metrics::increment(store, metrics::MetricName::TestModeActions, None);
            crate::admin::log_event(store, &crate::admin::EventLogEntry {
                ts: crate::admin::now_ts(),
                event: crate::admin::EventType::Block,
                ip: Some(ip.clone()),
                reason: Some("banned [TEST MODE]".to_string()),
                outcome: Some("would_serve_challenge".to_string()),
                admin: None,
            });
            return Response::new(200, "TEST MODE: Would serve challenge");
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
    // Ban: always show block page if banned (no challenge)
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
    // PoW endpoints (public, before JS verification)
    if path == "/pow" {
        if *req.method() != spin_sdk::http::Method::Get {
            return Response::new(405, "Method Not Allowed");
        }
        return pow::handle_pow_challenge(&ip, cfg.pow_difficulty, cfg.pow_ttl_seconds);
    }
    if path == "/pow/verify" {
        return pow::handle_pow_verify(req, &ip);
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
    // Compute risk score for potential step-up challenge
    let needs_js = path != "/health" && js::needs_js_verification_with_whitelist(req, store, site_id, &ip, &cfg.browser_whitelist);
    let geo_risk = geo::is_high_risk_geo(req, &cfg.geo_risk);
    let rate_usage = rate::current_rate_usage(store, site_id, &ip);
    let risk_score = compute_risk_score(needs_js, geo_risk, rate_usage, cfg.rate_limit);
    if risk_score >= cfg.challenge_risk_threshold {
        metrics::increment(store, metrics::MetricName::ChallengesTotal, None);
        crate::admin::log_event(store, &crate::admin::EventLogEntry {
            ts: crate::admin::now_ts(),
            event: crate::admin::EventType::Challenge,
            ip: Some(ip.clone()),
            reason: Some("risk_gate".to_string()),
            outcome: Some(format!("score={}", risk_score)),
            admin: None,
        });
        return challenge::render_challenge(req);
    }

    // JS verification (bypass for browser whitelist)
    if needs_js {
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
        return js::inject_js_challenge(&ip, cfg.pow_difficulty, cfg.pow_ttl_seconds);
    }
    // Geo-based escalation
    if geo_risk {
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
        return js::inject_js_challenge(&ip, cfg.pow_difficulty, cfg.pow_ttl_seconds);
    }
    Response::new(200, "OK (passed bot trap)")
}

#[http_component]
pub fn spin_entrypoint(req: Request) -> Response {
    handle_bot_trap_impl(&req)
}
