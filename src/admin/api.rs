use once_cell::sync::Lazy;
use rand::random;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Mutex;
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

/// Event log storage notes:
/// - v2 format stores immutable records per event: eventlog:v2:<hour>:<ts>-<nonce>
const EVENTLOG_V2_PREFIX: &str = "eventlog:v2";
const POW_DIFFICULTY_MIN: u8 = crate::config::POW_DIFFICULTY_MIN;
const POW_DIFFICULTY_MAX: u8 = crate::config::POW_DIFFICULTY_MAX;
const POW_TTL_MIN: u64 = crate::config::POW_TTL_MIN;
const POW_TTL_MAX: u64 = crate::config::POW_TTL_MAX;
const CONFIG_EXPORT_SECRET_KEYS: [&str; 7] = [
    "SHUMA_API_KEY",
    "SHUMA_ADMIN_READONLY_API_KEY",
    "SHUMA_JS_SECRET",
    "SHUMA_POW_SECRET",
    "SHUMA_CHALLENGE_SECRET",
    "SHUMA_FORWARDED_IP_SECRET",
    "SHUMA_HEALTH_SECRET",
];

static LAST_EVENTLOG_CLEANUP_HOUR: Lazy<Mutex<u64>> = Lazy::new(|| Mutex::new(0));

fn event_log_retention_hours() -> u64 {
    crate::config::event_log_retention_hours()
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
    // v2 cleanup.
    let v2_prefix = format!("{}:{}:", EVENTLOG_V2_PREFIX, cutoff_hour);
    if let Ok(keys) = store.get_keys() {
        for key in keys {
            if key.starts_with(&v2_prefix) {
                if let Err(e) = store.delete(&key) {
                    eprintln!("[eventlog] failed deleting expired key {}: {:?}", key, e);
                }
            }
        }
    }
}

fn make_v2_event_key(hour: u64, ts: u64) -> String {
    format!(
        "{}:{}:{}-{:016x}",
        EVENTLOG_V2_PREFIX,
        hour,
        ts,
        random::<u64>()
    )
}

fn parse_v2_event_hour(key: &str) -> Option<u64> {
    let mut parts = key.splitn(4, ':');
    match (parts.next(), parts.next(), parts.next()) {
        (Some("eventlog"), Some("v2"), Some(hour)) => hour.parse::<u64>().ok(),
        _ => None,
    }
}

pub fn log_event<S: crate::challenge::KeyValueStore>(store: &S, entry: &EventLogEntry) {
    // Write each event to a distinct immutable key to avoid read-modify-write races.
    let hour = entry.ts / 3600;
    maybe_cleanup_event_logs(store, hour);
    let key = make_v2_event_key(hour, entry.ts);
    match serde_json::to_vec(entry) {
        Ok(payload) => {
            if store.set(&key, &payload).is_err() {
                eprintln!("[log_event] KV error writing {}", key);
            }
        }
        Err(_) => eprintln!(
            "[log_event] serialization error; dropping event for key {}",
            key
        ),
    }
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
            MockStore {
                map: Mutex::new(HashMap::new()),
            }
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
        fn get_keys(&self) -> Result<Vec<String>, ()> {
            let m = self.map.lock().unwrap();
            Ok(m.keys().cloned().collect())
        }
    }

    #[test]
    fn log_event_writes_distinct_v2_records() {
        let store = MockStore::new();
        let now = now_ts();
        let entry = EventLogEntry {
            ts: now,
            event: EventType::AdminAction,
            ip: Some("1.2.3.4".to_string()),
            reason: Some("test".to_string()),
            outcome: Some("ok".to_string()),
            admin: Some("me".to_string()),
        };
        for _ in 0..5 {
            log_event(&store, &entry);
        }
        let hour = now / 3600;
        let prefix = format!("eventlog:v2:{}:", hour);
        let keys: Vec<String> = store
            .map
            .lock()
            .unwrap()
            .keys()
            .cloned()
            .filter(|k| k.starts_with(&prefix))
            .collect();
        assert_eq!(keys.len(), 5);
    }

    #[test]
    fn load_recent_events_includes_v2_records() {
        let store = MockStore::new();
        let now = now_ts();
        let entry = EventLogEntry {
            ts: now,
            event: EventType::AdminAction,
            ip: Some("1.2.3.4".to_string()),
            reason: Some("test".to_string()),
            outcome: Some("ok".to_string()),
            admin: Some("me".to_string()),
        };
        let hour = now / 3600;
        let key = format!("eventlog:v2:{}:{}-deadbeef", hour, now);
        store
            .set(&key, serde_json::to_vec(&entry).unwrap().as_slice())
            .unwrap();

        let events = load_recent_events(&store, now, 1);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].reason.as_deref(), Some("test"));
    }

    #[test]
    fn load_recent_events_ignores_legacy_v1_pages() {
        let store = MockStore::new();
        let now = now_ts();
        let entry = EventLogEntry {
            ts: now,
            event: EventType::AdminAction,
            ip: Some("1.2.3.4".to_string()),
            reason: Some("legacy".to_string()),
            outcome: Some("ok".to_string()),
            admin: Some("me".to_string()),
        };
        let hour = now / 3600;
        let key = format!("eventlog:{}:1", hour);
        let page = vec![entry];
        store
            .set(&key, serde_json::to_vec(&page).unwrap().as_slice())
            .unwrap();

        let events = load_recent_events(&store, now, 1);
        assert!(events.is_empty());
    }

    #[test]
    fn query_u64_param_parses_multi_param_query() {
        let query = "hours=24&limit=500";
        assert_eq!(query_u64_param(query, "hours", 1), 24);
        assert_eq!(query_u64_param(query, "limit", 10), 500);
        assert_eq!(query_u64_param(query, "missing", 42), 42);
    }

    #[test]
    fn is_cdp_event_reason_matches_detection_and_auto_ban() {
        assert!(is_cdp_event_reason("cdp_detected:tier=medium score=0.7"));
        assert!(is_cdp_event_reason("cdp_automation"));
        assert!(!is_cdp_event_reason("maze_crawler"));
    }
}

#[cfg(test)]
mod admin_config_tests {
    use super::*;
    use crate::challenge::KeyValueStore;
    use spin_sdk::http::{Method, Request};
    use std::collections::HashMap;
    use std::sync::Mutex;

    fn make_request(method: Method, path: &str, body: Vec<u8>) -> Request {
        let mut builder = Request::builder();
        builder
            .method(method)
            .uri(path)
            .header("authorization", "Bearer changeme-dev-only-api-key")
            .body(body);
        builder.build()
    }

    struct TestStore {
        map: Mutex<HashMap<String, Vec<u8>>>,
    }

    impl Default for TestStore {
        fn default() -> Self {
            let mut map = HashMap::new();
            let cfg = crate::config::defaults().clone();
            map.insert(
                "config:default".to_string(),
                serde_json::to_vec(&cfg).unwrap(),
            );
            Self {
                map: Mutex::new(map),
            }
        }
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

    impl crate::maze::state::MazeStateStore for TestStore {
        fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ()> {
            crate::challenge::KeyValueStore::get(self, key)
        }

        fn set(&self, key: &str, value: &[u8]) -> Result<(), ()> {
            crate::challenge::KeyValueStore::set(self, key, value)
        }

        fn delete(&self, key: &str) -> Result<(), ()> {
            crate::challenge::KeyValueStore::delete(self, key)
        }
    }

    fn clear_env(keys: &[&str]) {
        for key in keys {
            std::env::remove_var(key);
        }
    }

    #[test]
    fn admin_config_export_returns_non_secret_runtime_values() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_IP_ALLOWLIST", "203.0.113.0/24,198.51.100.8");
        std::env::set_var("SHUMA_ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE", "17");
        std::env::set_var("SHUMA_EVENT_LOG_RETENTION_HOURS", "240");
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        std::env::set_var("SHUMA_KV_STORE_FAIL_OPEN", "false");
        std::env::set_var("SHUMA_ENFORCE_HTTPS", "true");
        std::env::set_var("SHUMA_DEBUG_HEADERS", "true");
        std::env::set_var("SHUMA_RATE_LIMITER_REDIS_URL", "redis://redis:6379");
        std::env::set_var("SHUMA_BAN_STORE_REDIS_URL", "redis://redis:6379");
        std::env::set_var("SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN", "fail_open");
        std::env::set_var("SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH", "fail_closed");
        std::env::set_var("SHUMA_POW_CONFIG_MUTABLE", "true");
        std::env::set_var("SHUMA_CHALLENGE_CONFIG_MUTABLE", "true");
        std::env::set_var("SHUMA_BOTNESS_CONFIG_MUTABLE", "true");

        let store = TestStore::default();
        let mut cfg = crate::config::defaults().clone();
        cfg.rate_limit = 321;
        cfg.honeypots = vec!["/trap-a".to_string(), "/trap-b".to_string()];
        cfg.defence_modes.rate = crate::config::ComposabilityMode::Signal;
        cfg.provider_backends.fingerprint_signal = crate::config::ProviderBackend::External;
        cfg.edge_integration_mode = crate::config::EdgeIntegrationMode::Advisory;
        store
            .set("config:default", &serde_json::to_vec(&cfg).unwrap())
            .unwrap();

        let req = make_request(Method::Get, "/admin/config/export", Vec::new());
        let resp = handle_admin_config_export(&req, &store, "default");
        assert_eq!(*resp.status(), 200u16);

        let body: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
        let env = body.get("env").and_then(|v| v.as_object()).unwrap();
        assert_eq!(env.get("SHUMA_RATE_LIMIT"), Some(&serde_json::json!("321")));
        assert_eq!(
            env.get("SHUMA_HONEYPOTS"),
            Some(&serde_json::json!("[\"/trap-a\",\"/trap-b\"]"))
        );
        assert_eq!(
            env.get("SHUMA_MODE_RATE"),
            Some(&serde_json::json!("signal"))
        );
        assert_eq!(
            env.get("SHUMA_PROVIDER_FINGERPRINT_SIGNAL"),
            Some(&serde_json::json!("external"))
        );
        assert_eq!(
            env.get("SHUMA_EDGE_INTEGRATION_MODE"),
            Some(&serde_json::json!("advisory"))
        );
        assert_eq!(
            env.get("SHUMA_ADMIN_IP_ALLOWLIST"),
            Some(&serde_json::json!("203.0.113.0/24,198.51.100.8"))
        );
        assert_eq!(
            env.get("SHUMA_ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE"),
            Some(&serde_json::json!("17"))
        );
        assert_eq!(
            env.get("SHUMA_EVENT_LOG_RETENTION_HOURS"),
            Some(&serde_json::json!("240"))
        );
        assert_eq!(
            env.get("SHUMA_ADMIN_CONFIG_WRITE_ENABLED"),
            Some(&serde_json::json!("true"))
        );
        assert_eq!(
            env.get("SHUMA_KV_STORE_FAIL_OPEN"),
            Some(&serde_json::json!("false"))
        );
        assert_eq!(
            env.get("SHUMA_ENFORCE_HTTPS"),
            Some(&serde_json::json!("true"))
        );
        assert_eq!(
            env.get("SHUMA_DEBUG_HEADERS"),
            Some(&serde_json::json!("true"))
        );
        assert_eq!(
            env.get("SHUMA_RATE_LIMITER_REDIS_URL"),
            Some(&serde_json::json!("redis://redis:6379"))
        );
        assert_eq!(
            env.get("SHUMA_BAN_STORE_REDIS_URL"),
            Some(&serde_json::json!("redis://redis:6379"))
        );
        assert_eq!(
            env.get("SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN"),
            Some(&serde_json::json!("fail_open"))
        );
        assert_eq!(
            env.get("SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH"),
            Some(&serde_json::json!("fail_closed"))
        );

        let env_text = body.get("env_text").and_then(|v| v.as_str()).unwrap();
        assert!(env_text.contains("SHUMA_RATE_LIMIT=321"));
        assert!(env_text.contains("SHUMA_MODE_RATE=signal"));
        assert!(env_text.contains("SHUMA_PROVIDER_FINGERPRINT_SIGNAL=external"));
        assert!(env_text.contains("SHUMA_ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE=17"));
        assert!(env_text.contains("SHUMA_RATE_LIMITER_REDIS_URL=redis://redis:6379"));
        assert!(env_text.contains("SHUMA_BAN_STORE_REDIS_URL=redis://redis:6379"));
        assert!(env_text.contains("SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN=fail_open"));
        assert!(env_text.contains("SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH=fail_closed"));

        clear_env(&[
            "SHUMA_ADMIN_IP_ALLOWLIST",
            "SHUMA_ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE",
            "SHUMA_EVENT_LOG_RETENTION_HOURS",
            "SHUMA_ADMIN_CONFIG_WRITE_ENABLED",
            "SHUMA_KV_STORE_FAIL_OPEN",
            "SHUMA_ENFORCE_HTTPS",
            "SHUMA_DEBUG_HEADERS",
            "SHUMA_RATE_LIMITER_REDIS_URL",
            "SHUMA_BAN_STORE_REDIS_URL",
            "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN",
            "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH",
            "SHUMA_POW_CONFIG_MUTABLE",
            "SHUMA_CHALLENGE_CONFIG_MUTABLE",
            "SHUMA_BOTNESS_CONFIG_MUTABLE",
        ]);
    }

    #[test]
    fn admin_config_export_omits_secret_values() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_API_KEY", "admin-key-secret");
        std::env::set_var("SHUMA_JS_SECRET", "js-secret");
        std::env::set_var("SHUMA_POW_SECRET", "pow-secret");
        std::env::set_var("SHUMA_CHALLENGE_SECRET", "challenge-secret");
        std::env::set_var("SHUMA_FORWARDED_IP_SECRET", "forwarded-secret");
        std::env::set_var("SHUMA_HEALTH_SECRET", "health-secret");

        let store = TestStore::default();
        let req = make_request(Method::Get, "/admin/config/export", Vec::new());
        let resp = handle_admin_config_export(&req, &store, "default");
        assert_eq!(*resp.status(), 200u16);

        let body: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
        let env = body.get("env").and_then(|v| v.as_object()).unwrap();
        for secret_key in CONFIG_EXPORT_SECRET_KEYS {
            assert!(env.get(secret_key).is_none());
        }

        let env_text = body.get("env_text").and_then(|v| v.as_str()).unwrap();
        for secret_key in CONFIG_EXPORT_SECRET_KEYS {
            assert!(!env_text.contains(&format!("{}=", secret_key)));
        }

        let excluded = body
            .get("excluded_secrets")
            .and_then(|v| v.as_array())
            .unwrap();
        for secret_key in CONFIG_EXPORT_SECRET_KEYS {
            assert!(excluded
                .iter()
                .any(|item| item.as_str() == Some(secret_key)));
        }

        clear_env(&[
            "SHUMA_API_KEY",
            "SHUMA_JS_SECRET",
            "SHUMA_POW_SECRET",
            "SHUMA_CHALLENGE_SECRET",
            "SHUMA_FORWARDED_IP_SECRET",
            "SHUMA_HEALTH_SECRET",
        ]);
    }

    #[test]
    fn admin_config_includes_challenge_fields() {
        let _lock = crate::test_support::lock_env();
        std::env::remove_var("SHUMA_CHALLENGE_CONFIG_MUTABLE");
        std::env::remove_var("SHUMA_BOTNESS_CONFIG_MUTABLE");
        let req = make_request(Method::Get, "/admin/config", Vec::new());
        let store = TestStore::default();
        let resp = handle_admin_config(&req, &store, "default");
        assert_eq!(*resp.status(), 200u16);
        let body: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
        assert!(body.get("challenge_risk_threshold").is_some());
        assert!(body.get("challenge_config_mutable").is_some());
        assert!(body.get("challenge_risk_threshold_default").is_some());
        assert!(body.get("ai_policy_block_training").is_some());
        assert!(body.get("ai_policy_block_search").is_some());
        assert!(body.get("ai_policy_allow_search_engines").is_some());
        assert!(body.get("botness_maze_threshold").is_some());
        assert!(body.get("js_required_enforced").is_some());
        assert!(body.get("botness_weights").is_some());
        assert!(body.get("defence_modes").is_some());
        assert!(body.get("provider_backends").is_some());
        assert!(body.get("edge_integration_mode").is_some());
        assert!(body.get("defence_modes_effective").is_some());
        assert!(body.get("defence_mode_warnings").is_some());
        assert!(body.get("enterprise_multi_instance").is_some());
        assert!(body
            .get("enterprise_unsynced_state_exception_confirmed")
            .is_some());
        assert!(body.get("enterprise_state_guardrail_warnings").is_some());
        assert!(body.get("enterprise_state_guardrail_error").is_some());
        assert!(body.get("botness_config_mutable").is_some());
        assert!(body.get("botness_signal_definitions").is_some());
    }

    #[test]
    fn admin_config_rejects_challenge_update_when_immutable() {
        let _lock = crate::test_support::lock_env();
        let prior_challenge_mutable = std::env::var("SHUMA_CHALLENGE_CONFIG_MUTABLE").ok();
        let prior_botness_mutable = std::env::var("SHUMA_BOTNESS_CONFIG_MUTABLE").ok();
        std::env::set_var("SHUMA_CHALLENGE_CONFIG_MUTABLE", "0");
        std::env::set_var("SHUMA_BOTNESS_CONFIG_MUTABLE", "0");
        let body = br#"{"challenge_risk_threshold":5}"#.to_vec();
        let req = make_request(Method::Post, "/admin/config", body);
        let store = TestStore::default();
        let resp = handle_admin_config(&req, &store, "default");
        assert_eq!(*resp.status(), 403u16);
        if let Some(previous) = prior_challenge_mutable {
            std::env::set_var("SHUMA_CHALLENGE_CONFIG_MUTABLE", previous);
        } else {
            std::env::remove_var("SHUMA_CHALLENGE_CONFIG_MUTABLE");
        }
        if let Some(previous) = prior_botness_mutable {
            std::env::set_var("SHUMA_BOTNESS_CONFIG_MUTABLE", previous);
        } else {
            std::env::remove_var("SHUMA_BOTNESS_CONFIG_MUTABLE");
        }
    }

    #[test]
    fn admin_maze_seed_sources_round_trip_and_manual_refresh() {
        let _lock = crate::test_support::lock_env();
        let store = TestStore::default();
        let mut cfg = crate::config::defaults().clone();
        cfg.maze_seed_provider = crate::config::MazeSeedProvider::Operator;
        cfg.maze_seed_refresh_rate_limit_per_hour = 3;
        cfg.maze_seed_refresh_max_sources = 4;
        store
            .set("config:default", &serde_json::to_vec(&cfg).unwrap())
            .unwrap();

        let post_req = make_request(
            Method::Post,
            "/admin/maze/seeds",
            br#"{
                "sources":[
                    {
                        "id":"headlines",
                        "url":"https://example.com/feed",
                        "title":"Signal routing update",
                        "description":"Metadata-only refresh for maze corpus",
                        "keywords":["maze","checkpoint","budget"],
                        "allow_seed_use":true,
                        "robots_allowed":true
                    }
                ]
            }"#
            .to_vec(),
        );
        let post_resp = handle_admin_maze_seed_sources(&post_req, &store, "default");
        assert_eq!(*post_resp.status(), 200u16);

        let get_req = make_request(Method::Get, "/admin/maze/seeds", Vec::new());
        let get_resp = handle_admin_maze_seed_sources(&get_req, &store, "default");
        assert_eq!(*get_resp.status(), 200u16);
        let get_json: serde_json::Value = serde_json::from_slice(get_resp.body()).unwrap();
        assert_eq!(
            get_json
                .get("sources")
                .and_then(|v| v.as_array())
                .map(|v| v.len()),
            Some(1)
        );

        let refresh_req = make_request(Method::Post, "/admin/maze/seeds/refresh", Vec::new());
        let refresh_resp = handle_admin_maze_seed_refresh(&refresh_req, &store, "default");
        assert_eq!(*refresh_resp.status(), 200u16);
        let refresh_json: serde_json::Value = serde_json::from_slice(refresh_resp.body()).unwrap();
        assert_eq!(
            refresh_json.get("refreshed"),
            Some(&serde_json::Value::Bool(true))
        );
        assert!(
            refresh_json
                .get("term_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                > 0
        );
    }

    #[test]
    fn admin_maze_seed_refresh_requires_operator_provider() {
        let _lock = crate::test_support::lock_env();
        let store = TestStore::default();
        let refresh_req = make_request(Method::Post, "/admin/maze/seeds/refresh", Vec::new());
        let refresh_resp = handle_admin_maze_seed_refresh(&refresh_req, &store, "default");
        assert_eq!(*refresh_resp.status(), 409u16);
    }

    #[test]
    fn admin_config_rejects_updates_when_admin_config_write_disabled() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "false");
        let body = br#"{"test_mode":true}"#.to_vec();
        let req = make_request(Method::Post, "/admin/config", body);
        let store = TestStore::default();
        let resp = handle_admin_config(&req, &store, "default");
        assert_eq!(*resp.status(), 403u16);
        let msg = String::from_utf8_lossy(resp.body());
        assert!(msg.contains("SHUMA_ADMIN_CONFIG_WRITE_ENABLED=false"));
        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
    }

    #[test]
    fn admin_config_updates_geo_policy_lists() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        std::env::remove_var("SHUMA_GEO_RISK_COUNTRIES");
        std::env::remove_var("SHUMA_GEO_ALLOW_COUNTRIES");
        std::env::remove_var("SHUMA_GEO_CHALLENGE_COUNTRIES");
        std::env::remove_var("SHUMA_GEO_MAZE_COUNTRIES");
        std::env::remove_var("SHUMA_GEO_BLOCK_COUNTRIES");
        let store = TestStore::default();

        let body = br#"{
          "geo_risk": ["us", "CN", "us"],
          "geo_allow": ["gb"],
          "geo_challenge": ["br"],
          "geo_maze": ["ru"],
          "geo_block": ["kp"]
        }"#
        .to_vec();
        let post_req = make_request(Method::Post, "/admin/config", body);
        let post_resp = handle_admin_config(&post_req, &store, "default");
        assert_eq!(*post_resp.status(), 200u16);
        let post_json: serde_json::Value = serde_json::from_slice(post_resp.body()).unwrap();
        let cfg = post_json.get("config").unwrap();

        assert_eq!(
            cfg.get("geo_risk").unwrap(),
            &serde_json::json!(["US", "CN"])
        );
        assert_eq!(cfg.get("geo_allow").unwrap(), &serde_json::json!(["GB"]));
        assert_eq!(
            cfg.get("geo_challenge").unwrap(),
            &serde_json::json!(["BR"])
        );
        assert_eq!(cfg.get("geo_maze").unwrap(), &serde_json::json!(["RU"]));
        assert_eq!(cfg.get("geo_block").unwrap(), &serde_json::json!(["KP"]));

        let get_req = make_request(Method::Get, "/admin/config", Vec::new());
        let get_resp = handle_admin_config(&get_req, &store, "default");
        assert_eq!(*get_resp.status(), 200u16);
        let get_json: serde_json::Value = serde_json::from_slice(get_resp.body()).unwrap();
        assert_eq!(
            get_json.get("geo_risk").unwrap(),
            &serde_json::json!(["US", "CN"])
        );
        assert_eq!(
            get_json.get("geo_allow").unwrap(),
            &serde_json::json!(["GB"])
        );
        assert_eq!(
            get_json.get("geo_challenge").unwrap(),
            &serde_json::json!(["BR"])
        );
        assert_eq!(
            get_json.get("geo_maze").unwrap(),
            &serde_json::json!(["RU"])
        );
        assert_eq!(
            get_json.get("geo_block").unwrap(),
            &serde_json::json!(["KP"])
        );
        std::env::remove_var("SHUMA_GEO_RISK_COUNTRIES");
        std::env::remove_var("SHUMA_GEO_ALLOW_COUNTRIES");
        std::env::remove_var("SHUMA_GEO_CHALLENGE_COUNTRIES");
        std::env::remove_var("SHUMA_GEO_MAZE_COUNTRIES");
        std::env::remove_var("SHUMA_GEO_BLOCK_COUNTRIES");
        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
    }

    #[test]
    fn admin_config_rejects_non_iso_geo_country_codes() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        let store = TestStore::default();
        let body = br#"{"geo_risk": ["US", "ZZ"]}"#.to_vec();
        let post_req = make_request(Method::Post, "/admin/config", body);
        let post_resp = handle_admin_config(&post_req, &store, "default");
        assert_eq!(*post_resp.status(), 400u16);
        let msg = String::from_utf8_lossy(post_resp.body());
        assert!(msg.contains("invalid country code"));
        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
    }

    #[test]
    fn admin_config_updates_js_required_enforced_flag() {
        let _lock = crate::test_support::lock_env();
        let prior_js_required_env = std::env::var("SHUMA_JS_REQUIRED_ENFORCED").ok();
        std::env::remove_var("SHUMA_JS_REQUIRED_ENFORCED");
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        let store = TestStore::default();

        let post_req = make_request(
            Method::Post,
            "/admin/config",
            br#"{"js_required_enforced":false}"#.to_vec(),
        );
        let post_resp = handle_admin_config(&post_req, &store, "default");
        assert_eq!(*post_resp.status(), 200u16);
        let post_json: serde_json::Value = serde_json::from_slice(post_resp.body()).unwrap();
        let cfg = post_json.get("config").unwrap();
        assert_eq!(
            cfg.get("js_required_enforced"),
            Some(&serde_json::Value::Bool(false))
        );

        let saved_bytes = store.get("config:default").unwrap().unwrap();
        let saved_cfg: crate::config::Config = serde_json::from_slice(&saved_bytes).unwrap();
        assert!(!saved_cfg.js_required_enforced);

        if let Some(previous) = prior_js_required_env {
            std::env::set_var("SHUMA_JS_REQUIRED_ENFORCED", previous);
        } else {
            std::env::remove_var("SHUMA_JS_REQUIRED_ENFORCED");
        }
        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
    }

    #[test]
    fn admin_config_updates_ai_policy_fields_via_first_class_keys() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        let store = TestStore::default();

        let post_req = make_request(
            Method::Post,
            "/admin/config",
            br#"{
                "ai_policy_block_training": false,
                "ai_policy_block_search": true,
                "ai_policy_allow_search_engines": false
            }"#
            .to_vec(),
        );
        let post_resp = handle_admin_config(&post_req, &store, "default");
        assert_eq!(*post_resp.status(), 200u16);
        let post_json: serde_json::Value = serde_json::from_slice(post_resp.body()).unwrap();
        let cfg = post_json.get("config").expect("config payload should exist");
        assert_eq!(
            cfg.get("ai_policy_block_training"),
            Some(&serde_json::Value::Bool(false))
        );
        assert_eq!(
            cfg.get("ai_policy_block_search"),
            Some(&serde_json::Value::Bool(true))
        );
        assert_eq!(
            cfg.get("ai_policy_allow_search_engines"),
            Some(&serde_json::Value::Bool(false))
        );
        assert_eq!(
            cfg.get("robots_block_ai_training"),
            Some(&serde_json::Value::Bool(false))
        );
        assert_eq!(
            cfg.get("robots_block_ai_search"),
            Some(&serde_json::Value::Bool(true))
        );
        assert_eq!(
            cfg.get("robots_allow_search_engines"),
            Some(&serde_json::Value::Bool(false))
        );

        let saved_bytes = store.get("config:default").unwrap().unwrap();
        let saved_cfg: crate::config::Config = serde_json::from_slice(&saved_bytes).unwrap();
        assert!(!saved_cfg.robots_block_ai_training);
        assert!(saved_cfg.robots_block_ai_search);
        assert!(!saved_cfg.robots_allow_search_engines);

        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
    }

    #[test]
    fn admin_config_rejects_out_of_range_rate_limit() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        let store = TestStore::default();

        let post_req = make_request(
            Method::Post,
            "/admin/config",
            br#"{"rate_limit":0}"#.to_vec(),
        );
        let post_resp = handle_admin_config(&post_req, &store, "default");
        assert_eq!(*post_resp.status(), 400u16);
        let msg = String::from_utf8_lossy(post_resp.body());
        assert!(msg.contains("rate_limit out of range"));

        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
    }

    #[test]
    fn admin_config_updates_defence_modes_when_botness_mutable() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        std::env::set_var("SHUMA_BOTNESS_CONFIG_MUTABLE", "true");
        let store = TestStore::default();

        let post_req = make_request(
            Method::Post,
            "/admin/config",
            br#"{"defence_modes":{"rate":"signal","geo":"enforce","js":"off"}}"#.to_vec(),
        );
        let post_resp = handle_admin_config(&post_req, &store, "default");
        assert_eq!(*post_resp.status(), 200u16);
        let post_json: serde_json::Value = serde_json::from_slice(post_resp.body()).unwrap();
        let cfg = post_json.get("config").unwrap();
        assert_eq!(
            cfg.get("defence_modes"),
            Some(&serde_json::json!({"rate":"signal","geo":"enforce","js":"off"}))
        );

        let saved_bytes = store.get("config:default").unwrap().unwrap();
        let saved_cfg: crate::config::Config = serde_json::from_slice(&saved_bytes).unwrap();
        assert_eq!(
            saved_cfg.defence_modes.rate,
            crate::config::ComposabilityMode::Signal
        );
        assert_eq!(
            saved_cfg.defence_modes.geo,
            crate::config::ComposabilityMode::Enforce
        );
        assert_eq!(
            saved_cfg.defence_modes.js,
            crate::config::ComposabilityMode::Off
        );

        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
        std::env::remove_var("SHUMA_BOTNESS_CONFIG_MUTABLE");
    }

    #[test]
    fn admin_config_rejects_invalid_defence_mode_value() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        std::env::set_var("SHUMA_BOTNESS_CONFIG_MUTABLE", "true");
        let store = TestStore::default();

        let post_req = make_request(
            Method::Post,
            "/admin/config",
            br#"{"defence_modes":{"rate":"invalid"}}"#.to_vec(),
        );
        let post_resp = handle_admin_config(&post_req, &store, "default");
        assert_eq!(*post_resp.status(), 400u16);
        let msg = String::from_utf8_lossy(post_resp.body());
        assert!(msg.contains("defence_modes.rate must be one of"));

        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
        std::env::remove_var("SHUMA_BOTNESS_CONFIG_MUTABLE");
    }

    #[test]
    fn admin_config_rejects_unknown_defence_mode_key() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        std::env::set_var("SHUMA_BOTNESS_CONFIG_MUTABLE", "true");
        let store = TestStore::default();

        let post_req = make_request(
            Method::Post,
            "/admin/config",
            br#"{"defence_modes":{"rate":"both","foo":"off"}}"#.to_vec(),
        );
        let post_resp = handle_admin_config(&post_req, &store, "default");
        assert_eq!(*post_resp.status(), 400u16);
        let msg = String::from_utf8_lossy(post_resp.body());
        assert!(msg.contains("defence_modes.foo is not supported"));

        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
        std::env::remove_var("SHUMA_BOTNESS_CONFIG_MUTABLE");
    }

    #[test]
    fn admin_config_updates_provider_backends_and_edge_mode() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        let store = TestStore::default();

        let post_req = make_request(
            Method::Post,
            "/admin/config",
            br#"{
                "provider_backends": {
                    "rate_limiter": "external",
                    "ban_store": "external",
                    "fingerprint_signal": "external"
                },
                "edge_integration_mode": "advisory"
            }"#
            .to_vec(),
        );
        let post_resp = handle_admin_config(&post_req, &store, "default");
        assert_eq!(*post_resp.status(), 200u16);
        let post_json: serde_json::Value = serde_json::from_slice(post_resp.body()).unwrap();
        let cfg = post_json.get("config").unwrap();
        assert_eq!(
            cfg.get("provider_backends"),
            Some(&serde_json::json!({
                "rate_limiter": "external",
                "ban_store": "external",
                "challenge_engine": "internal",
                "maze_tarpit": "internal",
                "fingerprint_signal": "external"
            }))
        );
        assert_eq!(
            cfg.get("edge_integration_mode"),
            Some(&serde_json::json!("advisory"))
        );

        let saved_bytes = store.get("config:default").unwrap().unwrap();
        let saved_cfg: crate::config::Config = serde_json::from_slice(&saved_bytes).unwrap();
        assert_eq!(
            saved_cfg.provider_backends.rate_limiter,
            crate::config::ProviderBackend::External
        );
        assert_eq!(
            saved_cfg.provider_backends.ban_store,
            crate::config::ProviderBackend::External
        );
        assert_eq!(
            saved_cfg.provider_backends.fingerprint_signal,
            crate::config::ProviderBackend::External
        );
        assert_eq!(
            saved_cfg.edge_integration_mode,
            crate::config::EdgeIntegrationMode::Advisory
        );

        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
    }

    #[test]
    fn admin_config_rejects_invalid_provider_backend_value() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        let store = TestStore::default();

        let post_req = make_request(
            Method::Post,
            "/admin/config",
            br#"{"provider_backends":{"rate_limiter":"invalid"}}"#.to_vec(),
        );
        let post_resp = handle_admin_config(&post_req, &store, "default");
        assert_eq!(*post_resp.status(), 400u16);
        let msg = String::from_utf8_lossy(post_resp.body());
        assert!(msg.contains("provider_backends.rate_limiter must be one of"));

        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
    }

    #[test]
    fn admin_config_rejects_invalid_edge_integration_mode() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        let store = TestStore::default();

        let post_req = make_request(
            Method::Post,
            "/admin/config",
            br#"{"edge_integration_mode":"invalid"}"#.to_vec(),
        );
        let post_resp = handle_admin_config(&post_req, &store, "default");
        assert_eq!(*post_resp.status(), 400u16);
        let msg = String::from_utf8_lossy(post_resp.body());
        assert!(msg.contains("edge_integration_mode must be one of"));

        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
    }

    #[test]
    fn admin_config_rejects_unknown_provider_backend_key() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        let store = TestStore::default();

        let post_req = make_request(
            Method::Post,
            "/admin/config",
            br#"{"provider_backends":{"fingerprint_signal":"external","unknown":"external"}}"#
                .to_vec(),
        );
        let post_resp = handle_admin_config(&post_req, &store, "default");
        assert_eq!(*post_resp.status(), 400u16);
        let msg = String::from_utf8_lossy(post_resp.body());
        assert!(msg.contains("provider_backends.unknown is not supported"));

        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
    }
}

#[cfg(test)]
mod admin_auth_tests {
    use super::*;
    use spin_sdk::http::{Method, Request};
    use std::collections::HashMap;
    use std::sync::Mutex;

    #[derive(Default)]
    struct TestStore {
        map: Mutex<HashMap<String, Vec<u8>>>,
    }

    impl crate::challenge::KeyValueStore for TestStore {
        fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ()> {
            Ok(self.map.lock().unwrap().get(key).cloned())
        }

        fn set(&self, key: &str, value: &[u8]) -> Result<(), ()> {
            self.map
                .lock()
                .unwrap()
                .insert(key.to_string(), value.to_vec());
            Ok(())
        }

        fn delete(&self, key: &str) -> Result<(), ()> {
            self.map.lock().unwrap().remove(key);
            Ok(())
        }
    }

    fn login_request(api_key: &str) -> Request {
        let mut builder = Request::builder();
        builder
            .method(Method::Post)
            .uri("/admin/login")
            .header("content-type", "application/json")
            .body(format!(r#"{{"api_key":"{}"}}"#, api_key).into_bytes());
        builder.build()
    }

    fn logout_request() -> Request {
        let mut builder = Request::builder();
        builder.method(Method::Post).uri("/admin/logout");
        builder.build()
    }

    #[test]
    fn login_invalid_api_key_is_rate_limited() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_API_KEY", "test-admin-key");
        std::env::set_var("SHUMA_ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE", "2");
        let store = TestStore::default();

        let req = login_request("wrong-key");
        let first = handle_admin_login(&req, &store);
        assert_eq!(*first.status(), 401u16);

        let second = handle_admin_login(&req, &store);
        assert_eq!(*second.status(), 401u16);

        let third = handle_admin_login(&req, &store);
        assert_eq!(*third.status(), 429u16);

        std::env::remove_var("SHUMA_ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE");
        std::env::remove_var("SHUMA_API_KEY");
    }

    #[test]
    fn logout_unauthorized_is_rate_limited() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE", "1");
        let store = TestStore::default();
        let req = logout_request();

        let first = handle_admin_logout(&req, &store);
        assert_eq!(*first.status(), 401u16);

        let second = handle_admin_logout(&req, &store);
        assert_eq!(*second.status(), 429u16);

        std::env::remove_var("SHUMA_ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE");
    }

    #[test]
    fn write_access_matrix_covers_only_mutating_admin_routes() {
        assert!(request_requires_admin_write("/admin/config", &Method::Post));
        assert!(request_requires_admin_write("/admin/ban", &Method::Post));
        assert!(request_requires_admin_write("/admin/unban", &Method::Post));
        assert!(!request_requires_admin_write(
            "/admin/events",
            &Method::Post
        ));
        assert!(!request_requires_admin_write("/admin/config", &Method::Get));
        assert!(!request_requires_admin_write(
            "/admin/analytics",
            &Method::Get
        ));
    }
}

/// Utility to get current unix timestamp
pub fn now_ts() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
// src/admin.rs
// Admin API endpoints for WASM Bot Defence
// Provides HTTP endpoints for ban management and analytics, protected by API key auth.

use serde_json::json;
use spin_sdk::http::{Method, Request, Response};
use spin_sdk::key_value::Store;

const ADMIN_BAN_DURATION_MIN: u64 = 60;
const ADMIN_BAN_DURATION_MAX: u64 = 31_536_000;

/// Returns true if the path is a valid admin endpoint (prevents path traversal/abuse).
fn sanitize_path(path: &str) -> bool {
    matches!(
        path,
        "/admin"
            | "/admin/login"
            | "/admin/session"
            | "/admin/logout"
            | "/admin/ban"
            | "/admin/unban"
            | "/admin/analytics"
            | "/admin/events"
            | "/admin/config"
            | "/admin/config/export"
            | "/admin/maze"
            | "/admin/maze/seeds"
            | "/admin/maze/seeds/refresh"
            | "/admin/robots"
            | "/admin/cdp"
            | "/admin/cdp/events"
    )
}

fn session_cookie_value(session_id: &str) -> String {
    let max_age = crate::admin::auth::admin_session_ttl_seconds();
    let secure = if crate::config::https_enforced() {
        "; Secure"
    } else {
        ""
    };
    format!(
        "{}={}; Path=/; HttpOnly; SameSite=Strict; Max-Age={}{}",
        crate::admin::auth::admin_session_cookie_name(),
        session_id,
        max_age,
        secure
    )
}

fn clear_session_cookie_value() -> String {
    let secure = if crate::config::https_enforced() {
        "; Secure"
    } else {
        ""
    };
    format!(
        "{}=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0{}",
        crate::admin::auth::admin_session_cookie_name(),
        secure
    )
}

fn too_many_admin_auth_attempts_response() -> Response {
    Response::builder()
        .status(429)
        .header("Retry-After", "60")
        .header("Cache-Control", "no-store")
        .body("Too Many Requests")
        .build()
}

fn request_requires_admin_write(path: &str, method: &Method) -> bool {
    if !matches!(
        method,
        Method::Post | Method::Put | Method::Patch | Method::Delete
    ) {
        return false;
    }
    matches!(
        path,
        "/admin/ban"
            | "/admin/unban"
            | "/admin/config"
            | "/admin/maze/seeds"
            | "/admin/maze/seeds/refresh"
    )
}

fn log_admin_write_denied<S: crate::challenge::KeyValueStore>(
    store: &S,
    req: &Request,
    path: &str,
    auth: &crate::admin::auth::AdminAuthResult,
) {
    log_event(
        store,
        &EventLogEntry {
            ts: now_ts(),
            event: EventType::AdminAction,
            ip: None,
            reason: Some("admin_write_denied".to_string()),
            outcome: Some(format!(
                "path={} method={} access={}",
                path,
                req.method(),
                auth.access_label()
            )),
            admin: Some(auth.audit_actor_label().to_string()),
        },
    );
}

fn register_admin_auth_failure_with_selected_rate_limiter(
    store: &Store,
    req: &Request,
    scope: crate::admin::auth::AdminAuthFailureScope,
    provider_registry: Option<&crate::providers::registry::ProviderRegistry>,
) -> bool {
    if let Some(registry) = provider_registry {
        return crate::admin::auth::register_admin_auth_failure_with_provider(
            store, req, scope, registry,
        );
    }
    crate::admin::auth::register_admin_auth_failure(store, req, scope)
}

fn handle_admin_login_with_failure_handler<S, F>(
    req: &Request,
    store: &S,
    mut register_failure: F,
) -> Response
where
    S: crate::challenge::KeyValueStore,
    F: FnMut() -> bool,
{
    if req.method() != &spin_sdk::http::Method::Post {
        return Response::new(405, "Method Not Allowed");
    }

    let json = match crate::request_validation::parse_json_body(req.body(), 2048) {
        Ok(v) => v,
        Err(msg) => return Response::new(400, msg),
    };
    let Some(api_key) = json.get("api_key").and_then(|v| v.as_str()) else {
        return Response::new(400, "Bad Request: api_key is required");
    };

    if !crate::admin::auth::verify_admin_api_key_candidate(api_key) {
        if register_failure() {
            return too_many_admin_auth_attempts_response();
        }
        return Response::new(401, "Unauthorized");
    }

    let (session_id, csrf_token, expires_at) = match crate::admin::auth::create_admin_session(store)
    {
        Ok(v) => v,
        Err(_) => return Response::new(500, "Key-value store error"),
    };

    let body = serde_json::to_string(&json!({
        "authenticated": true,
        "csrf_token": csrf_token,
        "expires_at": expires_at
    }))
    .unwrap();
    Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .header("Cache-Control", "no-store")
        .header("Set-Cookie", session_cookie_value(&session_id))
        .body(body)
        .build()
}

#[cfg(test)]
fn handle_admin_login<S: crate::challenge::KeyValueStore>(req: &Request, store: &S) -> Response {
    handle_admin_login_with_failure_handler(req, store, || {
        crate::admin::auth::register_admin_auth_failure(
            store,
            req,
            crate::admin::auth::AdminAuthFailureScope::Login,
        )
    })
}

fn handle_admin_session<S: crate::challenge::KeyValueStore>(req: &Request, store: &S) -> Response {
    if req.method() != &spin_sdk::http::Method::Get {
        return Response::new(405, "Method Not Allowed");
    }

    let auth = crate::admin::auth::authenticate_admin(req, store);
    let (authenticated, method, csrf_token, access) = match auth.method {
        Some(crate::admin::auth::AdminAuthMethod::SessionCookie) => (
            true,
            "session",
            auth.csrf_token.clone(),
            crate::admin::auth::AdminAccessLevel::ReadWrite.as_str(),
        ),
        Some(crate::admin::auth::AdminAuthMethod::BearerToken) => {
            (true, "bearer", None, auth.access_label())
        }
        None => (false, "none", None, "none"),
    };
    let body = serde_json::to_string(&json!({
        "authenticated": authenticated,
        "method": method,
        "csrf_token": csrf_token,
        "access": access
    }))
    .unwrap();
    Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .header("Cache-Control", "no-store")
        .body(body)
        .build()
}

fn handle_admin_logout_with_failure_handler<S, F>(
    req: &Request,
    store: &S,
    mut register_failure: F,
) -> Response
where
    S: crate::challenge::KeyValueStore,
    F: FnMut() -> bool,
{
    if req.method() != &spin_sdk::http::Method::Post {
        return Response::new(405, "Method Not Allowed");
    }

    let auth = crate::admin::auth::authenticate_admin(req, store);
    if !auth.is_authorized() {
        if register_failure() {
            return too_many_admin_auth_attempts_response();
        }
        return Response::new(401, "Unauthorized: Invalid or missing API key");
    }
    if auth.requires_csrf(req) {
        let expected = auth.csrf_token.as_deref().unwrap_or("");
        if !crate::admin::auth::validate_session_csrf(req, expected) {
            return Response::new(403, "Forbidden");
        }
    }

    if let Err(e) = crate::admin::auth::clear_admin_session(store, req) {
        eprintln!("[admin] failed to clear admin session on logout: {:?}", e);
    }
    let body = serde_json::to_string(&json!({ "ok": true })).unwrap();
    Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .header("Cache-Control", "no-store")
        .header("Set-Cookie", clear_session_cookie_value())
        .body(body)
        .build()
}

#[cfg(test)]
fn handle_admin_logout<S: crate::challenge::KeyValueStore>(req: &Request, store: &S) -> Response {
    handle_admin_logout_with_failure_handler(req, store, || {
        crate::admin::auth::register_admin_auth_failure(
            store,
            req,
            crate::admin::auth::AdminAuthFailureScope::Endpoint,
        )
    })
}

fn query_u64_param(query: &str, key: &str, default: u64) -> u64 {
    query
        .split('&')
        .find_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            let k = parts.next()?;
            let v = parts.next().unwrap_or("");
            if k == key {
                v.parse::<u64>().ok()
            } else {
                None
            }
        })
        .unwrap_or(default)
}

fn load_recent_events<S: crate::challenge::KeyValueStore>(
    store: &S,
    now: u64,
    hours: u64,
) -> Vec<EventLogEntry> {
    let mut events: Vec<EventLogEntry> = Vec::new();
    let window_start = now.saturating_sub(hours.saturating_mul(3600));
    let window_start_hour = window_start / 3600;
    let now_hour = now / 3600;

    // v2 immutable records.
    if let Ok(keys) = store.get_keys() {
        for key in keys {
            let Some(event_hour) = parse_v2_event_hour(&key) else {
                continue;
            };
            if event_hour < window_start_hour || event_hour > now_hour {
                continue;
            }
            if let Ok(Some(val)) = store.get(&key) {
                if let Ok(entry) = serde_json::from_slice::<EventLogEntry>(&val) {
                    if entry.ts >= window_start {
                        events.push(entry);
                    }
                }
            }
        }
    }

    events
}

fn is_cdp_event_reason(reason: &str) -> bool {
    let lowered = reason.to_lowercase();
    lowered.starts_with("cdp_detected:") || lowered == "cdp_automation"
}

fn challenge_threshold_default() -> u8 {
    crate::config::defaults().challenge_risk_threshold
}

fn maze_threshold_default() -> u8 {
    crate::config::defaults().botness_maze_threshold
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

fn bool_env(value: bool) -> &'static str {
    if value {
        "true"
    } else {
        "false"
    }
}

fn json_env<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value).unwrap()
}

fn config_export_env_entries(cfg: &crate::config::Config) -> Vec<(String, String)> {
    vec![
        (
            "SHUMA_ADMIN_IP_ALLOWLIST".to_string(),
            std::env::var("SHUMA_ADMIN_IP_ALLOWLIST").unwrap_or_default(),
        ),
        (
            "SHUMA_ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE".to_string(),
            crate::admin::auth::admin_auth_failure_limit_per_minute().to_string(),
        ),
        (
            "SHUMA_EVENT_LOG_RETENTION_HOURS".to_string(),
            crate::config::event_log_retention_hours().to_string(),
        ),
        (
            "SHUMA_ADMIN_CONFIG_WRITE_ENABLED".to_string(),
            bool_env(crate::config::admin_config_write_enabled()).to_string(),
        ),
        (
            "SHUMA_KV_STORE_FAIL_OPEN".to_string(),
            bool_env(crate::config::kv_store_fail_open()).to_string(),
        ),
        (
            "SHUMA_ENFORCE_HTTPS".to_string(),
            bool_env(crate::config::https_enforced()).to_string(),
        ),
        (
            "SHUMA_DEBUG_HEADERS".to_string(),
            bool_env(crate::config::debug_headers_enabled()).to_string(),
        ),
        (
            "SHUMA_RATE_LIMITER_REDIS_URL".to_string(),
            crate::config::rate_limiter_redis_url().unwrap_or_default(),
        ),
        (
            "SHUMA_BAN_STORE_REDIS_URL".to_string(),
            crate::config::ban_store_redis_url().unwrap_or_default(),
        ),
        (
            "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN".to_string(),
            crate::config::rate_limiter_outage_mode_main()
                .as_str()
                .to_string(),
        ),
        (
            "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH".to_string(),
            crate::config::rate_limiter_outage_mode_admin_auth()
                .as_str()
                .to_string(),
        ),
        (
            "SHUMA_POW_CONFIG_MUTABLE".to_string(),
            bool_env(crate::config::pow_config_mutable()).to_string(),
        ),
        (
            "SHUMA_CHALLENGE_CONFIG_MUTABLE".to_string(),
            bool_env(crate::config::challenge_config_mutable()).to_string(),
        ),
        (
            "SHUMA_BOTNESS_CONFIG_MUTABLE".to_string(),
            bool_env(crate::config::botness_config_mutable()).to_string(),
        ),
        (
            "SHUMA_TEST_MODE".to_string(),
            bool_env(cfg.test_mode).to_string(),
        ),
        (
            "SHUMA_JS_REQUIRED_ENFORCED".to_string(),
            bool_env(cfg.js_required_enforced).to_string(),
        ),
        (
            "SHUMA_MODE_RATE".to_string(),
            cfg.defence_modes.rate.as_str().to_string(),
        ),
        (
            "SHUMA_MODE_GEO".to_string(),
            cfg.defence_modes.geo.as_str().to_string(),
        ),
        (
            "SHUMA_MODE_JS".to_string(),
            cfg.defence_modes.js.as_str().to_string(),
        ),
        (
            "SHUMA_PROVIDER_RATE_LIMITER".to_string(),
            cfg.provider_backends.rate_limiter.as_str().to_string(),
        ),
        (
            "SHUMA_PROVIDER_BAN_STORE".to_string(),
            cfg.provider_backends.ban_store.as_str().to_string(),
        ),
        (
            "SHUMA_PROVIDER_CHALLENGE_ENGINE".to_string(),
            cfg.provider_backends.challenge_engine.as_str().to_string(),
        ),
        (
            "SHUMA_PROVIDER_MAZE_TARPIT".to_string(),
            cfg.provider_backends.maze_tarpit.as_str().to_string(),
        ),
        (
            "SHUMA_PROVIDER_FINGERPRINT_SIGNAL".to_string(),
            cfg.provider_backends
                .fingerprint_signal
                .as_str()
                .to_string(),
        ),
        (
            "SHUMA_EDGE_INTEGRATION_MODE".to_string(),
            cfg.edge_integration_mode.as_str().to_string(),
        ),
        (
            "SHUMA_POW_ENABLED".to_string(),
            bool_env(cfg.pow_enabled).to_string(),
        ),
        (
            "SHUMA_POW_DIFFICULTY".to_string(),
            cfg.pow_difficulty.to_string(),
        ),
        (
            "SHUMA_POW_TTL_SECONDS".to_string(),
            cfg.pow_ttl_seconds.to_string(),
        ),
        (
            "SHUMA_CHALLENGE_TRANSFORM_COUNT".to_string(),
            cfg.challenge_transform_count.to_string(),
        ),
        (
            "SHUMA_CHALLENGE_RISK_THRESHOLD".to_string(),
            cfg.challenge_risk_threshold.to_string(),
        ),
        (
            "SHUMA_BOTNESS_MAZE_THRESHOLD".to_string(),
            cfg.botness_maze_threshold.to_string(),
        ),
        (
            "SHUMA_BOTNESS_WEIGHT_JS_REQUIRED".to_string(),
            cfg.botness_weights.js_required.to_string(),
        ),
        (
            "SHUMA_BOTNESS_WEIGHT_GEO_RISK".to_string(),
            cfg.botness_weights.geo_risk.to_string(),
        ),
        (
            "SHUMA_BOTNESS_WEIGHT_RATE_MEDIUM".to_string(),
            cfg.botness_weights.rate_medium.to_string(),
        ),
        (
            "SHUMA_BOTNESS_WEIGHT_RATE_HIGH".to_string(),
            cfg.botness_weights.rate_high.to_string(),
        ),
        (
            "SHUMA_BOTNESS_WEIGHT_MAZE_BEHAVIOR".to_string(),
            cfg.botness_weights.maze_behavior.to_string(),
        ),
        (
            "SHUMA_BAN_DURATION".to_string(),
            cfg.ban_duration.to_string(),
        ),
        (
            "SHUMA_BAN_DURATION_HONEYPOT".to_string(),
            cfg.ban_durations.honeypot.to_string(),
        ),
        (
            "SHUMA_BAN_DURATION_RATE_LIMIT".to_string(),
            cfg.ban_durations.rate_limit.to_string(),
        ),
        (
            "SHUMA_BAN_DURATION_BROWSER".to_string(),
            cfg.ban_durations.browser.to_string(),
        ),
        (
            "SHUMA_BAN_DURATION_ADMIN".to_string(),
            cfg.ban_durations.admin.to_string(),
        ),
        (
            "SHUMA_BAN_DURATION_CDP".to_string(),
            cfg.ban_durations.cdp.to_string(),
        ),
        ("SHUMA_RATE_LIMIT".to_string(), cfg.rate_limit.to_string()),
        ("SHUMA_HONEYPOTS".to_string(), json_env(&cfg.honeypots)),
        (
            "SHUMA_BROWSER_BLOCK".to_string(),
            json_env(&cfg.browser_block),
        ),
        (
            "SHUMA_BROWSER_WHITELIST".to_string(),
            json_env(&cfg.browser_whitelist),
        ),
        (
            "SHUMA_GEO_RISK_COUNTRIES".to_string(),
            json_env(&cfg.geo_risk),
        ),
        (
            "SHUMA_GEO_ALLOW_COUNTRIES".to_string(),
            json_env(&cfg.geo_allow),
        ),
        (
            "SHUMA_GEO_CHALLENGE_COUNTRIES".to_string(),
            json_env(&cfg.geo_challenge),
        ),
        (
            "SHUMA_GEO_MAZE_COUNTRIES".to_string(),
            json_env(&cfg.geo_maze),
        ),
        (
            "SHUMA_GEO_BLOCK_COUNTRIES".to_string(),
            json_env(&cfg.geo_block),
        ),
        ("SHUMA_WHITELIST".to_string(), json_env(&cfg.whitelist)),
        (
            "SHUMA_PATH_WHITELIST".to_string(),
            json_env(&cfg.path_whitelist),
        ),
        (
            "SHUMA_MAZE_ENABLED".to_string(),
            bool_env(cfg.maze_enabled).to_string(),
        ),
        (
            "SHUMA_MAZE_AUTO_BAN".to_string(),
            bool_env(cfg.maze_auto_ban).to_string(),
        ),
        (
            "SHUMA_MAZE_AUTO_BAN_THRESHOLD".to_string(),
            cfg.maze_auto_ban_threshold.to_string(),
        ),
        (
            "SHUMA_MAZE_ROLLOUT_PHASE".to_string(),
            cfg.maze_rollout_phase.as_str().to_string(),
        ),
        (
            "SHUMA_MAZE_TOKEN_TTL_SECONDS".to_string(),
            cfg.maze_token_ttl_seconds.to_string(),
        ),
        (
            "SHUMA_MAZE_TOKEN_MAX_DEPTH".to_string(),
            cfg.maze_token_max_depth.to_string(),
        ),
        (
            "SHUMA_MAZE_TOKEN_BRANCH_BUDGET".to_string(),
            cfg.maze_token_branch_budget.to_string(),
        ),
        (
            "SHUMA_MAZE_REPLAY_TTL_SECONDS".to_string(),
            cfg.maze_replay_ttl_seconds.to_string(),
        ),
        (
            "SHUMA_MAZE_ENTROPY_WINDOW_SECONDS".to_string(),
            cfg.maze_entropy_window_seconds.to_string(),
        ),
        (
            "SHUMA_MAZE_CLIENT_EXPANSION_ENABLED".to_string(),
            bool_env(cfg.maze_client_expansion_enabled).to_string(),
        ),
        (
            "SHUMA_MAZE_CHECKPOINT_EVERY_NODES".to_string(),
            cfg.maze_checkpoint_every_nodes.to_string(),
        ),
        (
            "SHUMA_MAZE_CHECKPOINT_EVERY_MS".to_string(),
            cfg.maze_checkpoint_every_ms.to_string(),
        ),
        (
            "SHUMA_MAZE_STEP_AHEAD_MAX".to_string(),
            cfg.maze_step_ahead_max.to_string(),
        ),
        (
            "SHUMA_MAZE_NO_JS_FALLBACK_MAX_DEPTH".to_string(),
            cfg.maze_no_js_fallback_max_depth.to_string(),
        ),
        (
            "SHUMA_MAZE_MICRO_POW_ENABLED".to_string(),
            bool_env(cfg.maze_micro_pow_enabled).to_string(),
        ),
        (
            "SHUMA_MAZE_MICRO_POW_DEPTH_START".to_string(),
            cfg.maze_micro_pow_depth_start.to_string(),
        ),
        (
            "SHUMA_MAZE_MICRO_POW_BASE_DIFFICULTY".to_string(),
            cfg.maze_micro_pow_base_difficulty.to_string(),
        ),
        (
            "SHUMA_MAZE_MAX_CONCURRENT_GLOBAL".to_string(),
            cfg.maze_max_concurrent_global.to_string(),
        ),
        (
            "SHUMA_MAZE_MAX_CONCURRENT_PER_IP_BUCKET".to_string(),
            cfg.maze_max_concurrent_per_ip_bucket.to_string(),
        ),
        (
            "SHUMA_MAZE_MAX_RESPONSE_BYTES".to_string(),
            cfg.maze_max_response_bytes.to_string(),
        ),
        (
            "SHUMA_MAZE_MAX_RESPONSE_DURATION_MS".to_string(),
            cfg.maze_max_response_duration_ms.to_string(),
        ),
        (
            "SHUMA_MAZE_SERVER_VISIBLE_LINKS".to_string(),
            cfg.maze_server_visible_links.to_string(),
        ),
        (
            "SHUMA_MAZE_MAX_LINKS".to_string(),
            cfg.maze_max_links.to_string(),
        ),
        (
            "SHUMA_MAZE_MAX_PARAGRAPHS".to_string(),
            cfg.maze_max_paragraphs.to_string(),
        ),
        (
            "SHUMA_MAZE_PATH_ENTROPY_SEGMENT_LEN".to_string(),
            cfg.maze_path_entropy_segment_len.to_string(),
        ),
        (
            "SHUMA_MAZE_COVERT_DECOYS_ENABLED".to_string(),
            bool_env(cfg.maze_covert_decoys_enabled).to_string(),
        ),
        (
            "SHUMA_MAZE_SEED_PROVIDER".to_string(),
            cfg.maze_seed_provider.as_str().to_string(),
        ),
        (
            "SHUMA_MAZE_SEED_REFRESH_INTERVAL_SECONDS".to_string(),
            cfg.maze_seed_refresh_interval_seconds.to_string(),
        ),
        (
            "SHUMA_MAZE_SEED_REFRESH_RATE_LIMIT_PER_HOUR".to_string(),
            cfg.maze_seed_refresh_rate_limit_per_hour.to_string(),
        ),
        (
            "SHUMA_MAZE_SEED_REFRESH_MAX_SOURCES".to_string(),
            cfg.maze_seed_refresh_max_sources.to_string(),
        ),
        (
            "SHUMA_MAZE_SEED_METADATA_ONLY".to_string(),
            bool_env(cfg.maze_seed_metadata_only).to_string(),
        ),
        (
            "SHUMA_ROBOTS_ENABLED".to_string(),
            bool_env(cfg.robots_enabled).to_string(),
        ),
        (
            "SHUMA_ROBOTS_BLOCK_AI_TRAINING".to_string(),
            bool_env(cfg.robots_block_ai_training).to_string(),
        ),
        (
            "SHUMA_ROBOTS_BLOCK_AI_SEARCH".to_string(),
            bool_env(cfg.robots_block_ai_search).to_string(),
        ),
        (
            "SHUMA_ROBOTS_ALLOW_SEARCH_ENGINES".to_string(),
            bool_env(cfg.robots_allow_search_engines).to_string(),
        ),
        (
            "SHUMA_AI_POLICY_BLOCK_TRAINING".to_string(),
            bool_env(cfg.robots_block_ai_training).to_string(),
        ),
        (
            "SHUMA_AI_POLICY_BLOCK_SEARCH".to_string(),
            bool_env(cfg.robots_block_ai_search).to_string(),
        ),
        (
            "SHUMA_AI_POLICY_ALLOW_SEARCH_ENGINES".to_string(),
            bool_env(cfg.robots_allow_search_engines).to_string(),
        ),
        (
            "SHUMA_ROBOTS_CRAWL_DELAY".to_string(),
            cfg.robots_crawl_delay.to_string(),
        ),
        (
            "SHUMA_CDP_DETECTION_ENABLED".to_string(),
            bool_env(cfg.cdp_detection_enabled).to_string(),
        ),
        (
            "SHUMA_CDP_AUTO_BAN".to_string(),
            bool_env(cfg.cdp_auto_ban).to_string(),
        ),
        (
            "SHUMA_CDP_DETECTION_THRESHOLD".to_string(),
            cfg.cdp_detection_threshold.to_string(),
        ),
    ]
}

fn handle_admin_config_export(
    req: &Request,
    store: &impl crate::challenge::KeyValueStore,
    site_id: &str,
) -> Response {
    if *req.method() != spin_sdk::http::Method::Get {
        return Response::new(405, "Method Not Allowed");
    }
    let cfg = match crate::config::Config::load(store, site_id) {
        Ok(cfg) => cfg,
        Err(err) => return Response::new(500, err.user_message()),
    };
    let entries = config_export_env_entries(&cfg);
    let env_map: BTreeMap<String, String> = entries.iter().cloned().collect();
    let env_text = entries
        .iter()
        .map(|(key, value)| format!("{}={}", key, value))
        .collect::<Vec<_>>()
        .join("\n");

    log_event(
        store,
        &EventLogEntry {
            ts: now_ts(),
            event: EventType::AdminAction,
            ip: None,
            reason: Some("config_export".to_string()),
            outcome: Some(format!("{} keys", entries.len())),
            admin: Some(crate::admin::auth::get_admin_id(req)),
        },
    );

    let body = serde_json::to_string(&json!({
        "format": "env",
        "site_id": site_id,
        "generated_at": now_ts(),
        "excluded_secrets": CONFIG_EXPORT_SECRET_KEYS,
        "env": env_map,
        "env_text": env_text
    }))
    .unwrap();
    Response::new(200, body)
}

fn parse_country_list_json(field: &str, value: &serde_json::Value) -> Result<Vec<String>, String> {
    let items = value
        .as_array()
        .ok_or_else(|| format!("{} must be an array of 2-letter country codes", field))?;
    let mut parsed = Vec::with_capacity(items.len());
    for item in items {
        let raw = item
            .as_str()
            .ok_or_else(|| format!("{} must contain only strings", field))?;
        let code = crate::signals::geo::normalize_country_code(raw)
            .ok_or_else(|| format!("{} contains invalid country code '{}'", field, raw))?;
        parsed.push(code);
    }
    Ok(crate::signals::geo::normalize_country_list(&parsed))
}

fn parse_composability_mode_json(
    field: &str,
    value: &serde_json::Value,
) -> Result<crate::config::ComposabilityMode, String> {
    let raw = value
        .as_str()
        .ok_or_else(|| format!("{} must be one of: off, signal, enforce, both", field))?;
    crate::config::parse_composability_mode(raw)
        .ok_or_else(|| format!("{} must be one of: off, signal, enforce, both", field))
}

fn parse_provider_backend_json(
    field: &str,
    value: &serde_json::Value,
) -> Result<crate::config::ProviderBackend, String> {
    let raw = value
        .as_str()
        .ok_or_else(|| format!("{} must be one of: internal, external", field))?;
    crate::config::parse_provider_backend(raw)
        .ok_or_else(|| format!("{} must be one of: internal, external", field))
}

fn parse_edge_integration_mode_json(
    field: &str,
    value: &serde_json::Value,
) -> Result<crate::config::EdgeIntegrationMode, String> {
    let raw = value
        .as_str()
        .ok_or_else(|| format!("{} must be one of: off, advisory, authoritative", field))?;
    crate::config::parse_edge_integration_mode(raw)
        .ok_or_else(|| format!("{} must be one of: off, advisory, authoritative", field))
}

fn parse_maze_rollout_phase_json(
    field: &str,
    value: &serde_json::Value,
) -> Result<crate::config::MazeRolloutPhase, String> {
    let raw = value
        .as_str()
        .ok_or_else(|| format!("{} must be one of: instrument, advisory, enforce", field))?;
    crate::config::parse_maze_rollout_phase(raw)
        .ok_or_else(|| format!("{} must be one of: instrument, advisory, enforce", field))
}

fn parse_maze_seed_provider_json(
    field: &str,
    value: &serde_json::Value,
) -> Result<crate::config::MazeSeedProvider, String> {
    let raw = value
        .as_str()
        .ok_or_else(|| format!("{} must be one of: internal, operator", field))?;
    crate::config::parse_maze_seed_provider(raw)
        .ok_or_else(|| format!("{} must be one of: internal, operator", field))
}

fn handle_admin_config(
    req: &Request,
    store: &impl crate::challenge::KeyValueStore,
    site_id: &str,
) -> Response {
    // GET: Return current config
    // POST: Update config (supports {"test_mode": true/false})
    if *req.method() == spin_sdk::http::Method::Post {
        if !crate::config::admin_config_write_enabled() {
            return Response::new(
                403,
                "Config updates are disabled when SHUMA_ADMIN_CONFIG_WRITE_ENABLED=false",
            );
        }
        let json = match crate::request_validation::parse_json_body(
            req.body(),
            crate::request_validation::MAX_ADMIN_JSON_BYTES,
        ) {
            Ok(v) => v,
            Err(e) => return Response::new(400, e),
        };
        // Load current config
        let mut cfg = match crate::config::Config::load(store, site_id) {
            Ok(cfg) => cfg,
            Err(err) => return Response::new(500, err.user_message()),
        };
        let mut changed = false;

        // Update test_mode if provided
        if let Some(test_mode) = json.get("test_mode").and_then(|v| v.as_bool()) {
            let old_value = cfg.test_mode;
            cfg.test_mode = test_mode;
            if old_value != test_mode {
                changed = true;
                // Log test_mode toggle event
                log_event(
                    store,
                    &EventLogEntry {
                        ts: now_ts(),
                        event: EventType::AdminAction,
                        ip: None,
                        reason: Some("test_mode_toggle".to_string()),
                        outcome: Some(format!("{} -> {}", old_value, test_mode)),
                        admin: Some(crate::admin::auth::get_admin_id(req)),
                    },
                );
            }
        }

        // Update other config fields if provided
        if let Some(ban_duration) = json.get("ban_duration").and_then(|v| v.as_u64()) {
            cfg.ban_duration = ban_duration;
            changed = true;
        }
        if let Some(rate_limit) = json.get("rate_limit").and_then(|v| v.as_u64()) {
            if !(1..=1_000_000).contains(&rate_limit) {
                return Response::new(400, "rate_limit out of range (1-1000000)");
            }
            cfg.rate_limit = rate_limit as u32;
            changed = true;
        }
        if let Some(js_required_enforced) =
            json.get("js_required_enforced").and_then(|v| v.as_bool())
        {
            cfg.js_required_enforced = js_required_enforced;
            changed = true;
        }

        // Update GEO policy lists if provided.
        if let Some(value) = json.get("geo_risk") {
            match parse_country_list_json("geo_risk", value) {
                Ok(list) => {
                    cfg.geo_risk = list;
                    changed = true;
                }
                Err(msg) => return Response::new(400, msg),
            }
        }
        if let Some(value) = json.get("geo_allow") {
            match parse_country_list_json("geo_allow", value) {
                Ok(list) => {
                    cfg.geo_allow = list;
                    changed = true;
                }
                Err(msg) => return Response::new(400, msg),
            }
        }
        if let Some(value) = json.get("geo_challenge") {
            match parse_country_list_json("geo_challenge", value) {
                Ok(list) => {
                    cfg.geo_challenge = list;
                    changed = true;
                }
                Err(msg) => return Response::new(400, msg),
            }
        }
        if let Some(value) = json.get("geo_maze") {
            match parse_country_list_json("geo_maze", value) {
                Ok(list) => {
                    cfg.geo_maze = list;
                    changed = true;
                }
                Err(msg) => return Response::new(400, msg),
            }
        }
        if let Some(value) = json.get("geo_block") {
            match parse_country_list_json("geo_block", value) {
                Ok(list) => {
                    cfg.geo_block = list;
                    changed = true;
                }
                Err(msg) => return Response::new(400, msg),
            }
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
        if let Some(maze_auto_ban_threshold) =
            json.get("maze_auto_ban_threshold").and_then(|v| v.as_u64())
        {
            cfg.maze_auto_ban_threshold = maze_auto_ban_threshold as u32;
            changed = true;
        }
        if let Some(value) = json.get("maze_rollout_phase") {
            cfg.maze_rollout_phase = match parse_maze_rollout_phase_json("maze_rollout_phase", value)
            {
                Ok(phase) => phase,
                Err(msg) => return Response::new(400, msg),
            };
            changed = true;
        }
        if let Some(v) = json.get("maze_token_ttl_seconds").and_then(|v| v.as_u64()) {
            cfg.maze_token_ttl_seconds = v;
            changed = true;
        }
        if let Some(v) = json.get("maze_token_max_depth").and_then(|v| v.as_u64()) {
            cfg.maze_token_max_depth = v as u16;
            changed = true;
        }
        if let Some(v) = json.get("maze_token_branch_budget").and_then(|v| v.as_u64()) {
            cfg.maze_token_branch_budget = v as u8;
            changed = true;
        }
        if let Some(v) = json.get("maze_replay_ttl_seconds").and_then(|v| v.as_u64()) {
            cfg.maze_replay_ttl_seconds = v;
            changed = true;
        }
        if let Some(v) = json.get("maze_entropy_window_seconds").and_then(|v| v.as_u64()) {
            cfg.maze_entropy_window_seconds = v;
            changed = true;
        }
        if let Some(v) = json
            .get("maze_client_expansion_enabled")
            .and_then(|v| v.as_bool())
        {
            cfg.maze_client_expansion_enabled = v;
            changed = true;
        }
        if let Some(v) = json
            .get("maze_checkpoint_every_nodes")
            .and_then(|v| v.as_u64())
        {
            cfg.maze_checkpoint_every_nodes = v;
            changed = true;
        }
        if let Some(v) = json.get("maze_checkpoint_every_ms").and_then(|v| v.as_u64()) {
            cfg.maze_checkpoint_every_ms = v;
            changed = true;
        }
        if let Some(v) = json.get("maze_step_ahead_max").and_then(|v| v.as_u64()) {
            cfg.maze_step_ahead_max = v;
            changed = true;
        }
        if let Some(v) = json
            .get("maze_no_js_fallback_max_depth")
            .and_then(|v| v.as_u64())
        {
            cfg.maze_no_js_fallback_max_depth = v as u16;
            changed = true;
        }
        if let Some(v) = json.get("maze_micro_pow_enabled").and_then(|v| v.as_bool()) {
            cfg.maze_micro_pow_enabled = v;
            changed = true;
        }
        if let Some(v) = json
            .get("maze_micro_pow_depth_start")
            .and_then(|v| v.as_u64())
        {
            cfg.maze_micro_pow_depth_start = v as u16;
            changed = true;
        }
        if let Some(v) = json
            .get("maze_micro_pow_base_difficulty")
            .and_then(|v| v.as_u64())
        {
            cfg.maze_micro_pow_base_difficulty = v as u8;
            changed = true;
        }
        if let Some(v) = json
            .get("maze_max_concurrent_global")
            .and_then(|v| v.as_u64())
        {
            cfg.maze_max_concurrent_global = v as u32;
            changed = true;
        }
        if let Some(v) = json
            .get("maze_max_concurrent_per_ip_bucket")
            .and_then(|v| v.as_u64())
        {
            cfg.maze_max_concurrent_per_ip_bucket = v as u32;
            changed = true;
        }
        if let Some(v) = json.get("maze_max_response_bytes").and_then(|v| v.as_u64()) {
            cfg.maze_max_response_bytes = v as u32;
            changed = true;
        }
        if let Some(v) = json
            .get("maze_max_response_duration_ms")
            .and_then(|v| v.as_u64())
        {
            cfg.maze_max_response_duration_ms = v;
            changed = true;
        }
        if let Some(v) = json
            .get("maze_server_visible_links")
            .and_then(|v| v.as_u64())
        {
            cfg.maze_server_visible_links = v as u32;
            changed = true;
        }
        if let Some(v) = json.get("maze_max_links").and_then(|v| v.as_u64()) {
            cfg.maze_max_links = v as u32;
            changed = true;
        }
        if let Some(v) = json.get("maze_max_paragraphs").and_then(|v| v.as_u64()) {
            cfg.maze_max_paragraphs = v as u32;
            changed = true;
        }
        if let Some(v) = json
            .get("maze_path_entropy_segment_len")
            .and_then(|v| v.as_u64())
        {
            cfg.maze_path_entropy_segment_len = v as u8;
            changed = true;
        }
        if let Some(v) = json
            .get("maze_covert_decoys_enabled")
            .and_then(|v| v.as_bool())
        {
            cfg.maze_covert_decoys_enabled = v;
            changed = true;
        }
        if let Some(value) = json.get("maze_seed_provider") {
            cfg.maze_seed_provider = match parse_maze_seed_provider_json("maze_seed_provider", value)
            {
                Ok(provider) => provider,
                Err(msg) => return Response::new(400, msg),
            };
            changed = true;
        }
        if let Some(v) = json
            .get("maze_seed_refresh_interval_seconds")
            .and_then(|v| v.as_u64())
        {
            cfg.maze_seed_refresh_interval_seconds = v;
            changed = true;
        }
        if let Some(v) = json
            .get("maze_seed_refresh_rate_limit_per_hour")
            .and_then(|v| v.as_u64())
        {
            cfg.maze_seed_refresh_rate_limit_per_hour = v as u32;
            changed = true;
        }
        if let Some(v) = json
            .get("maze_seed_refresh_max_sources")
            .and_then(|v| v.as_u64())
        {
            cfg.maze_seed_refresh_max_sources = v as u32;
            changed = true;
        }
        if let Some(v) = json.get("maze_seed_metadata_only").and_then(|v| v.as_bool()) {
            cfg.maze_seed_metadata_only = v;
            changed = true;
        }

        // Update robots.txt settings if provided
        if let Some(robots_enabled) = json.get("robots_enabled").and_then(|v| v.as_bool()) {
            cfg.robots_enabled = robots_enabled;
            changed = true;
        }
        let ai_policy_block_training = json
            .get("ai_policy_block_training")
            .and_then(|v| v.as_bool())
            .or_else(|| {
                json.get("robots_block_ai_training")
                    .and_then(|v| v.as_bool())
            });
        if let Some(robots_block_ai_training) = ai_policy_block_training {
            cfg.robots_block_ai_training = robots_block_ai_training;
            changed = true;
        }
        let ai_policy_block_search = json
            .get("ai_policy_block_search")
            .and_then(|v| v.as_bool())
            .or_else(|| json.get("robots_block_ai_search").and_then(|v| v.as_bool()));
        if let Some(robots_block_ai_search) = ai_policy_block_search {
            cfg.robots_block_ai_search = robots_block_ai_search;
            changed = true;
        }
        let ai_policy_allow_search_engines = json
            .get("ai_policy_allow_search_engines")
            .and_then(|v| v.as_bool())
            .or_else(|| {
                json.get("robots_allow_search_engines")
                    .and_then(|v| v.as_bool())
            });
        if let Some(robots_allow_search_engines) = ai_policy_allow_search_engines {
            cfg.robots_allow_search_engines = robots_allow_search_engines;
            changed = true;
        }
        if let Some(robots_crawl_delay) = json.get("robots_crawl_delay").and_then(|v| v.as_u64()) {
            cfg.robots_crawl_delay = robots_crawl_delay as u32;
            changed = true;
        }

        // Update CDP detection settings if provided
        if let Some(cdp_detection_enabled) =
            json.get("cdp_detection_enabled").and_then(|v| v.as_bool())
        {
            cfg.cdp_detection_enabled = cdp_detection_enabled;
            changed = true;
        }
        if let Some(cdp_auto_ban) = json.get("cdp_auto_ban").and_then(|v| v.as_bool()) {
            cfg.cdp_auto_ban = cdp_auto_ban;
            changed = true;
        }
        if let Some(cdp_detection_threshold) =
            json.get("cdp_detection_threshold").and_then(|v| v.as_f64())
        {
            cfg.cdp_detection_threshold = cdp_detection_threshold as f32;
            changed = true;
        }

        let old_pow_difficulty = cfg.pow_difficulty;
        let old_pow_ttl = cfg.pow_ttl_seconds;
        let mut pow_changed = false;

        // Update PoW settings if provided (guarded by env flag)
        if json.get("pow_difficulty").is_some() || json.get("pow_ttl_seconds").is_some() {
            if !crate::config::pow_config_mutable() {
                return Response::new(
                    403,
                    "PoW config is immutable (set SHUMA_POW_CONFIG_MUTABLE=1 to allow changes)",
                );
            }
        }
        if let Some(pow_difficulty) = json.get("pow_difficulty").and_then(|v| v.as_u64()) {
            if pow_difficulty < POW_DIFFICULTY_MIN as u64
                || pow_difficulty > POW_DIFFICULTY_MAX as u64
            {
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
            log_event(
                store,
                &EventLogEntry {
                    ts: now_ts(),
                    event: EventType::AdminAction,
                    ip: None,
                    reason: Some("pow_config_update".to_string()),
                    outcome: Some(format!(
                        "difficulty:{}->{} ttl:{}->{}",
                        old_pow_difficulty, cfg.pow_difficulty, old_pow_ttl, cfg.pow_ttl_seconds
                    )),
                    admin: Some(crate::admin::auth::get_admin_id(req)),
                },
            );
        }

        let mut provider_selection_changed = false;
        let old_provider_backends = cfg.provider_backends.clone();
        let old_edge_integration_mode = cfg.edge_integration_mode;

        if let Some(provider_backends) = json.get("provider_backends") {
            let Some(backends_obj) = provider_backends.as_object() else {
                return Response::new(
                    400,
                    "provider_backends must be an object with optional keys: rate_limiter, ban_store, challenge_engine, maze_tarpit, fingerprint_signal",
                );
            };
            for key in backends_obj.keys() {
                if !matches!(
                    key.as_str(),
                    "rate_limiter"
                        | "ban_store"
                        | "challenge_engine"
                        | "maze_tarpit"
                        | "fingerprint_signal"
                ) {
                    return Response::new(
                        400,
                        format!("provider_backends.{} is not supported", key),
                    );
                }
            }

            if let Some(value) = backends_obj.get("rate_limiter") {
                cfg.provider_backends.rate_limiter =
                    match parse_provider_backend_json("provider_backends.rate_limiter", value) {
                        Ok(backend) => backend,
                        Err(msg) => return Response::new(400, msg),
                    };
                changed = true;
                provider_selection_changed = true;
            }
            if let Some(value) = backends_obj.get("ban_store") {
                cfg.provider_backends.ban_store =
                    match parse_provider_backend_json("provider_backends.ban_store", value) {
                        Ok(backend) => backend,
                        Err(msg) => return Response::new(400, msg),
                    };
                changed = true;
                provider_selection_changed = true;
            }
            if let Some(value) = backends_obj.get("challenge_engine") {
                cfg.provider_backends.challenge_engine = match parse_provider_backend_json(
                    "provider_backends.challenge_engine",
                    value,
                ) {
                    Ok(backend) => backend,
                    Err(msg) => return Response::new(400, msg),
                };
                changed = true;
                provider_selection_changed = true;
            }
            if let Some(value) = backends_obj.get("maze_tarpit") {
                cfg.provider_backends.maze_tarpit =
                    match parse_provider_backend_json("provider_backends.maze_tarpit", value) {
                        Ok(backend) => backend,
                        Err(msg) => return Response::new(400, msg),
                    };
                changed = true;
                provider_selection_changed = true;
            }
            if let Some(value) = backends_obj.get("fingerprint_signal") {
                cfg.provider_backends.fingerprint_signal = match parse_provider_backend_json(
                    "provider_backends.fingerprint_signal",
                    value,
                ) {
                    Ok(backend) => backend,
                    Err(msg) => return Response::new(400, msg),
                };
                changed = true;
                provider_selection_changed = true;
            }
        }

        if let Some(value) = json.get("edge_integration_mode") {
            cfg.edge_integration_mode =
                match parse_edge_integration_mode_json("edge_integration_mode", value) {
                    Ok(mode) => mode,
                    Err(msg) => return Response::new(400, msg),
                };
            changed = true;
            provider_selection_changed = true;
        }

        if provider_selection_changed {
            log_event(
                store,
                &EventLogEntry {
                    ts: now_ts(),
                    event: EventType::AdminAction,
                    ip: None,
                    reason: Some("provider_selection_update".to_string()),
                    outcome: Some(format!(
                        "providers(rate_limiter:{}->{} ban_store:{}->{} challenge_engine:{}->{} maze_tarpit:{}->{} fingerprint_signal:{}->{}) edge:{}->{}",
                        old_provider_backends.rate_limiter.as_str(),
                        cfg.provider_backends.rate_limiter.as_str(),
                        old_provider_backends.ban_store.as_str(),
                        cfg.provider_backends.ban_store.as_str(),
                        old_provider_backends.challenge_engine.as_str(),
                        cfg.provider_backends.challenge_engine.as_str(),
                        old_provider_backends.maze_tarpit.as_str(),
                        cfg.provider_backends.maze_tarpit.as_str(),
                        old_provider_backends.fingerprint_signal.as_str(),
                        cfg.provider_backends.fingerprint_signal.as_str(),
                        old_edge_integration_mode.as_str(),
                        cfg.edge_integration_mode.as_str(),
                    )),
                    admin: Some(crate::admin::auth::get_admin_id(req)),
                },
            );
        }

        let botness_mutable = crate::config::botness_config_mutable();
        let mut botness_changed = false;
        let old_challenge_threshold = cfg.challenge_risk_threshold;
        let old_maze_threshold = cfg.botness_maze_threshold;
        let old_weights = cfg.botness_weights.clone();
        let old_modes = cfg.defence_modes.clone();
        let botness_update_requested = json.get("challenge_risk_threshold").is_some()
            || json.get("botness_maze_threshold").is_some()
            || json.get("botness_weights").is_some()
            || json.get("defence_modes").is_some();
        if botness_update_requested && !botness_mutable {
            return Response::new(
                    403,
                    "Botness config is immutable (set SHUMA_BOTNESS_CONFIG_MUTABLE=true or SHUMA_CHALLENGE_CONFIG_MUTABLE=true to allow changes)"
                );
        }
        if let Some(challenge_threshold) = json
            .get("challenge_risk_threshold")
            .and_then(|v| v.as_u64())
        {
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
            if let Some(maze_behavior) = weights.get("maze_behavior").and_then(|v| v.as_u64()) {
                if maze_behavior > 10 {
                    return Response::new(400, "botness_weights.maze_behavior out of range (0-10)");
                }
                cfg.botness_weights.maze_behavior = maze_behavior as u8;
                changed = true;
                botness_changed = true;
            }
        }
        if let Some(defence_modes) = json.get("defence_modes") {
            let Some(modes_obj) = defence_modes.as_object() else {
                return Response::new(
                    400,
                    "defence_modes must be an object with optional keys: rate, geo, js",
                );
            };
            for key in modes_obj.keys() {
                if !matches!(key.as_str(), "rate" | "geo" | "js") {
                    return Response::new(400, format!("defence_modes.{} is not supported", key));
                }
            }

            if let Some(value) = modes_obj.get("rate") {
                cfg.defence_modes.rate =
                    match parse_composability_mode_json("defence_modes.rate", value) {
                        Ok(mode) => mode,
                        Err(msg) => return Response::new(400, msg),
                    };
                changed = true;
                botness_changed = true;
            }
            if let Some(value) = modes_obj.get("geo") {
                cfg.defence_modes.geo =
                    match parse_composability_mode_json("defence_modes.geo", value) {
                        Ok(mode) => mode,
                        Err(msg) => return Response::new(400, msg),
                    };
                changed = true;
                botness_changed = true;
            }
            if let Some(value) = modes_obj.get("js") {
                cfg.defence_modes.js =
                    match parse_composability_mode_json("defence_modes.js", value) {
                        Ok(mode) => mode,
                        Err(msg) => return Response::new(400, msg),
                    };
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
                        "challenge:{}->{} maze:{}->{} weights(js:{}->{} geo:{}->{} rate_med:{}->{} rate_high:{}->{} maze_behavior:{}->{}) modes(rate:{:?}->{:?} geo:{:?}->{:?} js:{:?}->{:?})",
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
                        cfg.botness_weights.rate_high,
                        old_weights.maze_behavior,
                        cfg.botness_weights.maze_behavior,
                        old_modes.rate,
                        cfg.defence_modes.rate,
                        old_modes.geo,
                        cfg.defence_modes.geo,
                        old_modes.js,
                        cfg.defence_modes.js
                    )),
                    admin: Some(crate::admin::auth::get_admin_id(req)),
                });
        }

        // Save config to KV store
        if changed {
            let key = format!("config:{}", site_id);
            if let Ok(val) = serde_json::to_vec(&cfg) {
                if store.set(&key, &val).is_ok() {
                    crate::config::invalidate_runtime_cache(site_id);
                }
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
                "geo_allow": cfg.geo_allow,
                "geo_challenge": cfg.geo_challenge,
                "geo_maze": cfg.geo_maze,
                "geo_block": cfg.geo_block,
                "maze_enabled": cfg.maze_enabled,
                "maze_auto_ban": cfg.maze_auto_ban,
                "maze_auto_ban_threshold": cfg.maze_auto_ban_threshold,
                "maze_rollout_phase": cfg.maze_rollout_phase,
                "maze_token_ttl_seconds": cfg.maze_token_ttl_seconds,
                "maze_token_max_depth": cfg.maze_token_max_depth,
                "maze_token_branch_budget": cfg.maze_token_branch_budget,
                "maze_replay_ttl_seconds": cfg.maze_replay_ttl_seconds,
                "maze_entropy_window_seconds": cfg.maze_entropy_window_seconds,
                "maze_client_expansion_enabled": cfg.maze_client_expansion_enabled,
                "maze_checkpoint_every_nodes": cfg.maze_checkpoint_every_nodes,
                "maze_checkpoint_every_ms": cfg.maze_checkpoint_every_ms,
                "maze_step_ahead_max": cfg.maze_step_ahead_max,
                "maze_no_js_fallback_max_depth": cfg.maze_no_js_fallback_max_depth,
                "maze_micro_pow_enabled": cfg.maze_micro_pow_enabled,
                "maze_micro_pow_depth_start": cfg.maze_micro_pow_depth_start,
                "maze_micro_pow_base_difficulty": cfg.maze_micro_pow_base_difficulty,
                "maze_max_concurrent_global": cfg.maze_max_concurrent_global,
                "maze_max_concurrent_per_ip_bucket": cfg.maze_max_concurrent_per_ip_bucket,
                "maze_max_response_bytes": cfg.maze_max_response_bytes,
                "maze_max_response_duration_ms": cfg.maze_max_response_duration_ms,
                "maze_server_visible_links": cfg.maze_server_visible_links,
                "maze_max_links": cfg.maze_max_links,
                "maze_max_paragraphs": cfg.maze_max_paragraphs,
                "maze_path_entropy_segment_len": cfg.maze_path_entropy_segment_len,
                "maze_covert_decoys_enabled": cfg.maze_covert_decoys_enabled,
                "maze_seed_provider": cfg.maze_seed_provider,
                "maze_seed_refresh_interval_seconds": cfg.maze_seed_refresh_interval_seconds,
                "maze_seed_refresh_rate_limit_per_hour": cfg.maze_seed_refresh_rate_limit_per_hour,
                "maze_seed_refresh_max_sources": cfg.maze_seed_refresh_max_sources,
                "maze_seed_metadata_only": cfg.maze_seed_metadata_only,
                "robots_enabled": cfg.robots_enabled,
                "ai_policy_block_training": cfg.robots_block_ai_training,
                "ai_policy_block_search": cfg.robots_block_ai_search,
                "ai_policy_allow_search_engines": cfg.robots_allow_search_engines,
                "robots_block_ai_training": cfg.robots_block_ai_training,
                "robots_block_ai_search": cfg.robots_block_ai_search,
                "robots_allow_search_engines": cfg.robots_allow_search_engines,
                "robots_crawl_delay": cfg.robots_crawl_delay,
                "cdp_detection_enabled": cfg.cdp_detection_enabled,
                "cdp_auto_ban": cfg.cdp_auto_ban,
                "cdp_detection_threshold": cfg.cdp_detection_threshold,
                "js_required_enforced": cfg.js_required_enforced,
                "pow_enabled": cfg.pow_enabled,
                "pow_config_mutable": crate::config::pow_config_mutable(),
                "pow_difficulty": cfg.pow_difficulty,
                "pow_ttl_seconds": cfg.pow_ttl_seconds,
                "admin_config_write_enabled": crate::config::admin_config_write_enabled(),
                "https_enforced": crate::config::https_enforced(),
                "forwarded_header_trust_configured": crate::config::forwarded_header_trust_configured(),
                "challenge_risk_threshold": cfg.challenge_risk_threshold,
                "challenge_config_mutable": crate::config::challenge_config_mutable(),
                "challenge_risk_threshold_default": challenge_default,
                "botness_maze_threshold": cfg.botness_maze_threshold,
                "botness_maze_threshold_default": maze_default,
                "botness_weights": {
                    "js_required": cfg.botness_weights.js_required,
                    "geo_risk": cfg.botness_weights.geo_risk,
                    "rate_medium": cfg.botness_weights.rate_medium,
                    "rate_high": cfg.botness_weights.rate_high,
                    "maze_behavior": cfg.botness_weights.maze_behavior
                },
                "defence_modes": {
                    "rate": cfg.defence_modes.rate,
                    "geo": cfg.defence_modes.geo,
                    "js": cfg.defence_modes.js
                },
                "provider_backends": {
                    "rate_limiter": cfg.provider_backends.rate_limiter,
                    "ban_store": cfg.provider_backends.ban_store,
                    "challenge_engine": cfg.provider_backends.challenge_engine,
                    "maze_tarpit": cfg.provider_backends.maze_tarpit,
                    "fingerprint_signal": cfg.provider_backends.fingerprint_signal
                },
                "edge_integration_mode": cfg.edge_integration_mode,
                "defence_modes_effective": cfg.defence_modes_effective(),
                "defence_mode_warnings": cfg.defence_mode_warnings(),
                "enterprise_multi_instance": crate::config::enterprise_multi_instance_enabled(),
                "enterprise_unsynced_state_exception_confirmed": crate::config::enterprise_unsynced_state_exception_confirmed(),
                "enterprise_state_guardrail_warnings": cfg.enterprise_state_guardrail_warnings(),
                "enterprise_state_guardrail_error": cfg.enterprise_state_guardrail_error(),
                "botness_config_mutable": botness_mutable,
                "botness_signal_definitions": botness_signal_definitions(&cfg)
            }
        }))
        .unwrap();
        return Response::new(200, body);
    }
    // GET: Return current config
    let cfg = match crate::config::Config::load(store, site_id) {
        Ok(cfg) => cfg,
        Err(err) => return Response::new(500, err.user_message()),
    };
    log_event(
        store,
        &EventLogEntry {
            ts: now_ts(),
            event: EventType::AdminAction,
            ip: None,
            reason: Some("config_view".to_string()),
            outcome: Some(format!("test_mode={}", cfg.test_mode)),
            admin: Some(crate::admin::auth::get_admin_id(req)),
        },
    );
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
        "geo_allow": cfg.geo_allow,
        "geo_challenge": cfg.geo_challenge,
        "geo_maze": cfg.geo_maze,
        "geo_block": cfg.geo_block,
        "whitelist": cfg.whitelist,
        "path_whitelist": cfg.path_whitelist,
        "maze_enabled": cfg.maze_enabled,
        "maze_auto_ban": cfg.maze_auto_ban,
        "maze_auto_ban_threshold": cfg.maze_auto_ban_threshold,
        "maze_rollout_phase": cfg.maze_rollout_phase,
        "maze_token_ttl_seconds": cfg.maze_token_ttl_seconds,
        "maze_token_max_depth": cfg.maze_token_max_depth,
        "maze_token_branch_budget": cfg.maze_token_branch_budget,
        "maze_replay_ttl_seconds": cfg.maze_replay_ttl_seconds,
        "maze_entropy_window_seconds": cfg.maze_entropy_window_seconds,
        "maze_client_expansion_enabled": cfg.maze_client_expansion_enabled,
        "maze_checkpoint_every_nodes": cfg.maze_checkpoint_every_nodes,
        "maze_checkpoint_every_ms": cfg.maze_checkpoint_every_ms,
        "maze_step_ahead_max": cfg.maze_step_ahead_max,
        "maze_no_js_fallback_max_depth": cfg.maze_no_js_fallback_max_depth,
        "maze_micro_pow_enabled": cfg.maze_micro_pow_enabled,
        "maze_micro_pow_depth_start": cfg.maze_micro_pow_depth_start,
        "maze_micro_pow_base_difficulty": cfg.maze_micro_pow_base_difficulty,
        "maze_max_concurrent_global": cfg.maze_max_concurrent_global,
        "maze_max_concurrent_per_ip_bucket": cfg.maze_max_concurrent_per_ip_bucket,
        "maze_max_response_bytes": cfg.maze_max_response_bytes,
        "maze_max_response_duration_ms": cfg.maze_max_response_duration_ms,
        "maze_server_visible_links": cfg.maze_server_visible_links,
        "maze_max_links": cfg.maze_max_links,
        "maze_max_paragraphs": cfg.maze_max_paragraphs,
        "maze_path_entropy_segment_len": cfg.maze_path_entropy_segment_len,
        "maze_covert_decoys_enabled": cfg.maze_covert_decoys_enabled,
        "maze_seed_provider": cfg.maze_seed_provider,
        "maze_seed_refresh_interval_seconds": cfg.maze_seed_refresh_interval_seconds,
        "maze_seed_refresh_rate_limit_per_hour": cfg.maze_seed_refresh_rate_limit_per_hour,
        "maze_seed_refresh_max_sources": cfg.maze_seed_refresh_max_sources,
        "maze_seed_metadata_only": cfg.maze_seed_metadata_only,
        "robots_enabled": cfg.robots_enabled,
        "ai_policy_block_training": cfg.robots_block_ai_training,
        "ai_policy_block_search": cfg.robots_block_ai_search,
        "ai_policy_allow_search_engines": cfg.robots_allow_search_engines,
        "robots_block_ai_training": cfg.robots_block_ai_training,
        "robots_block_ai_search": cfg.robots_block_ai_search,
        "robots_allow_search_engines": cfg.robots_allow_search_engines,
        "robots_crawl_delay": cfg.robots_crawl_delay,
        "cdp_detection_enabled": cfg.cdp_detection_enabled,
        "cdp_auto_ban": cfg.cdp_auto_ban,
        "cdp_detection_threshold": cfg.cdp_detection_threshold,
        "js_required_enforced": cfg.js_required_enforced,
        "pow_enabled": cfg.pow_enabled,
        "pow_config_mutable": crate::config::pow_config_mutable(),
        "pow_difficulty": cfg.pow_difficulty,
        "pow_ttl_seconds": cfg.pow_ttl_seconds,
        "admin_config_write_enabled": crate::config::admin_config_write_enabled(),
        "https_enforced": crate::config::https_enforced(),
        "forwarded_header_trust_configured": crate::config::forwarded_header_trust_configured(),
        "challenge_risk_threshold": cfg.challenge_risk_threshold,
        "challenge_config_mutable": crate::config::challenge_config_mutable(),
        "challenge_risk_threshold_default": challenge_default,
        "botness_maze_threshold": cfg.botness_maze_threshold,
        "botness_maze_threshold_default": maze_default,
        "botness_weights": {
            "js_required": cfg.botness_weights.js_required,
            "geo_risk": cfg.botness_weights.geo_risk,
            "rate_medium": cfg.botness_weights.rate_medium,
            "rate_high": cfg.botness_weights.rate_high,
            "maze_behavior": cfg.botness_weights.maze_behavior
        },
        "defence_modes": {
            "rate": cfg.defence_modes.rate,
            "geo": cfg.defence_modes.geo,
            "js": cfg.defence_modes.js
        },
        "provider_backends": {
            "rate_limiter": cfg.provider_backends.rate_limiter,
            "ban_store": cfg.provider_backends.ban_store,
            "challenge_engine": cfg.provider_backends.challenge_engine,
            "maze_tarpit": cfg.provider_backends.maze_tarpit,
            "fingerprint_signal": cfg.provider_backends.fingerprint_signal
        },
        "edge_integration_mode": cfg.edge_integration_mode,
        "defence_modes_effective": cfg.defence_modes_effective(),
        "defence_mode_warnings": cfg.defence_mode_warnings(),
        "enterprise_multi_instance": crate::config::enterprise_multi_instance_enabled(),
        "enterprise_unsynced_state_exception_confirmed": crate::config::enterprise_unsynced_state_exception_confirmed(),
        "enterprise_state_guardrail_warnings": cfg.enterprise_state_guardrail_warnings(),
        "enterprise_state_guardrail_error": cfg.enterprise_state_guardrail_error(),
        "botness_config_mutable": crate::config::botness_config_mutable(),
        "botness_signal_definitions": botness_signal_definitions(&cfg)
    }))
    .unwrap();
    Response::new(200, body)
}

fn parse_operator_seed_sources_json(
    value: &serde_json::Value,
) -> Result<Vec<crate::maze::seeds::OperatorSeedSource>, String> {
    let entries = value
        .as_array()
        .ok_or_else(|| "sources must be an array".to_string())?;
    let mut sources = Vec::with_capacity(entries.len());
    for entry in entries {
        let obj = entry
            .as_object()
            .ok_or_else(|| "each source must be an object".to_string())?;
        let id = obj
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let url = obj
            .get("url")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
        let title = obj
            .get("title")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
        let description = obj
            .get("description")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
        let keywords = obj
            .get("keywords")
            .and_then(|v| v.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str().map(|s| s.to_string()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let allow_seed_use = obj
            .get("allow_seed_use")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let robots_allowed = obj
            .get("robots_allowed")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let body_excerpt = obj
            .get("body_excerpt")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
        sources.push(crate::maze::seeds::OperatorSeedSource {
            id,
            url,
            title,
            description,
            keywords,
            allow_seed_use,
            robots_allowed,
            body_excerpt,
        });
    }
    Ok(sources)
}

fn handle_admin_maze_seed_sources<S>(req: &Request, store: &S, site_id: &str) -> Response
where
    S: crate::challenge::KeyValueStore + crate::maze::state::MazeStateStore,
{
    let cfg = match crate::config::Config::load(store, site_id) {
        Ok(cfg) => cfg,
        Err(err) => return Response::new(500, err.user_message()),
    };

    match *req.method() {
        Method::Get => {
            let sources = crate::maze::seeds::list_operator_sources(store);
            let cache = crate::maze::seeds::cached_seed_snapshot(store);
            let body = serde_json::to_string(&json!({
                "seed_provider": cfg.maze_seed_provider,
                "seed_refresh_interval_seconds": cfg.maze_seed_refresh_interval_seconds,
                "seed_refresh_rate_limit_per_hour": cfg.maze_seed_refresh_rate_limit_per_hour,
                "seed_refresh_max_sources": cfg.maze_seed_refresh_max_sources,
                "seed_metadata_only": cfg.maze_seed_metadata_only,
                "sources": sources,
                "cache": cache
            }))
            .unwrap();
            Response::new(200, body)
        }
        Method::Post => {
            let payload = match crate::request_validation::parse_json_body(
                req.body(),
                crate::request_validation::MAX_ADMIN_JSON_BYTES,
            ) {
                Ok(payload) => payload,
                Err(err) => return Response::new(400, err),
            };
            let Some(value) = payload.get("sources") else {
                return Response::new(400, "sources field is required");
            };
            let sources = match parse_operator_seed_sources_json(value) {
                Ok(sources) => sources,
                Err(err) => return Response::new(400, err),
            };
            if let Err(err) = crate::maze::seeds::save_operator_sources(store, &cfg, sources.clone()) {
                return Response::new(400, err);
            }
            log_event(
                store,
                &EventLogEntry {
                    ts: now_ts(),
                    event: EventType::AdminAction,
                    ip: None,
                    reason: Some("maze_seed_sources_update".to_string()),
                    outcome: Some(format!("sources={}", sources.len())),
                    admin: Some(crate::admin::auth::get_admin_id(req)),
                },
            );
            let body = serde_json::to_string(&json!({
                "updated": true,
                "source_count": sources.len()
            }))
            .unwrap();
            Response::new(200, body)
        }
        _ => Response::new(405, "Method Not Allowed"),
    }
}

fn handle_admin_maze_seed_refresh<S>(req: &Request, store: &S, site_id: &str) -> Response
where
    S: crate::challenge::KeyValueStore + crate::maze::state::MazeStateStore,
{
    if *req.method() != Method::Post {
        return Response::new(405, "Method Not Allowed");
    }
    let cfg = match crate::config::Config::load(store, site_id) {
        Ok(cfg) => cfg,
        Err(err) => return Response::new(500, err.user_message()),
    };
    if cfg.maze_seed_provider != crate::config::MazeSeedProvider::Operator {
        return Response::new(
            409,
            "maze_seed_provider must be 'operator' for manual seed refresh",
        );
    }

    let now = now_ts();
    let refreshed = match crate::maze::seeds::manual_refresh_operator_corpus(store, &cfg, now) {
        Ok(refreshed) => refreshed,
        Err(err) => {
            if err.contains("rate limit exceeded") {
                return Response::new(429, err);
            }
            return Response::new(400, err);
        }
    };
    log_event(
        store,
        &EventLogEntry {
            ts: now,
            event: EventType::AdminAction,
            ip: None,
            reason: Some("maze_seed_refresh".to_string()),
            outcome: Some(format!(
                "provider={} version={} terms={} sources={}",
                refreshed.provider,
                refreshed.version,
                refreshed.terms.len(),
                refreshed.source_count
            )),
            admin: Some(crate::admin::auth::get_admin_id(req)),
        },
    );
    let body = serde_json::to_string(&json!({
        "refreshed": true,
        "provider": refreshed.provider,
        "version": refreshed.version,
        "metadata_only": refreshed.metadata_only,
        "source_count": refreshed.source_count,
        "term_count": refreshed.terms.len()
    }))
    .unwrap();
    Response::new(200, body)
}

/// Handles all /admin API endpoints.
/// Supports:
///   - POST /admin/login: Exchange API key for short-lived admin session cookie
///   - GET /admin/session: Return current admin auth session state
///   - POST /admin/logout: Clear admin session cookie
///   - GET /admin/ban: List all bans for the site
///   - POST /admin/ban: Manually ban an IP (expects JSON body: {"ip": "1.2.3.4", "duration": 3600}; reason is fixed to "manual_ban")
///   - POST /admin/unban?ip=...: Remove a ban for an IP
///   - GET /admin/analytics: Return ban count and test_mode status
///   - GET /admin/events: Query event log
///   - GET /admin/cdp/events: Query CDP-only events
///   - GET /admin/config: Get current config including test_mode status
///   - POST /admin/config: Update config (e.g., toggle test_mode)
///   - GET /admin/config/export: Export non-secret runtime config for immutable deploy handoff
///   - GET /admin: API help
pub fn handle_admin(req: &Request) -> Response {
    // Optional admin IP allowlist
    if !crate::admin::auth::is_admin_ip_allowed(req) {
        return Response::new(403, "Forbidden");
    }
    if !crate::admin::auth::is_admin_api_key_configured() {
        return Response::new(503, "Admin API disabled: key not configured");
    }

    let path = req.path();
    if !sanitize_path(path) {
        return Response::new(400, "Bad Request: Invalid admin endpoint");
    }

    if path == "/admin/login" || path == "/admin/session" || path == "/admin/logout" {
        let store = match Store::open_default() {
            Ok(s) => s,
            Err(_) => return Response::new(500, "Key-value store error"),
        };
        let provider_registry = crate::config::load_runtime_cached(&store, "default")
            .ok()
            .map(|cfg| crate::providers::registry::ProviderRegistry::from_config(&cfg));
        return match path {
            "/admin/login" => handle_admin_login_with_failure_handler(req, &store, || {
                register_admin_auth_failure_with_selected_rate_limiter(
                    &store,
                    req,
                    crate::admin::auth::AdminAuthFailureScope::Login,
                    provider_registry.as_ref(),
                )
            }),
            "/admin/session" => handle_admin_session(req, &store),
            "/admin/logout" => handle_admin_logout_with_failure_handler(req, &store, || {
                register_admin_auth_failure_with_selected_rate_limiter(
                    &store,
                    req,
                    crate::admin::auth::AdminAuthFailureScope::Endpoint,
                    provider_registry.as_ref(),
                )
            }),
            _ => Response::new(400, "Bad Request: Invalid admin endpoint"),
        };
    }

    let has_bearer = crate::admin::auth::is_bearer_authorized(req);
    let has_session_cookie = crate::admin::auth::has_admin_session_cookie(req);
    if !has_bearer && !has_session_cookie {
        return Response::new(401, "Unauthorized: Invalid or missing API key");
    }

    let store = match Store::open_default() {
        Ok(s) => s,
        Err(_) => return Response::new(500, "Key-value store error"),
    };
    let provider_registry = crate::config::load_runtime_cached(&store, "default")
        .ok()
        .map(|cfg| crate::providers::registry::ProviderRegistry::from_config(&cfg));

    // Require either a valid bearer token or a valid admin session cookie.
    let auth = crate::admin::auth::authenticate_admin(req, &store);
    if !auth.is_authorized() {
        if register_admin_auth_failure_with_selected_rate_limiter(
            &store,
            req,
            crate::admin::auth::AdminAuthFailureScope::Endpoint,
            provider_registry.as_ref(),
        ) {
            return too_many_admin_auth_attempts_response();
        }
        return Response::new(401, "Unauthorized: Invalid or missing API key");
    }
    if auth.requires_csrf(req) {
        let expected = auth.csrf_token.as_deref().unwrap_or("");
        if !crate::admin::auth::validate_session_csrf(req, expected) {
            return Response::new(403, "Forbidden");
        }
    }
    if request_requires_admin_write(path, req.method()) && !auth.is_write_authorized() {
        log_admin_write_denied(&store, req, path, &auth);
        return Response::new(403, "Forbidden: admin write access required");
    }

    let site_id = "default";

    match path {
        "/admin/events" => {
            // Query event log for recent events, top IPs, and event statistics
            // Query params: ?hours=N (default 24, max 720)
            let hours = query_u64_param(req.query(), "hours", 24).clamp(1, 720);
            let now = now_ts();
            let mut events = load_recent_events(&store, now, hours);
            let mut ip_counts = std::collections::HashMap::new();
            let mut event_counts = std::collections::HashMap::new();

            for e in &events {
                if let Some(ip) = &e.ip {
                    *ip_counts.entry(ip.clone()).or_insert(0u32) += 1;
                }
                *event_counts.entry(format!("{:?}", e.event)).or_insert(0u32) += 1;
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
            }))
            .unwrap();
            // Log admin analytics view
            log_event(
                &store,
                &EventLogEntry {
                    ts: now_ts(),
                    event: EventType::AdminAction,
                    ip: None,
                    reason: Some("events_view".to_string()),
                    outcome: Some(format!("{} events", events.len())),
                    admin: Some(crate::admin::auth::get_admin_id(req)),
                },
            );
            Response::new(200, body)
        }
        "/admin/cdp/events" => {
            // Query params: ?hours=N&limit=M
            // hours default 24 (max 720), limit default 500 (max 5000)
            let hours = query_u64_param(req.query(), "hours", 24).clamp(1, 720);
            let limit = query_u64_param(req.query(), "limit", 500).clamp(1, 5000) as usize;
            let now = now_ts();
            let mut cdp_events: Vec<EventLogEntry> = load_recent_events(&store, now, hours)
                .into_iter()
                .filter(|entry| {
                    entry
                        .reason
                        .as_deref()
                        .map(is_cdp_event_reason)
                        .unwrap_or(false)
                })
                .collect();

            cdp_events.sort_by(|a, b| b.ts.cmp(&a.ts));

            let total_matches = cdp_events.len();
            let detections = cdp_events
                .iter()
                .filter(|entry| {
                    entry
                        .reason
                        .as_deref()
                        .map(|reason| reason.to_lowercase().starts_with("cdp_detected:"))
                        .unwrap_or(false)
                })
                .count();
            let auto_bans = cdp_events
                .iter()
                .filter(|entry| {
                    entry
                        .reason
                        .as_deref()
                        .map(|reason| reason.eq_ignore_ascii_case("cdp_automation"))
                        .unwrap_or(false)
                })
                .count();

            cdp_events.truncate(limit);

            // Log admin view for CDP-focused telemetry
            log_event(
                &store,
                &EventLogEntry {
                    ts: now_ts(),
                    event: EventType::AdminAction,
                    ip: None,
                    reason: Some("cdp_events_view".to_string()),
                    outcome: Some(format!(
                        "{} cdp events (hours={}, limit={})",
                        total_matches, hours, limit
                    )),
                    admin: Some(crate::admin::auth::get_admin_id(req)),
                },
            );

            let body = serde_json::to_string(&json!({
                "events": cdp_events,
                "hours": hours,
                "limit": limit,
                "total_matches": total_matches,
                "counts": {
                    "detections": detections,
                    "auto_bans": auto_bans
                }
            }))
            .unwrap();
            Response::new(200, body)
        }
        "/admin/ban" => {
            let cfg = match crate::config::load_runtime_cached(&store, site_id) {
                Ok(cfg) => cfg,
                Err(err) => return Response::new(500, err.user_message()),
            };
            let provider_registry = crate::providers::registry::ProviderRegistry::from_config(&cfg);

            // POST: Manually ban an IP
            if *req.method() == spin_sdk::http::Method::Post {
                let json = match crate::request_validation::parse_json_body(
                    req.body(),
                    crate::request_validation::MAX_ADMIN_JSON_BYTES,
                ) {
                    Ok(v) => v,
                    Err(e) => return Response::new(400, e),
                };

                let ip_raw = match json.get("ip").and_then(|v| v.as_str()) {
                    Some(v) => v,
                    None => return Response::new(400, "Missing 'ip' field in request body"),
                };
                let ip = match crate::request_validation::parse_ip_addr(ip_raw) {
                    Some(v) => v,
                    None => return Response::new(400, "Invalid IP address"),
                };
                // Manual bans are always tagged with a fixed reason to prevent client-side tampering.
                let reason = "manual_ban".to_string();
                let duration = json
                    .get("duration")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(21600)
                    .clamp(ADMIN_BAN_DURATION_MIN, ADMIN_BAN_DURATION_MAX);

                provider_registry
                    .ban_store_provider()
                    .ban_ip_with_fingerprint(
                        &store,
                        site_id,
                        ip.as_str(),
                        reason.as_str(),
                        duration,
                        Some(crate::enforcement::ban::BanFingerprint {
                            score: None,
                            signals: vec!["manual_admin".to_string()],
                            summary: Some("manual_admin_ban".to_string()),
                        }),
                    );
                // Log ban event
                log_event(
                    &store,
                    &EventLogEntry {
                        ts: now_ts(),
                        event: EventType::Ban,
                        ip: Some(ip.clone()),
                        reason: Some(reason.clone()),
                        outcome: Some("banned".to_string()),
                        admin: Some(crate::admin::auth::get_admin_id(req)),
                    },
                );
                return Response::new(200, json!({"status": "banned", "ip": ip}).to_string());
            }
            // GET: List all bans for this site (keys starting with ban:site_id:)
            let mut bans = vec![];
            for (ip, ban) in provider_registry
                .ban_store_provider()
                .list_active_bans(&store, site_id)
            {
                bans.push(json!({
                    "ip": ip,
                    "reason": ban.reason,
                    "expires": ban.expires,
                    "banned_at": ban.banned_at,
                    "fingerprint": ban.fingerprint
                }));
            }
            // Log admin action
            log_event(
                &store,
                &EventLogEntry {
                    ts: now_ts(),
                    event: EventType::AdminAction,
                    ip: None,
                    reason: Some("ban_list".to_string()),
                    outcome: Some(format!("{} bans listed", bans.len())),
                    admin: Some(crate::admin::auth::get_admin_id(req)),
                },
            );
            let body = serde_json::to_string(&json!({"bans": bans})).unwrap();
            Response::new(200, body)
        }
        "/admin/unban" => {
            if *req.method() != spin_sdk::http::Method::Post {
                return Response::new(405, "Method Not Allowed");
            }
            // Unban IP (expects ?ip=...)
            let ip_raw = match crate::request_validation::query_param(req.query(), "ip") {
                Some(v) => v,
                None => return Response::new(400, "Missing ip param"),
            };
            let ip = match crate::request_validation::parse_ip_addr(&ip_raw) {
                Some(v) => v,
                None => return Response::new(400, "Invalid IP address"),
            };
            if ip.is_empty() {
                return Response::new(400, "Missing ip param");
            }
            let cfg = match crate::config::load_runtime_cached(&store, site_id) {
                Ok(cfg) => cfg,
                Err(err) => return Response::new(500, err.user_message()),
            };
            let provider_registry = crate::providers::registry::ProviderRegistry::from_config(&cfg);
            provider_registry
                .ban_store_provider()
                .unban_ip(&store, site_id, ip.as_str());
            // Log unban event
            log_event(
                &store,
                &EventLogEntry {
                    ts: now_ts(),
                    event: EventType::Unban,
                    ip: Some(ip.to_string()),
                    reason: Some("admin_unban".to_string()),
                    outcome: Some("unbanned".to_string()),
                    admin: Some(crate::admin::auth::get_admin_id(req)),
                },
            );
            Response::new(200, "Unbanned")
        }
        "/admin/analytics" => {
            // Return analytics: ban count and test_mode status
            let cfg = match crate::config::Config::load(&store, site_id) {
                Ok(cfg) => cfg,
                Err(err) => return Response::new(500, err.user_message()),
            };
            let ban_count =
                crate::enforcement::ban::list_active_bans_with_scan(&store, site_id).len();
            let fail_mode = if crate::config::kv_store_fail_open() {
                "open"
            } else {
                "closed"
            };
            // Log admin analytics view
            log_event(
                &store,
                &EventLogEntry {
                    ts: now_ts(),
                    event: EventType::AdminAction,
                    ip: None,
                    reason: Some("analytics_view".to_string()),
                    outcome: Some(format!("ban_count={}", ban_count)),
                    admin: Some(crate::admin::auth::get_admin_id(req)),
                },
            );
            let body = serde_json::to_string(&json!({
                "ban_count": ban_count,
                "test_mode": cfg.test_mode,
                "fail_mode": fail_mode
            }))
            .unwrap();
            Response::new(200, body)
        }
        "/admin/config" => {
            return handle_admin_config(req, &store, site_id);
        }
        "/admin/config/export" => {
            return handle_admin_config_export(req, &store, site_id);
        }
        "/admin/maze/seeds" => {
            return handle_admin_maze_seed_sources(req, &store, site_id);
        }
        "/admin/maze/seeds/refresh" => {
            return handle_admin_maze_seed_refresh(req, &store, site_id);
        }
        "/admin" => {
            // API help endpoint
            log_event(
                &store,
                &EventLogEntry {
                    ts: now_ts(),
                    event: EventType::AdminAction,
                    ip: None,
                    reason: Some("help".to_string()),
                    outcome: None,
                    admin: Some(crate::admin::auth::get_admin_id(req)),
                },
            );
            Response::new(200, "WASM Bot Defence Admin API. Endpoints: /admin/ban, /admin/unban?ip=IP, /admin/analytics, /admin/events, /admin/config, /admin/config/export, /admin/maze (GET for maze stats), /admin/maze/seeds (GET/POST seed source adapters), /admin/maze/seeds/refresh (POST manual seed refresh), /admin/robots (GET for robots.txt config & preview), /admin/cdp (GET for CDP detection config & stats), /admin/cdp/events (GET for CDP detection and auto-ban events).")
        }
        "/admin/maze" => {
            // Return maze statistics
            // - Total unique IPs that have visited maze pages
            // - Per-IP hit counts (top crawlers)
            // - Total maze hits
            let mut maze_ips: Vec<(String, u32)> = Vec::new();
            let mut total_hits: u32 = 0;

            if let Ok(keys) = store.get_keys() {
                for k in keys {
                    if k.starts_with("maze_hits:") {
                        let ip = k
                            .strip_prefix("maze_hits:")
                            .unwrap_or("unknown")
                            .to_string();
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
            let deepest = maze_ips
                .first()
                .map(|(ip, hits)| json!({"ip": ip, "hits": hits}));

            // Top 10 crawlers
            let top_crawlers: Vec<_> = maze_ips
                .iter()
                .take(10)
                .map(|(ip, hits)| json!({"ip": ip, "hits": hits}))
                .collect();

            // Count auto-bans from maze (check bans with reason "maze_crawler")
            let maze_bans = crate::enforcement::ban::list_active_bans_with_scan(&store, site_id)
                .into_iter()
                .filter(|(_, ban)| ban.reason == "maze_crawler")
                .count();

            // Log admin maze view
            log_event(
                &store,
                &EventLogEntry {
                    ts: now_ts(),
                    event: EventType::AdminAction,
                    ip: None,
                    reason: Some("maze_stats_view".to_string()),
                    outcome: Some(format!("{} crawlers, {} hits", maze_ips.len(), total_hits)),
                    admin: Some(crate::admin::auth::get_admin_id(req)),
                },
            );

            let body = serde_json::to_string(&json!({
                "total_hits": total_hits,
                "unique_crawlers": maze_ips.len(),
                "maze_auto_bans": maze_bans,
                "deepest_crawler": deepest,
                "top_crawlers": top_crawlers
            }))
            .unwrap();
            Response::new(200, body)
        }
        "/admin/robots" => {
            // Return robots.txt configuration and preview
            let cfg = match crate::config::Config::load(&store, site_id) {
                Ok(cfg) => cfg,
                Err(err) => return Response::new(500, err.user_message()),
            };

            // Generate preview of robots.txt content
            let preview = crate::crawler_policy::robots::generate_robots_txt(&cfg);
            let content_signal = crate::crawler_policy::robots::get_content_signal_header(&cfg);

            // Log admin action
            log_event(
                &store,
                &EventLogEntry {
                    ts: now_ts(),
                    event: EventType::AdminAction,
                    ip: None,
                    reason: Some("robots_config_view".to_string()),
                    outcome: None,
                    admin: Some(crate::admin::auth::get_admin_id(req)),
                },
            );

            let body = serde_json::to_string(&json!({
                "config": {
                    "enabled": cfg.robots_enabled,
                    "ai_policy_block_training": cfg.robots_block_ai_training,
                    "ai_policy_block_search": cfg.robots_block_ai_search,
                    "ai_policy_allow_search_engines": cfg.robots_allow_search_engines,
                    "block_ai_training": cfg.robots_block_ai_training,
                    "block_ai_search": cfg.robots_block_ai_search,
                    "allow_search_engines": cfg.robots_allow_search_engines,
                    "crawl_delay": cfg.robots_crawl_delay
                },
                "content_signal_header": content_signal,
                "ai_training_bots": crate::crawler_policy::robots::AI_TRAINING_BOTS,
                "ai_search_bots": crate::crawler_policy::robots::AI_SEARCH_BOTS,
                "search_engine_bots": crate::crawler_policy::robots::SEARCH_ENGINE_BOTS,
                "preview": preview
            }))
            .unwrap();
            Response::new(200, body)
        }
        "/admin/cdp" => {
            // Return CDP detection configuration and stats
            let cfg = match crate::config::Config::load(&store, site_id) {
                Ok(cfg) => cfg,
                Err(err) => return Response::new(500, err.user_message()),
            };

            // Get CDP detection stats from KV store
            let cdp_detections: u64 = store
                .get("cdp:detections")
                .ok()
                .flatten()
                .and_then(|v| String::from_utf8(v).ok())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);

            let cdp_auto_bans: u64 = store
                .get("cdp:auto_bans")
                .ok()
                .flatten()
                .and_then(|v| String::from_utf8(v).ok())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);

            // Log admin action
            log_event(
                &store,
                &EventLogEntry {
                    ts: now_ts(),
                    event: EventType::AdminAction,
                    ip: None,
                    reason: Some("cdp_config_view".to_string()),
                    outcome: None,
                    admin: Some(crate::admin::auth::get_admin_id(req)),
                },
            );

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
            }))
            .unwrap();
            Response::new(200, body)
        }
        _ => Response::new(404, "Not found"),
    }
}
