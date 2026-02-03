use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};
/// Event types for activity logging
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EventType {
    Ban,
    Unban,
    Challenge,
    Block,
    AdminAction,
}

/// Event log entry
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EventLogEntry {
    pub ts: u64, // unix timestamp
    pub event: EventType,
    pub ip: Option<String>,
    pub reason: Option<String>,
    pub outcome: Option<String>,
    pub admin: Option<String>,
}

/// Append an event to the event log (simple append-only, time-bucketed by hour)
/// 
/// TODO: Implement data retention policy
/// - Add configurable retention period (e.g., 90 days)
/// - Create background cleanup job to periodically remove old event buckets
/// - Consider adding admin endpoint to manually trigger cleanup
/// - Example: Delete keys matching "eventlog:*" where hour < (now - retention_period)
pub fn log_event(store: &Store, entry: &EventLogEntry) {
    let hour = entry.ts / 3600;
    let key = format!("eventlog:{}", hour);
    let mut log: Vec<EventLogEntry> = store.get(&key)
        .ok()
        .flatten()
        .and_then(|v| serde_json::from_slice(&v).ok())
        .unwrap_or_else(Vec::new);
    log.push(entry.clone());
    let _ = store.set(&key, serde_json::to_vec(&log).unwrap().as_slice());
}

/// Utility to get current unix timestamp
pub fn now_ts() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}
// src/admin.rs
// Admin API endpoints for WASM Bot Trap
// Provides HTTP endpoints for ban management and analytics, protected by API key auth.

use spin_sdk::http::{Request, Response};
use spin_sdk::key_value::Store;
use serde_json::json;

/// Returns true if the path is a valid admin endpoint (prevents path traversal/abuse).
fn sanitize_path(path: &str) -> bool {
    matches!(path, "/admin" | "/admin/ban" | "/admin/unban" | "/admin/analytics" | "/admin/events" | "/admin/config" | "/admin/maze" | "/admin/robots" | "/admin/cdp")
}

/// Handles all /admin API endpoints. Requires valid API key in Authorization header.
/// Supports:
///   - GET /admin/ban: List all bans for the site
///   - POST /admin/ban: Manually ban an IP (expects JSON body: {"ip": "1.2.3.4", "reason": "...", "duration": 3600})
///   - POST /admin/unban?ip=...: Remove a ban for an IP
///   - GET /admin/analytics: Return ban count and test_mode status
///   - GET /admin/events: Query event log
///   - GET /admin/config: Get current config including test_mode status
///   - POST /admin/config: Update config (e.g., toggle test_mode)
///   - GET /admin: API help
pub fn handle_admin(req: &Request) -> Response {
    // Require valid API key
    if !crate::auth::is_authorized(req) {
        return Response::new(401, "Unauthorized: Invalid or missing API key");
    }
    let path = req.path();
    if !sanitize_path(path) {
        return Response::new(400, "Bad Request: Invalid admin endpoint");
    }
    let store = Store::open_default().expect("open default store");
    let site_id = "default";

    match path {
                "/admin/events" => {
                    // Query event log for recent events, top IPs, and event statistics
                    // Query params: ?hours=N (default 24)
                    let hours: u64 = req.query().strip_prefix("hours=").and_then(|v| v.parse().ok()).unwrap_or(24);
                    let now = now_ts();
                    let mut events: Vec<EventLogEntry> = Vec::new();
                    let mut ip_counts = std::collections::HashMap::new();
                    let mut event_counts = std::collections::HashMap::new();
                    let store = &store;
                    for h in 0..hours {
                        let hour = (now / 3600).saturating_sub(h);
                        let key = format!("eventlog:{}", hour);
                        if let Ok(Some(val)) = store.get(&key) {
                            if let Ok(log) = serde_json::from_slice::<Vec<EventLogEntry>>(&val) {
                                for e in &log {
                                    // Only include events within the time window
                                    if e.ts >= now - hours * 3600 {
                                        if let Some(ip) = &e.ip {
                                            *ip_counts.entry(ip.clone()).or_insert(0u32) += 1;
                                        }
                                        *event_counts.entry(format!("{:?}", e.event)).or_insert(0u32) += 1;
                                        events.push(e.clone());
                                    }
                                }
                            }
                        }
                    }
                    // Sort events by timestamp descending
                    events.sort_by(|a, b| b.ts.cmp(&a.ts));
                    // Top 10 IPs
                    let mut top_ips: Vec<_> = ip_counts.into_iter().collect();
                    top_ips.sort_by(|a, b| b.1.cmp(&a.1));
                    let top_ips: Vec<_> = top_ips.into_iter().take(10).collect();
                    let body = serde_json::to_string(&json!({
                        "recent_events": events.iter().take(100).collect::<Vec<_>>(),
                        "event_counts": event_counts,
                        "top_ips": top_ips,
                    })).unwrap();
                    // Log admin analytics view
                    log_event(store, &EventLogEntry {
                        ts: now_ts(),
                        event: EventType::AdminAction,
                        ip: None,
                        reason: Some("events_view".to_string()),
                        outcome: Some(format!("{} events", events.len())),
                        admin: Some(crate::auth::get_admin_id(req)),
                    });
                    Response::new(200, body)
                }
        "/admin/ban" => {
            // POST: Manually ban an IP
            if *req.method() == spin_sdk::http::Method::Post {
                let body = String::from_utf8_lossy(req.body());
                let parsed: Result<serde_json::Value, _> = serde_json::from_str(&body);
                if let Ok(json) = parsed {
                    if let (Some(ip), reason, duration) = (
                        json.get("ip").and_then(|v| v.as_str()),
                        json.get("reason").and_then(|v| v.as_str()).unwrap_or("admin_ban"),
                        json.get("duration").and_then(|v| v.as_u64()).unwrap_or(21600),
                    ) {
                        crate::ban::ban_ip(&store, site_id, ip, reason, duration);
                        // Log ban event
                        log_event(&store, &EventLogEntry {
                            ts: now_ts(),
                            event: EventType::Ban,
                            ip: Some(ip.to_string()),
                            reason: Some(reason.to_string()),
                            outcome: Some("banned".to_string()),
                            admin: Some(crate::auth::get_admin_id(req)),
                        });
                        return Response::new(200, json!({"status": "banned", "ip": ip}).to_string());
                    } else {
                        return Response::new(400, "Missing 'ip' field in request body");
                    }
                } else {
                    return Response::new(400, "Invalid JSON in request body");
                }
            }
            // GET: List all bans for this site (keys starting with ban:site_id:)
            let mut bans = vec![];
            if let Ok(keys) = store.get_keys() {
                for k in keys {
                    if k.starts_with(&format!("ban:{}:", site_id)) {
                        if let Ok(Some(val)) = store.get(&k) {
                            if let Ok(ban) = serde_json::from_slice::<crate::ban::BanEntry>(&val) {
                                bans.push(json!({"ip": k.split(':').last().unwrap_or("?"), "reason": ban.reason, "expires": ban.expires}));
                            }
                        }
                    }
                }
            }
            // Log admin action
            log_event(&store, &EventLogEntry {
                ts: now_ts(),
                event: EventType::AdminAction,
                ip: None,
                reason: Some("ban_list".to_string()),
                outcome: Some(format!("{} bans listed", bans.len())),
                admin: Some(crate::auth::get_admin_id(req)),
            });
            let body = serde_json::to_string(&json!({"bans": bans})).unwrap();
            Response::new(200, body)
        }
        "/admin/unban" => {
            // Unban IP (expects ?ip=...)
            let ip = req.query().strip_prefix("ip=").unwrap_or("");
            if ip.is_empty() {
                return Response::new(400, "Missing ip param");
            }
            let key = format!("ban:{}:{}", site_id, ip);
            let _ = store.delete(&key);
            // Log unban event
            log_event(&store, &EventLogEntry {
                ts: now_ts(),
                event: EventType::Unban,
                ip: Some(ip.to_string()),
                reason: Some("admin_unban".to_string()),
                outcome: Some("unbanned".to_string()),
                admin: Some(crate::auth::get_admin_id(req)),
            });
            Response::new(200, "Unbanned")
        }
        "/admin/analytics" => {
            // Return analytics: ban count and test_mode status
            let cfg = crate::config::Config::load(&store, site_id);
            let mut ban_count = 0;
            if let Ok(keys) = store.get_keys() {
                for k in keys {
                    if k.starts_with(&format!("ban:{}:", site_id)) {
                        ban_count += 1;
                    }
                }
            }
            // Log admin analytics view
            log_event(&store, &EventLogEntry {
                ts: now_ts(),
                event: EventType::AdminAction,
                ip: None,
                reason: Some("analytics_view".to_string()),
                outcome: Some(format!("ban_count={}", ban_count)),
                admin: Some(crate::auth::get_admin_id(req)),
            });
            let body = serde_json::to_string(&json!({
                "ban_count": ban_count,
                "test_mode": cfg.test_mode
            })).unwrap();
            Response::new(200, body)
        }
        "/admin/config" => {
            // GET: Return current config
            // POST: Update config (supports {"test_mode": true/false})
            if *req.method() == spin_sdk::http::Method::Post {
                let body_str = String::from_utf8_lossy(req.body());
                let parsed: Result<serde_json::Value, _> = serde_json::from_str(&body_str);
                if let Ok(json) = parsed {
                    // Load current config
                    let mut cfg = crate::config::Config::load(&store, site_id);
                    let mut changed = false;
                    
                    // Update test_mode if provided
                    if let Some(test_mode) = json.get("test_mode").and_then(|v| v.as_bool()) {
                        let old_value = cfg.test_mode;
                        cfg.test_mode = test_mode;
                        if old_value != test_mode {
                            changed = true;
                            // Log test_mode toggle event
                            log_event(&store, &EventLogEntry {
                                ts: now_ts(),
                                event: EventType::AdminAction,
                                ip: None,
                                reason: Some("test_mode_toggle".to_string()),
                                outcome: Some(format!("{} -> {}", old_value, test_mode)),
                                admin: Some(crate::auth::get_admin_id(req)),
                            });
                        }
                    }
                    
                    // Update other config fields if provided
                    if let Some(ban_duration) = json.get("ban_duration").and_then(|v| v.as_u64()) {
                        cfg.ban_duration = ban_duration;
                        changed = true;
                    }
                    if let Some(rate_limit) = json.get("rate_limit").and_then(|v| v.as_u64()) {
                        cfg.rate_limit = rate_limit as u32;
                        changed = true;
                    }
                    
                    // Update per-type ban durations if provided
                    if let Some(ban_durations) = json.get("ban_durations") {
                        if let Some(honeypot) = ban_durations.get("honeypot").and_then(|v| v.as_u64()) {
                            cfg.ban_durations.honeypot = honeypot;
                            changed = true;
                        }
                        if let Some(rate_limit) = ban_durations.get("rate_limit").and_then(|v| v.as_u64()) {
                            cfg.ban_durations.rate_limit = rate_limit;
                            changed = true;
                        }
                        if let Some(browser) = ban_durations.get("browser").and_then(|v| v.as_u64()) {
                            cfg.ban_durations.browser = browser;
                            changed = true;
                        }
                        if let Some(admin) = ban_durations.get("admin").and_then(|v| v.as_u64()) {
                            cfg.ban_durations.admin = admin;
                            changed = true;
                        }
                    }
                    
                    // Update maze settings if provided
                    if let Some(maze_enabled) = json.get("maze_enabled").and_then(|v| v.as_bool()) {
                        cfg.maze_enabled = maze_enabled;
                        changed = true;
                    }
                    if let Some(maze_auto_ban) = json.get("maze_auto_ban").and_then(|v| v.as_bool()) {
                        cfg.maze_auto_ban = maze_auto_ban;
                        changed = true;
                    }
                    if let Some(maze_auto_ban_threshold) = json.get("maze_auto_ban_threshold").and_then(|v| v.as_u64()) {
                        cfg.maze_auto_ban_threshold = maze_auto_ban_threshold as u32;
                        changed = true;
                    }
                    
                    // Update robots.txt settings if provided
                    if let Some(robots_enabled) = json.get("robots_enabled").and_then(|v| v.as_bool()) {
                        cfg.robots_enabled = robots_enabled;
                        changed = true;
                    }
                    if let Some(robots_block_ai_training) = json.get("robots_block_ai_training").and_then(|v| v.as_bool()) {
                        cfg.robots_block_ai_training = robots_block_ai_training;
                        changed = true;
                    }
                    if let Some(robots_block_ai_search) = json.get("robots_block_ai_search").and_then(|v| v.as_bool()) {
                        cfg.robots_block_ai_search = robots_block_ai_search;
                        changed = true;
                    }
                    if let Some(robots_allow_search_engines) = json.get("robots_allow_search_engines").and_then(|v| v.as_bool()) {
                        cfg.robots_allow_search_engines = robots_allow_search_engines;
                        changed = true;
                    }
                    if let Some(robots_crawl_delay) = json.get("robots_crawl_delay").and_then(|v| v.as_u64()) {
                        cfg.robots_crawl_delay = robots_crawl_delay as u32;
                        changed = true;
                    }
                    
                    // Update CDP detection settings if provided
                    if let Some(cdp_detection_enabled) = json.get("cdp_detection_enabled").and_then(|v| v.as_bool()) {
                        cfg.cdp_detection_enabled = cdp_detection_enabled;
                        changed = true;
                    }
                    if let Some(cdp_auto_ban) = json.get("cdp_auto_ban").and_then(|v| v.as_bool()) {
                        cfg.cdp_auto_ban = cdp_auto_ban;
                        changed = true;
                    }
                    if let Some(cdp_detection_threshold) = json.get("cdp_detection_threshold").and_then(|v| v.as_f64()) {
                        cfg.cdp_detection_threshold = cdp_detection_threshold as f32;
                        changed = true;
                    }
                    
                    // Save config to KV store
                    if changed {
                        let key = format!("config:{}", site_id);
                        if let Ok(val) = serde_json::to_vec(&cfg) {
                            let _ = store.set(&key, &val);
                        }
                    }
                    
                    let body = serde_json::to_string(&json!({
                        "status": "updated",
                        "config": {
                            "test_mode": cfg.test_mode,
                            "ban_duration": cfg.ban_duration,
                            "ban_durations": {
                                "honeypot": cfg.ban_durations.honeypot,
                                "rate_limit": cfg.ban_durations.rate_limit,
                                "browser": cfg.ban_durations.browser,
                                "admin": cfg.ban_durations.admin
                            },
                            "rate_limit": cfg.rate_limit,
                            "honeypots": cfg.honeypots,
                            "geo_risk": cfg.geo_risk,
                            "maze_enabled": cfg.maze_enabled,
                            "maze_auto_ban": cfg.maze_auto_ban,
                            "maze_auto_ban_threshold": cfg.maze_auto_ban_threshold,
                            "robots_enabled": cfg.robots_enabled,
                            "robots_block_ai_training": cfg.robots_block_ai_training,
                            "robots_block_ai_search": cfg.robots_block_ai_search,
                            "robots_allow_search_engines": cfg.robots_allow_search_engines,
                            "robots_crawl_delay": cfg.robots_crawl_delay,
                            "cdp_detection_enabled": cfg.cdp_detection_enabled,
                            "cdp_auto_ban": cfg.cdp_auto_ban,
                            "cdp_detection_threshold": cfg.cdp_detection_threshold
                        }
                    })).unwrap();
                    return Response::new(200, body);
                } else {
                    return Response::new(400, "Invalid JSON in request body");
                }
            }
            // GET: Return current config
            let cfg = crate::config::Config::load(&store, site_id);
            log_event(&store, &EventLogEntry {
                ts: now_ts(),
                event: EventType::AdminAction,
                ip: None,
                reason: Some("config_view".to_string()),
                outcome: Some(format!("test_mode={}", cfg.test_mode)),
                admin: Some(crate::auth::get_admin_id(req)),
            });
            let body = serde_json::to_string(&json!({
                "test_mode": cfg.test_mode,
                "ban_duration": cfg.ban_duration,
                "ban_durations": {
                    "honeypot": cfg.ban_durations.honeypot,
                    "rate_limit": cfg.ban_durations.rate_limit,
                    "browser": cfg.ban_durations.browser,
                    "admin": cfg.ban_durations.admin
                },
                "rate_limit": cfg.rate_limit,
                "honeypots": cfg.honeypots,
                "browser_block": cfg.browser_block,
                "browser_whitelist": cfg.browser_whitelist,
                "geo_risk": cfg.geo_risk,
                "whitelist": cfg.whitelist,
                "path_whitelist": cfg.path_whitelist,
                "maze_enabled": cfg.maze_enabled,
                "maze_auto_ban": cfg.maze_auto_ban,
                "maze_auto_ban_threshold": cfg.maze_auto_ban_threshold,
                "robots_enabled": cfg.robots_enabled,
                "robots_block_ai_training": cfg.robots_block_ai_training,
                "robots_block_ai_search": cfg.robots_block_ai_search,
                "robots_allow_search_engines": cfg.robots_allow_search_engines,
                "robots_crawl_delay": cfg.robots_crawl_delay,
                "cdp_detection_enabled": cfg.cdp_detection_enabled,
                "cdp_auto_ban": cfg.cdp_auto_ban,
                "cdp_detection_threshold": cfg.cdp_detection_threshold
            })).unwrap();
            Response::new(200, body)
        }
        "/admin" => {
            // API help endpoint
            log_event(&store, &EventLogEntry {
                ts: now_ts(),
                event: EventType::AdminAction,
                ip: None,
                reason: Some("help".to_string()),
                outcome: None,
                admin: Some(crate::auth::get_admin_id(req)),
            });
            Response::new(200, "WASM Bot Trap Admin API. Endpoints: /admin/ban, /admin/unban?ip=IP, /admin/analytics, /admin/events, /admin/config, /admin/maze (GET for maze stats), /admin/robots (GET for robots.txt config & preview), /admin/cdp (GET for CDP detection config & stats).")
        }
        "/admin/maze" => {
            // Return maze honeypot statistics
            // - Total unique IPs that have visited maze pages
            // - Per-IP hit counts (top crawlers)
            // - Total maze hits
            let mut maze_ips: Vec<(String, u32)> = Vec::new();
            let mut total_hits: u32 = 0;
            
            if let Ok(keys) = store.get_keys() {
                for k in keys {
                    if k.starts_with("maze_hits:") {
                        let ip = k.strip_prefix("maze_hits:").unwrap_or("unknown").to_string();
                        if let Ok(Some(val)) = store.get(&k) {
                            if let Ok(hits) = String::from_utf8_lossy(&val).parse::<u32>() {
                                total_hits += hits;
                                maze_ips.push((ip, hits));
                            }
                        }
                    }
                }
            }
            
            // Sort by hits descending
            maze_ips.sort_by(|a, b| b.1.cmp(&a.1));
            
            // Get the deepest crawler (most maze page visits)
            let deepest = maze_ips.first().map(|(ip, hits)| json!({"ip": ip, "hits": hits}));
            
            // Top 10 crawlers
            let top_crawlers: Vec<_> = maze_ips.iter()
                .take(10)
                .map(|(ip, hits)| json!({"ip": ip, "hits": hits}))
                .collect();
            
            // Count auto-bans from maze (check bans with reason "maze_crawler")
            let mut maze_bans = 0;
            if let Ok(keys) = store.get_keys() {
                for k in keys {
                    if k.starts_with(&format!("ban:{}:", site_id)) {
                        if let Ok(Some(val)) = store.get(&k) {
                            if let Ok(ban) = serde_json::from_slice::<crate::ban::BanEntry>(&val) {
                                if ban.reason == "maze_crawler" {
                                    maze_bans += 1;
                                }
                            }
                        }
                    }
                }
            }
            
            // Log admin maze view
            log_event(&store, &EventLogEntry {
                ts: now_ts(),
                event: EventType::AdminAction,
                ip: None,
                reason: Some("maze_stats_view".to_string()),
                outcome: Some(format!("{} crawlers, {} hits", maze_ips.len(), total_hits)),
                admin: Some(crate::auth::get_admin_id(req)),
            });
            
            let body = serde_json::to_string(&json!({
                "total_hits": total_hits,
                "unique_crawlers": maze_ips.len(),
                "maze_auto_bans": maze_bans,
                "deepest_crawler": deepest,
                "top_crawlers": top_crawlers
            })).unwrap();
            Response::new(200, body)
        }
        "/admin/robots" => {
            // Return robots.txt configuration and preview
            let cfg = crate::config::Config::load(&store, site_id);
            
            // Generate preview of robots.txt content
            let preview = crate::robots::generate_robots_txt(&cfg);
            let content_signal = crate::robots::get_content_signal_header(&cfg);
            
            // Log admin action
            log_event(&store, &EventLogEntry {
                ts: now_ts(),
                event: EventType::AdminAction,
                ip: None,
                reason: Some("robots_config_view".to_string()),
                outcome: None,
                admin: Some(crate::auth::get_admin_id(req)),
            });
            
            let body = serde_json::to_string(&json!({
                "config": {
                    "enabled": cfg.robots_enabled,
                    "block_ai_training": cfg.robots_block_ai_training,
                    "block_ai_search": cfg.robots_block_ai_search,
                    "allow_search_engines": cfg.robots_allow_search_engines,
                    "crawl_delay": cfg.robots_crawl_delay
                },
                "content_signal_header": content_signal,
                "ai_training_bots": crate::robots::AI_TRAINING_BOTS,
                "ai_search_bots": crate::robots::AI_SEARCH_BOTS,
                "search_engine_bots": crate::robots::SEARCH_ENGINE_BOTS,
                "preview": preview
            })).unwrap();
            Response::new(200, body)
        }
        "/admin/cdp" => {
            // Return CDP detection configuration and stats
            let cfg = crate::config::Config::load(&store, site_id);
            
            // Get CDP detection stats from KV store
            let cdp_detections: u64 = store.get("cdp:detections")
                .ok()
                .flatten()
                .and_then(|v| String::from_utf8(v).ok())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            
            let cdp_auto_bans: u64 = store.get("cdp:auto_bans")
                .ok()
                .flatten()
                .and_then(|v| String::from_utf8(v).ok())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            
            // Log admin action
            log_event(&store, &EventLogEntry {
                ts: now_ts(),
                event: EventType::AdminAction,
                ip: None,
                reason: Some("cdp_config_view".to_string()),
                outcome: None,
                admin: Some(crate::auth::get_admin_id(req)),
            });
            
            let body = serde_json::to_string(&json!({
                "config": {
                    "enabled": cfg.cdp_detection_enabled,
                    "auto_ban": cfg.cdp_auto_ban,
                    "detection_threshold": cfg.cdp_detection_threshold
                },
                "stats": {
                    "total_detections": cdp_detections,
                    "auto_bans": cdp_auto_bans
                },
                "detection_methods": [
                    "Error stack timing analysis (Runtime.Enable leak)",
                    "navigator.webdriver property check",
                    "Automation-specific window properties",
                    "Chrome object consistency verification",
                    "Plugin array anomaly detection"
                ]
            })).unwrap();
            Response::new(200, body)
        }
        _ => Response::new(404, "Not found"),
    }
}
