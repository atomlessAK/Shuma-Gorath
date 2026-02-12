#![recursion_limit = "256"]

#[cfg(test)]
mod lib_tests;
#[cfg(test)]
mod test_support;
// src/lib.rs
// Entry point for the WASM Stealth Bot Defence Spin app

use crate::enforcement::{ban, block_page};
use crate::signals::{browser, cdp, geo, js, whitelist};
use serde::Serialize;
use spin_sdk::http::{Request, Response};
use spin_sdk::http_component;
use spin_sdk::key_value::Store;
use std::env;
use std::io::Write;

mod admin; // Admin API endpoints
mod boundaries; // Domain boundary adapters for future repo splits
mod challenge; // Interactive math challenge for banned users
mod config; // Config loading and defaults
mod enforcement; // Enforcement actions (ban, block page, honeypot, rate limiting)
mod input_validation;
mod maze; // maze crawler trap
mod metrics; // Prometheus metrics
mod pow; // Proof-of-work verification
mod runtime; // request-time orchestration helpers
mod robots; // robots.txt generation
mod signals; // Risk and identity signals (browser/CDP/GEO/IP/JS/whitelist)
mod test_mode; // test-mode routing behavior

/// Main HTTP handler for the bot defence. This function is invoked for every HTTP request.
/// It applies a series of anti-bot checks in order of cost and effectiveness, returning early on block/allow.

/// Returns true if forwarded IP headers should be trusted for this request.
/// If SHUMA_FORWARDED_IP_SECRET is set, require a matching X-Shuma-Forwarded-Secret header.
fn forwarded_ip_trusted(req: &Request) -> bool {
    match env::var("SHUMA_FORWARDED_IP_SECRET") {
        Ok(secret) if !secret.trim().is_empty() => req
            .header("x-shuma-forwarded-secret")
            .and_then(|v| v.as_str())
            .map(|v| v == secret)
            .unwrap_or(false),
        _ => false,
    }
}

fn forwarded_proto_is_https(req: &Request) -> bool {
    if !forwarded_ip_trusted(req) {
        return false;
    }

    if let Some(proto) = req.header("x-forwarded-proto").and_then(|v| v.as_str()) {
        let first = proto.split(',').next().unwrap_or("").trim();
        if first.eq_ignore_ascii_case("https") {
            return true;
        }
    }

    if let Some(forwarded) = req.header("forwarded").and_then(|v| v.as_str()) {
        for part in forwarded.split(',') {
            for segment in part.split(';') {
                let segment = segment.trim();
                let lower = segment.to_ascii_lowercase();
                let Some(value) = lower.strip_prefix("proto=") else {
                    continue;
                };
                let value = value.trim().trim_matches('"');
                if value.eq_ignore_ascii_case("https") {
                    return true;
                }
            }
        }
    }

    false
}

fn request_is_https(req: &Request) -> bool {
    if req.uri().trim_start().starts_with("https://") {
        return true;
    }
    forwarded_proto_is_https(req)
}

fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.bytes().zip(b.bytes()) {
        diff |= x ^ y;
    }
    diff == 0
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

/// Extract client IP for `/health` checks.
///
/// Security posture:
/// - Only trust forwarded headers when `forwarded_ip_trusted` is true.
/// - Reject multi-hop XFF chains for health checks to avoid accepting attacker-
///   supplied left-most values when an upstream proxy appends addresses.
fn extract_health_client_ip(req: &Request) -> String {
    if forwarded_ip_trusted(req) {
        if let Some(h) = req.header("x-forwarded-for") {
            let mut entries = h
                .as_str()
                .unwrap_or("")
                .split(',')
                .map(|ip| ip.trim())
                .filter(|ip| !ip.is_empty() && *ip != "unknown");

            if let Some(first) = entries.next() {
                if entries.next().is_some() {
                    return "unknown".to_string();
                }
                return first.to_string();
            }
        }

        if let Some(h) = req.header("x-real-ip") {
            let val = h.as_str().unwrap_or("").trim();
            if !val.is_empty() && val != "unknown" {
                return val.to_string();
            }
        }
    }

    "unknown".to_string()
}

fn health_secret_authorized(req: &Request) -> bool {
    let expected = match env::var("SHUMA_HEALTH_SECRET") {
        Ok(secret) => secret.trim().to_string(),
        Err(_) => return true,
    };
    if expected.is_empty() {
        return true;
    }

    let presented = req
        .header("x-shuma-health-secret")
        .and_then(|v| v.as_str())
        .map(|v| v.trim())
        .unwrap_or("");

    constant_time_eq(presented, expected.as_str())
}

/// Return true when KV outage policy is fail-open.
fn shuma_fail_open() -> bool {
    config::kv_store_fail_open()
}

fn fail_mode_label(fail_open: bool) -> &'static str {
    if fail_open {
        "open"
    } else {
        "closed"
    }
}

fn debug_headers_enabled() -> bool {
    env::var("SHUMA_DEBUG_HEADERS")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn response_with_optional_debug_headers(
    status: u16,
    body: &str,
    kv_status: &str,
    fail_mode: &str,
) -> Response {
    let mut response_builder = Response::builder();
    let builder = response_builder.status(status);
    if debug_headers_enabled() {
        builder
            .header("X-KV-Status", kv_status)
            .header("X-Shuma-Fail-Mode", fail_mode)
            .body(body)
            .build()
    } else {
        builder.body(body).build()
    }
}

fn config_error_response(err: config::ConfigLoadError, path: &str) -> Response {
    log_line(&format!(
        "[CONFIG ERROR] path={} error={}",
        path,
        err.user_message()
    ));
    Response::new(500, "Configuration unavailable")
}

fn load_runtime_config(
    store: &Store,
    site_id: &str,
    path: &str,
) -> Result<config::Config, Response> {
    config::load_runtime_cached(store, site_id).map_err(|err| config_error_response(err, path))
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

#[allow(dead_code)]
pub(crate) fn compute_risk_score(
    js_needed: bool,
    geo_risk: bool,
    rate_count: u32,
    rate_limit: u32,
) -> u8 {
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

pub type BotnessContribution = crate::signals::botness::BotSignal;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct BotnessAssessment {
    pub score: u8,
    pub contributions: Vec<BotnessContribution>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeoAssessment {
    pub country: Option<String>,
    pub headers_trusted: bool,
    pub route: geo::GeoPolicyRoute,
    pub scored_risk: bool,
}

pub(crate) fn assess_geo_request(req: &Request, cfg: &config::Config) -> GeoAssessment {
    let headers_trusted = forwarded_ip_trusted(req);
    let country = geo::extract_geo_country(req, headers_trusted);
    let route = geo::evaluate_geo_policy(country.as_deref(), cfg);
    let scored_risk = if route == geo::GeoPolicyRoute::Allow {
        false
    } else {
        country
            .as_deref()
            .map(|value| geo::country_in_list(value, &cfg.geo_risk))
            .unwrap_or(false)
    };
    GeoAssessment {
        country,
        headers_trusted,
        route,
        scored_risk,
    }
}

pub(crate) fn compute_botness_assessment(
    js_enforced: bool,
    js_needed: bool,
    geo_signal_available: bool,
    geo_risk: bool,
    rate_count: u32,
    rate_limit: u32,
    cfg: &config::Config,
) -> BotnessAssessment {
    let mut accumulator = crate::signals::botness::SignalAccumulator::with_capacity(4);
    accumulator.push(js::bot_signal(
        js_enforced,
        js_needed,
        cfg.botness_weights.js_required,
    ));
    accumulator.push(geo::bot_signal(
        geo_signal_available,
        geo_risk,
        cfg.botness_weights.geo_risk,
    ));

    for rate_signal in crate::enforcement::rate::bot_signals(
        rate_count,
        rate_limit,
        cfg.botness_weights.rate_medium,
        cfg.botness_weights.rate_high,
    ) {
        accumulator.push(rate_signal);
    }

    let (score, contributions) = accumulator.finish();
    BotnessAssessment {
        score,
        contributions,
    }
}

pub(crate) fn botness_signals_summary(assessment: &BotnessAssessment) -> String {
    let active = assessment
        .contributions
        .iter()
        .filter(|c| c.active)
        .map(|c| format!("{}:{}", c.key, c.contribution))
        .collect::<Vec<_>>();
    if active.is_empty() {
        "none".to_string()
    } else {
        active.join(",")
    }
}

pub(crate) fn write_log_line(out: &mut impl Write, msg: &str) {
    let _ = writeln!(out, "{}", msg);
}

fn log_line(msg: &str) {
    let mut out = std::io::stdout();
    write_log_line(&mut out, msg);
}

pub(crate) fn serve_maze_with_tracking(
    store: &Store,
    cfg: &config::Config,
    ip: &str,
    path: &str,
    event_reason: &str,
    event_outcome: &str,
) -> Response {
    metrics::increment(store, metrics::MetricName::MazeHits, None);

    crate::admin::log_event(
        store,
        &crate::admin::EventLogEntry {
            ts: crate::admin::now_ts(),
            event: crate::admin::EventType::Challenge,
            ip: Some(ip.to_string()),
            reason: Some(event_reason.to_string()),
            outcome: Some(event_outcome.to_string()),
            admin: None,
        },
    );

    // Bucket the IP to reduce KV cardinality and avoid per-IP explosion.
    let maze_bucket = crate::signals::ip::bucket_ip(ip);
    let maze_key = format!("maze_hits:{}", maze_bucket);
    let hits: u32 = store
        .get(&maze_key)
        .ok()
        .flatten()
        .and_then(|v| String::from_utf8(v).ok())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    if let Err(e) = store.set(&maze_key, (hits + 1).to_string().as_bytes()) {
        log_line(&format!(
            "[maze] failed to persist hit counter {}: {:?}",
            maze_key, e
        ));
    }

    if hits >= cfg.maze_auto_ban_threshold && cfg.maze_auto_ban {
        ban::ban_ip_with_fingerprint(
            store,
            "default",
            ip,
            "maze_crawler",
            cfg.get_ban_duration("honeypot"),
            Some(crate::enforcement::ban::BanFingerprint {
                score: None,
                signals: vec!["maze_crawler_threshold".to_string()],
                summary: Some(format!(
                    "maze_hits={} threshold={}",
                    hits + 1,
                    cfg.maze_auto_ban_threshold
                )),
            }),
        );
        metrics::increment(store, metrics::MetricName::BansTotal, Some("maze_crawler"));
        crate::admin::log_event(
            store,
            &crate::admin::EventLogEntry {
                ts: crate::admin::now_ts(),
                event: crate::admin::EventType::Ban,
                ip: Some(ip.to_string()),
                reason: Some("maze_crawler".to_string()),
                outcome: Some(format!(
                    "banned_after_{}_maze_pages",
                    cfg.maze_auto_ban_threshold
                )),
                admin: None,
            },
        );
    }

    boundaries::handle_maze_request(path)
}

/// Main handler logic, testable as a plain Rust function.
pub fn handle_bot_defence_impl(req: &Request) -> Response {
    if let Err(err) = config::validate_env_only_once() {
        log_line(&format!("[ENV ERROR] {}", err));
        return Response::new(500, "Server configuration error");
    }
    let path = req.path();

    if crate::config::https_enforced() && !request_is_https(req) {
        return Response::new(403, "HTTPS required");
    }

    if let Some(response) = runtime::request_router::maybe_handle_early_route(req, path) {
        return response;
    }

    // CDP Report endpoint - receives automation detection reports from client-side JS
    if path == "/cdp-report" && *req.method() == spin_sdk::http::Method::Post {
        if let Ok(store) = Store::open_default() {
            return cdp::handle_cdp_report(&store, req);
        }
        return Response::new(500, "Key-value store error");
    }

    let site_id = "default";
    let ip = extract_client_ip(req);
    let ua = req
        .header("user-agent")
        .map(|v| v.as_str().unwrap_or(""))
        .unwrap_or("");

    let store = match runtime::kv_gate::open_store_or_fail_mode_response() {
        Ok(store) => store,
        Err(response) => return response,
    };
    let store = &store;

    let cfg = match load_runtime_config(store, site_id, path) {
        Ok(cfg) => cfg,
        Err(resp) => return resp,
    };
    let geo_assessment = assess_geo_request(req, &cfg);

    // Maze - trap crawlers in infinite loops (only if enabled)
    if boundaries::is_maze_path(path) {
        if !cfg.maze_enabled {
            return Response::new(404, "Not Found");
        }
        return serve_maze_with_tracking(store, &cfg, &ip, path, "maze_trap", "maze_page_served");
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
    if let Some(response) = test_mode::maybe_handle_test_mode(
        store,
        &cfg,
        site_id,
        &ip,
        ua,
        path,
        geo_assessment.route,
        || js::needs_js_verification(req, store, site_id, &ip),
        || metrics::increment(store, metrics::MetricName::TestModeActions, None),
    ) {
        return response;
    }
    if let Some(response) =
        runtime::policy_pipeline::maybe_handle_honeypot(store, &cfg, site_id, &ip, path)
    {
        return response;
    }
    if let Some(response) = runtime::policy_pipeline::maybe_handle_rate_limit(store, &cfg, site_id, &ip) {
        return response;
    }
    if let Some(response) = runtime::policy_pipeline::maybe_handle_existing_ban(store, site_id, &ip) {
        return response;
    }
    // PoW endpoints (public, before JS verification)
    if path == "/pow" {
        if *req.method() != spin_sdk::http::Method::Get {
            return Response::new(405, "Method Not Allowed");
        }
        return pow::handle_pow_challenge(
            &ip,
            cfg.pow_enabled,
            cfg.pow_difficulty,
            cfg.pow_ttl_seconds,
        );
    }
    if path == "/pow/verify" {
        return pow::handle_pow_verify(req, &ip, cfg.pow_enabled);
    }
    // Outdated browser
    if browser::is_outdated_browser(ua, &cfg.browser_block) {
        ban::ban_ip_with_fingerprint(
            store,
            site_id,
            &ip,
            "browser",
            cfg.get_ban_duration("browser"),
            Some(crate::enforcement::ban::BanFingerprint {
                score: None,
                signals: vec!["outdated_browser".to_string()],
                summary: Some(format!("ua={}", ua)),
            }),
        );
        metrics::increment(store, metrics::MetricName::BansTotal, Some("browser"));
        metrics::increment(store, metrics::MetricName::BlocksTotal, None);
        // Log ban event
        crate::admin::log_event(
            store,
            &crate::admin::EventLogEntry {
                ts: crate::admin::now_ts(),
                event: crate::admin::EventType::Ban,
                ip: Some(ip.clone()),
                reason: Some("browser".to_string()),
                outcome: Some("banned".to_string()),
                admin: None,
            },
        );
        return Response::new(
            403,
            block_page::render_block_page(block_page::BlockReason::OutdatedBrowser),
        );
    }
    if let Some(response) =
        runtime::policy_pipeline::maybe_handle_geo_policy(req, store, &cfg, &ip, &geo_assessment)
    {
        return response;
    }

    let needs_js =
        runtime::policy_pipeline::compute_needs_js(req, store, &cfg, site_id, path, &ip);

    if let Some(response) = runtime::policy_pipeline::maybe_handle_botness(
        req,
        store,
        &cfg,
        site_id,
        &ip,
        needs_js,
        &geo_assessment,
    ) {
        return response;
    }

    if let Some(response) = runtime::policy_pipeline::maybe_handle_js(store, &cfg, &ip, needs_js) {
        return response;
    }

    Response::new(200, "OK (passed bot defence)")
}

#[http_component]
pub fn spin_entrypoint(req: Request) -> Response {
    handle_bot_defence_impl(&req)
}
