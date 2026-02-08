use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};
use std::env;
use once_cell::sync::Lazy;
use std::sync::Mutex;
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
const EVENT_PAGE_SIZE: usize = 500; // max entries per page
const EVENT_MAX_PAGES_PER_HOUR: usize = 256; // safety cap
const DEFAULT_EVENT_RETENTION_HOURS: u64 = 168; // 7 days
const POW_DIFFICULTY_MIN: u8 = crate::config::POW_DIFFICULTY_MIN;
const POW_DIFFICULTY_MAX: u8 = crate::config::POW_DIFFICULTY_MAX;
const POW_TTL_MIN: u64 = crate::config::POW_TTL_MIN;
const POW_TTL_MAX: u64 = crate::config::POW_TTL_MAX;

static LAST_EVENTLOG_CLEANUP_HOUR: Lazy<Mutex<u64>> = Lazy::new(|| Mutex::new(0));

fn event_log_retention_hours() -> u64 {
    env::var("EVENT_LOG_RETENTION_HOURS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(DEFAULT_EVENT_RETENTION_HOURS)
}

fn maybe_cleanup_event_logs<S: crate::challenge::KeyValueStore>(store: &S, current_hour: u64) {
    let retention = event_log_retention_hours();
    if retention == 0 {
        return;
    }
    let mut last = LAST_EVENTLOG_CLEANUP_HOUR.lock().unwrap();
    if *last == current_hour {
        return;
    }
    *last = current_hour;

    let cutoff_hour = current_hour.saturating_sub(retention);
    for page in 1..=EVENT_MAX_PAGES_PER_HOUR {
        let key = format!("eventlog:{}:{}", cutoff_hour, page);
        let _ = store.delete(&key);
    }
}

pub fn log_event<S: crate::challenge::KeyValueStore>(store: &S, entry: &EventLogEntry) {
    // Use paged hourly event logs to avoid unbounded vector growth and expensive
    // read-modify-write cycles. Each hour is split into pages of limited size.
    let hour = entry.ts / 3600;
    let prefix = format!("eventlog:{}", hour);
    maybe_cleanup_event_logs(store, hour);

    for page in 1..=EVENT_MAX_PAGES_PER_HOUR {
        let page_key = format!("{}:{}", prefix, page);
        match store.get(&page_key) {
            Ok(Some(val)) => {
                // Try to decode existing page
                match serde_json::from_slice::<Vec<EventLogEntry>>(&val) {
                    Ok(mut log) => {
                        if log.len() < EVENT_PAGE_SIZE {
                            log.push(entry.clone());
                            let _ = store.set(&page_key, serde_json::to_vec(&log).unwrap().as_slice());
                            return;
                        } else {
                            // page full, try next
                            continue;
                        }
                    }
                    Err(_) => {
                        // Corrupted page: overwrite with new page containing this entry
                        let new_page = vec![entry.clone()];
                        let _ = store.set(&page_key, serde_json::to_vec(&new_page).unwrap().as_slice());
                        return;
                    }
                }
            }
            Ok(None) => {
                // Create first page entry
                let new_page = vec![entry.clone()];
                let _ = store.set(&page_key, serde_json::to_vec(&new_page).unwrap().as_slice());
                return;
            }
            Err(_) => {
                // KV error; best-effort: log to stderr and return without blocking
                eprintln!("[log_event] KV error writing {}", page_key);
                return;
            }
        }
    }

    // If we reach here, we've exhausted EVENT_MAX_PAGES_PER_HOUR — drop the event and log
    eprintln!("[log_event] reached max pages for hour {}, dropping event", hour);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::challenge::KeyValueStore;
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct MockStore {
        map: Mutex<HashMap<String, Vec<u8>>>,
    }

    impl MockStore {
        fn new() -> Self {
            MockStore { map: Mutex::new(HashMap::new()) }
        }
    }

    impl crate::challenge::KeyValueStore for MockStore {
        fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ()> {
            let m = self.map.lock().unwrap();
            Ok(m.get(key).cloned())
        }
        fn set(&self, key: &str, value: &[u8]) -> Result<(), ()> {
            let mut m = self.map.lock().unwrap();
            m.insert(key.to_string(), value.to_vec());
            Ok(())
        }
        fn delete(&self, key: &str) -> Result<(), ()> {
            let mut m = self.map.lock().unwrap();
            m.remove(key);
            Ok(())
        }
    }

    #[test]
    fn paged_event_log_creates_pages_and_appends() {
        let store = MockStore::new();
        let now = now_ts();
        let entry = EventLogEntry { ts: now, event: EventType::AdminAction, ip: Some("1.2.3.4".to_string()), reason: Some("test".to_string()), outcome: Some("ok".to_string()), admin: Some("me".to_string()) };
        // Log a few events
        for _ in 0..5 {
            log_event(&store, &entry);
        }
        // Verify page 1 exists and has 5 entries
        let key = format!("eventlog:{}:1", now / 3600);
        let val = store.get(&key).unwrap().unwrap();
        let v: Vec<EventLogEntry> = serde_json::from_slice(&val).unwrap();
        assert_eq!(v.len(), 5);
    }

    #[test]
    fn corrupted_page_is_overwritten() {
        let store = MockStore::new();
        let hour = now_ts() / 3600;
        let page_key = format!("eventlog:{}:1", hour);
        // Insert corrupted data
        store.set(&page_key, b"not-json").unwrap();
        let entry = EventLogEntry { ts: now_ts(), event: EventType::AdminAction, ip: None, reason: None, outcome: None, admin: None };
        log_event(&store, &entry);
        let val = store.get(&page_key).unwrap().unwrap();
        let v: Vec<EventLogEntry> = serde_json::from_slice(&val).unwrap();
        assert_eq!(v.len(), 1);
    }
}

#[cfg(test)]
mod admin_config_tests {
    use super::*;
    use spin_sdk::http::{Method, Request};
    use once_cell::sync::Lazy;
    use std::sync::Mutex;
    use std::collections::HashMap;

    static ENV_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    fn make_request(method: Method, path: &str, body: Vec<u8>) -> Request {
        let mut builder = Request::builder();
        builder
            .method(method)
            .uri(path)
            .header("authorization", "Bearer changeme-dev-only-api-key")
            .body(body);
        builder.build()
    }

    #[derive(Default)]
    struct TestStore {
        map: Mutex<HashMap<String, Vec<u8>>>,
    }

    impl crate::challenge::KeyValueStore for TestStore {
        fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ()> {
            let m = self.map.lock().unwrap();
            Ok(m.get(key).cloned())
        }
        fn set(&self, key: &str, value: &[u8]) -> Result<(), ()> {
            let mut m = self.map.lock().unwrap();
            m.insert(key.to_string(), value.to_vec());
            Ok(())
        }
        fn delete(&self, key: &str) -> Result<(), ()> {
            let mut m = self.map.lock().unwrap();
            m.remove(key);
            Ok(())
        }
    }

    #[test]
    fn admin_config_includes_challenge_fields() {
        let _lock = ENV_MUTEX.lock().unwrap();
        std::env::remove_var("CHALLENGE_CONFIG_MUTABLE");
        std::env::remove_var("BOTNESS_CONFIG_MUTABLE");
        let req = make_request(Method::Get, "/admin/config", Vec::new());
        let store = TestStore::default();
        let resp = handle_admin_config(&req, &store, "default");
        assert_eq!(*resp.status(), 200u16);
        let body: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
        assert!(body.get("challenge_risk_threshold").is_some());
        assert!(body.get("challenge_config_mutable").is_some());
        assert!(body.get("challenge_risk_threshold_default").is_some());
        assert!(body.get("botness_maze_threshold").is_some());
        assert!(body.get("botness_weights").is_some());
        assert!(body.get("botness_config_mutable").is_some());
        assert!(body.get("botness_signal_definitions").is_some());
    }

    #[test]
    fn admin_config_rejects_challenge_update_when_immutable() {
        let _lock = ENV_MUTEX.lock().unwrap();
        std::env::set_var("CHALLENGE_CONFIG_MUTABLE", "0");
        let body = br#"{"challenge_risk_threshold":5}"#.to_vec();
        let req = make_request(Method::Post, "/admin/config", body);
        let store = TestStore::default();
        let resp = handle_admin_config(&req, &store, "default");
        assert_eq!(*resp.status(), 403u16);
        std::env::remove_var("CHALLENGE_CONFIG_MUTABLE");
    }

    #[test]
    fn admin_config_rejects_updates_in_env_only_mode() {
        let _lock = ENV_MUTEX.lock().unwrap();
        std::env::set_var("SHUMA_CONFIG_MODE", "env_only");
        let body = br#"{"test_mode":true}"#.to_vec();
        let req = make_request(Method::Post, "/admin/config", body);
        let store = TestStore::default();
        let resp = handle_admin_config(&req, &store, "default");
        assert_eq!(*resp.status(), 403u16);
        let msg = String::from_utf8_lossy(resp.body());
        assert!(msg.contains("SHUMA_CONFIG_MODE=env_only"));
        std::env::remove_var("SHUMA_CONFIG_MODE");
    }
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

fn challenge_threshold_default() -> u8 {
    crate::config::parse_challenge_threshold(env::var("CHALLENGE_RISK_THRESHOLD").ok().as_deref())
}

fn maze_threshold_default() -> u8 {
    crate::config::parse_maze_threshold(env::var("BOTNESS_MAZE_THRESHOLD").ok().as_deref())
}

fn botness_signal_definitions(cfg: &crate::config::Config) -> serde_json::Value {
    json!({
        "scored_signals": [
            {
                "key": "js_verification_required",
                "label": "JS verification required",
                "weight": cfg.botness_weights.js_required
            },
            {
                "key": "geo_risk",
                "label": "High-risk geography",
                "weight": cfg.botness_weights.geo_risk
            },
            {
                "key": "rate_pressure_medium",
                "label": "Rate pressure (>=50%)",
                "weight": cfg.botness_weights.rate_medium
            },
            {
                "key": "rate_pressure_high",
                "label": "Rate pressure (>=80%)",
                "weight": cfg.botness_weights.rate_high
            }
        ],
        "terminal_signals": [
            { "key": "honeypot", "label": "Honeypot hit", "action": "Immediate ban" },
            { "key": "rate_limit_exceeded", "label": "Rate limit exceeded", "action": "Immediate ban" },
            { "key": "outdated_browser", "label": "Outdated browser", "action": "Immediate ban" },
            { "key": "cdp_automation", "label": "CDP automation detected", "action": "Immediate ban (if enabled)" },
            { "key": "maze_crawler_threshold", "label": "Maze crawler threshold reached", "action": "Immediate ban (if enabled)" },
            { "key": "already_banned", "label": "Existing active ban", "action": "Block page" }
        ]
    })
}

fn handle_admin_config(req: &Request, store: &impl crate::challenge::KeyValueStore, site_id: &str) -> Response {
    // GET: Return current config
    // POST: Update config (supports {"test_mode": true/false})
    if *req.method() == spin_sdk::http::Method::Post {
        if matches!(crate::config::config_mode(), crate::config::ConfigMode::EnvOnly) {
            return Response::new(403, "Config updates are disabled when SHUMA_CONFIG_MODE=env_only");
        }
        let body_str = String::from_utf8_lossy(req.body());
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&body_str);
        if let Ok(json) = parsed {
            // Load current config
            let mut cfg = crate::config::Config::load(store, site_id);
            let mut changed = false;
            
            // Update test_mode if provided
            if let Some(test_mode) = json.get("test_mode").and_then(|v| v.as_bool()) {
                let old_value = cfg.test_mode;
                cfg.test_mode = test_mode;
                if old_value != test_mode {
                    changed = true;
                    // Log test_mode toggle event
                    log_event(store, &EventLogEntry {
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

            let old_pow_difficulty = cfg.pow_difficulty;
            let old_pow_ttl = cfg.pow_ttl_seconds;
            let mut pow_changed = false;

            // Update PoW settings if provided (guarded by env flag)
            if json.get("pow_difficulty").is_some() || json.get("pow_ttl_seconds").is_some() {
                if !crate::config::pow_config_mutable() {
                    return Response::new(403, "PoW config is immutable (set POW_CONFIG_MUTABLE=1 to allow changes)");
                }
            }
            if let Some(pow_difficulty) = json.get("pow_difficulty").and_then(|v| v.as_u64()) {
                if pow_difficulty < POW_DIFFICULTY_MIN as u64 || pow_difficulty > POW_DIFFICULTY_MAX as u64 {
                    return Response::new(400, "pow_difficulty out of range (12-20)");
                }
                cfg.pow_difficulty = pow_difficulty as u8;
                changed = true;
                pow_changed = true;
            }
            if let Some(pow_ttl_seconds) = json.get("pow_ttl_seconds").and_then(|v| v.as_u64()) {
                if pow_ttl_seconds < POW_TTL_MIN || pow_ttl_seconds > POW_TTL_MAX {
                    return Response::new(400, "pow_ttl_seconds out of range (30-300)");
                }
                cfg.pow_ttl_seconds = pow_ttl_seconds;
                changed = true;
                pow_changed = true;
            }

            if pow_changed {
                log_event(store, &EventLogEntry {
                    ts: now_ts(),
                    event: EventType::AdminAction,
                    ip: None,
                    reason: Some("pow_config_update".to_string()),
                    outcome: Some(format!(
                        "difficulty:{}->{} ttl:{}->{}",
                        old_pow_difficulty,
                        cfg.pow_difficulty,
                        old_pow_ttl,
                        cfg.pow_ttl_seconds
                    )),
                    admin: Some(crate::auth::get_admin_id(req)),
                });
            }

            let botness_mutable = crate::config::botness_config_mutable();
            let mut botness_changed = false;
            let old_challenge_threshold = cfg.challenge_risk_threshold;
            let old_maze_threshold = cfg.botness_maze_threshold;
            let old_weights = cfg.botness_weights.clone();
            let botness_update_requested =
                json.get("challenge_risk_threshold").is_some()
                || json.get("botness_maze_threshold").is_some()
                || json.get("botness_weights").is_some();
            if botness_update_requested && !botness_mutable {
                return Response::new(
                    403,
                    "Botness config is immutable (set BOTNESS_CONFIG_MUTABLE=true or CHALLENGE_CONFIG_MUTABLE=true to allow changes)"
                );
            }
            if let Some(challenge_threshold) = json.get("challenge_risk_threshold").and_then(|v| v.as_u64()) {
                if challenge_threshold < 1 || challenge_threshold > 10 {
                    return Response::new(400, "challenge_risk_threshold out of range (1-10)");
                }
                cfg.challenge_risk_threshold = challenge_threshold as u8;
                changed = true;
                botness_changed = true;
            }
            if let Some(maze_threshold) = json.get("botness_maze_threshold").and_then(|v| v.as_u64()) {
                if maze_threshold < 1 || maze_threshold > 10 {
                    return Response::new(400, "botness_maze_threshold out of range (1-10)");
                }
                cfg.botness_maze_threshold = maze_threshold as u8;
                changed = true;
                botness_changed = true;
            }
            if let Some(weights) = json.get("botness_weights") {
                if let Some(js_required) = weights.get("js_required").and_then(|v| v.as_u64()) {
                    if js_required > 10 {
                        return Response::new(400, "botness_weights.js_required out of range (0-10)");
                    }
                    cfg.botness_weights.js_required = js_required as u8;
                    changed = true;
                    botness_changed = true;
                }
                if let Some(geo_risk) = weights.get("geo_risk").and_then(|v| v.as_u64()) {
                    if geo_risk > 10 {
                        return Response::new(400, "botness_weights.geo_risk out of range (0-10)");
                    }
                    cfg.botness_weights.geo_risk = geo_risk as u8;
                    changed = true;
                    botness_changed = true;
                }
                if let Some(rate_medium) = weights.get("rate_medium").and_then(|v| v.as_u64()) {
                    if rate_medium > 10 {
                        return Response::new(400, "botness_weights.rate_medium out of range (0-10)");
                    }
                    cfg.botness_weights.rate_medium = rate_medium as u8;
                    changed = true;
                    botness_changed = true;
                }
                if let Some(rate_high) = weights.get("rate_high").and_then(|v| v.as_u64()) {
                    if rate_high > 10 {
                        return Response::new(400, "botness_weights.rate_high out of range (0-10)");
                    }
                    cfg.botness_weights.rate_high = rate_high as u8;
                    changed = true;
                    botness_changed = true;
                }
            }

            if botness_changed {
                log_event(store, &EventLogEntry {
                    ts: now_ts(),
                    event: EventType::AdminAction,
                    ip: None,
                    reason: Some("botness_config_update".to_string()),
                    outcome: Some(format!(
                        "challenge:{}->{} maze:{}->{} weights(js:{}->{} geo:{}->{} rate_med:{}->{} rate_high:{}->{})",
                        old_challenge_threshold,
                        cfg.challenge_risk_threshold,
                        old_maze_threshold,
                        cfg.botness_maze_threshold,
                        old_weights.js_required,
                        cfg.botness_weights.js_required,
                        old_weights.geo_risk,
                        cfg.botness_weights.geo_risk,
                        old_weights.rate_medium,
                        cfg.botness_weights.rate_medium,
                        old_weights.rate_high,
                        cfg.botness_weights.rate_high
                    )),
                    admin: Some(crate::auth::get_admin_id(req)),
                });
            }
            
            // Save config to KV store
            if changed {
                let key = format!("config:{}", site_id);
                if let Ok(val) = serde_json::to_vec(&cfg) {
                    let _ = store.set(&key, &val);
                }
            }

            let challenge_default = challenge_threshold_default();
            let maze_default = maze_threshold_default();
            
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
                    "cdp_detection_threshold": cfg.cdp_detection_threshold,
                    "pow_enabled": crate::pow::pow_enabled(),
                    "pow_config_mutable": crate::config::pow_config_mutable(),
                    "pow_difficulty": cfg.pow_difficulty,
                    "pow_ttl_seconds": cfg.pow_ttl_seconds,
                    "config_mode": crate::config::config_mode_label(),
                    "challenge_risk_threshold": cfg.challenge_risk_threshold,
                    "challenge_config_mutable": crate::config::challenge_config_mutable(),
                    "challenge_risk_threshold_default": challenge_default,
                    "botness_maze_threshold": cfg.botness_maze_threshold,
                    "botness_maze_threshold_default": maze_default,
                    "botness_weights": {
                        "js_required": cfg.botness_weights.js_required,
                        "geo_risk": cfg.botness_weights.geo_risk,
                        "rate_medium": cfg.botness_weights.rate_medium,
                        "rate_high": cfg.botness_weights.rate_high
                    },
                    "botness_config_mutable": botness_mutable,
                    "botness_signal_definitions": botness_signal_definitions(&cfg)
                }
            })).unwrap();
            return Response::new(200, body);
        } else {
            return Response::new(400, "Invalid JSON in request body");
        }
    }
    // GET: Return current config
    let cfg = crate::config::Config::load(store, site_id);
    log_event(store, &EventLogEntry {
        ts: now_ts(),
        event: EventType::AdminAction,
        ip: None,
        reason: Some("config_view".to_string()),
        outcome: Some(format!("test_mode={}", cfg.test_mode)),
        admin: Some(crate::auth::get_admin_id(req)),
    });
    let challenge_default = challenge_threshold_default();
    let maze_default = maze_threshold_default();
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
        "cdp_detection_threshold": cfg.cdp_detection_threshold,
        "pow_enabled": crate::pow::pow_enabled(),
        "pow_config_mutable": crate::config::pow_config_mutable(),
        "pow_difficulty": cfg.pow_difficulty,
        "pow_ttl_seconds": cfg.pow_ttl_seconds,
        "config_mode": crate::config::config_mode_label(),
        "challenge_risk_threshold": cfg.challenge_risk_threshold,
        "challenge_config_mutable": crate::config::challenge_config_mutable(),
        "challenge_risk_threshold_default": challenge_default,
        "botness_maze_threshold": cfg.botness_maze_threshold,
        "botness_maze_threshold_default": maze_default,
        "botness_weights": {
            "js_required": cfg.botness_weights.js_required,
            "geo_risk": cfg.botness_weights.geo_risk,
            "rate_medium": cfg.botness_weights.rate_medium,
            "rate_high": cfg.botness_weights.rate_high
        },
        "botness_config_mutable": crate::config::botness_config_mutable(),
        "botness_signal_definitions": botness_signal_definitions(&cfg)
    })).unwrap();
    Response::new(200, body)
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
    // Optional admin IP allowlist
    if !crate::auth::is_admin_ip_allowed(req) {
        return Response::new(403, "Forbidden");
    }
    if !crate::auth::is_admin_api_key_configured() {
        return Response::new(503, "Admin API disabled: API_KEY must be set to a non-default value");
    }
    // Require valid API key
    if !crate::auth::is_authorized(req) {
        return Response::new(401, "Unauthorized: Invalid or missing API key");
    }
    let path = req.path();
    if !sanitize_path(path) {
        return Response::new(400, "Bad Request: Invalid admin endpoint");
    }
    let store = match Store::open_default() {
        Ok(s) => s,
        Err(_) => return Response::new(500, "Key-value store error"),
    };
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
                        // Iterate eventlog pages for this hour
                        for page in 1..=EVENT_MAX_PAGES_PER_HOUR {
                            let key = format!("eventlog:{}:{}", hour, page);
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
                            } else {
                                // No page present -> no further pages for this hour
                                break;
                            }
                        }
                    }
                    // Sort events by timestamp descending
                    events.sort_by(|a, b| b.ts.cmp(&a.ts));
                    // Unique IP count before consuming the map
                    let unique_ips = ip_counts.len();
                    // Top 10 IPs
                    let mut top_ips: Vec<_> = ip_counts.into_iter().collect();
                    top_ips.sort_by(|a, b| b.1.cmp(&a.1));
                    let top_ips: Vec<_> = top_ips.into_iter().take(10).collect();
                    let body = serde_json::to_string(&json!({
                        "recent_events": events.iter().take(100).collect::<Vec<_>>(),
                        "event_counts": event_counts,
                        "top_ips": top_ips,
                        "unique_ips": unique_ips,
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
                        crate::ban::ban_ip_with_fingerprint(
                            &store,
                            site_id,
                            ip,
                            reason,
                            duration,
                            Some(crate::ban::BanFingerprint {
                                score: None,
                                signals: vec!["manual_admin".to_string()],
                                summary: Some("manual_admin_ban".to_string()),
                            }),
                        );
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
            for (ip, ban) in crate::ban::list_active_bans_with_scan(&store, site_id) {
                bans.push(json!({
                    "ip": ip,
                    "reason": ban.reason,
                    "expires": ban.expires,
                    "banned_at": ban.banned_at,
                    "fingerprint": ban.fingerprint
                }));
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
            // Use the ban module's unban_ip function for consistency
            crate::ban::unban_ip(&store, site_id, ip);
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
            let ban_count = crate::ban::list_active_bans_with_scan(&store, site_id).len();
            let fail_mode = env::var("SHUMA_KV_STORE_FAIL_MODE").unwrap_or_else(|_| "open".to_string()).to_lowercase();
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
                "test_mode": cfg.test_mode,
                "fail_mode": fail_mode
            })).unwrap();
            Response::new(200, body)
        }
        "/admin/config" => {
            return handle_admin_config(req, &store, site_id);
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
            let maze_bans = crate::ban::list_active_bans_with_scan(&store, site_id)
                .into_iter()
                .filter(|(_, ban)| ban.reason == "maze_crawler")
                .count();
            
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
