use super::*;
use crate::challenge::KeyValueStore;
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Mutex,
};

#[derive(Default)]
struct CountingStore {
    map: Mutex<HashMap<String, Vec<u8>>>,
    get_count: AtomicUsize,
}

impl CountingStore {
    fn get_count(&self) -> usize {
        self.get_count.load(Ordering::SeqCst)
    }
}

impl KeyValueStore for CountingStore {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ()> {
        self.get_count.fetch_add(1, Ordering::SeqCst);
        let map = self.map.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        Ok(map.get(key).cloned())
    }

    fn set(&self, key: &str, value: &[u8]) -> Result<(), ()> {
        let mut map = self.map.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        map.insert(key.to_string(), value.to_vec());
        Ok(())
    }

    fn delete(&self, key: &str) -> Result<(), ()> {
        let mut map = self.map.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        map.remove(key);
        Ok(())
    }
}

fn clear_env(keys: &[&str]) {
    for key in keys {
        std::env::remove_var(key);
    }
}

fn store_config_with_rate_limit(store: &CountingStore, rate_limit: u32) {
    let mut cfg = defaults().clone();
    cfg.rate_limit = rate_limit;
    store
        .set("config:default", &serde_json::to_vec(&cfg).unwrap())
        .unwrap();
}

#[test]
fn parse_challenge_threshold_defaults_to_3() {
    assert_eq!(parse_challenge_threshold(None), 3);
}

#[test]
fn parse_challenge_threshold_clamps_range() {
    assert_eq!(parse_challenge_threshold(Some("0")), 1);
    assert_eq!(parse_challenge_threshold(Some("99")), 10);
    assert_eq!(parse_challenge_threshold(Some("5")), 5);
    assert_eq!(parse_challenge_threshold(Some("junk")), 3);
}

#[test]
fn parse_maze_threshold_clamps_range() {
    assert_eq!(parse_maze_threshold(Some("0")), 1);
    assert_eq!(parse_maze_threshold(Some("99")), 10);
    assert_eq!(parse_maze_threshold(Some("6")), 6);
    assert_eq!(parse_maze_threshold(Some("junk")), 6);
}

#[test]
fn parse_botness_weight_clamps_range() {
    assert_eq!(parse_botness_weight(Some("0"), 3), 0);
    assert_eq!(parse_botness_weight(Some("11"), 3), 10);
    assert_eq!(parse_botness_weight(Some("4"), 3), 4);
    assert_eq!(parse_botness_weight(Some("junk"), 3), 3);
}

#[test]
fn parse_composability_mode_accepts_expected_values() {
    assert_eq!(
        parse_composability_mode("off"),
        Some(ComposabilityMode::Off)
    );
    assert_eq!(
        parse_composability_mode("signal"),
        Some(ComposabilityMode::Signal)
    );
    assert_eq!(
        parse_composability_mode("enforce"),
        Some(ComposabilityMode::Enforce)
    );
    assert_eq!(
        parse_composability_mode("both"),
        Some(ComposabilityMode::Both)
    );
    assert_eq!(
        parse_composability_mode("  BoTh "),
        Some(ComposabilityMode::Both)
    );
    assert_eq!(parse_composability_mode("invalid"), None);
    assert_eq!(ComposabilityMode::Off.as_str(), "off");
    assert_eq!(ComposabilityMode::Signal.as_str(), "signal");
    assert_eq!(ComposabilityMode::Enforce.as_str(), "enforce");
    assert_eq!(ComposabilityMode::Both.as_str(), "both");
}

#[test]
fn parse_provider_backend_accepts_expected_values() {
    assert_eq!(
        parse_provider_backend("internal"),
        Some(ProviderBackend::Internal)
    );
    assert_eq!(
        parse_provider_backend("external"),
        Some(ProviderBackend::External)
    );
    assert_eq!(
        parse_provider_backend("  ExTeRnAl "),
        Some(ProviderBackend::External)
    );
    assert_eq!(parse_provider_backend("invalid"), None);
    assert_eq!(ProviderBackend::Internal.as_str(), "internal");
    assert_eq!(ProviderBackend::External.as_str(), "external");
}

#[test]
fn defaults_enable_both_signal_and_action_paths() {
    let cfg = defaults().clone();
    assert!(cfg.js_required_enforced);
    assert_eq!(cfg.defence_modes.js, ComposabilityMode::Both);
    assert_eq!(cfg.defence_modes.geo, ComposabilityMode::Both);
    assert_eq!(cfg.defence_modes.rate, ComposabilityMode::Both);
    assert!(cfg.rate_signal_enabled());
    assert!(cfg.rate_action_enabled());
    assert!(cfg.geo_signal_enabled());
    assert!(cfg.geo_action_enabled());
    assert!(cfg.js_signal_enabled());
    assert!(cfg.js_action_enabled());

    let effective = cfg.defence_modes_effective();
    assert!(effective.rate.signal_enabled);
    assert!(effective.rate.action_enabled);
    assert!(effective.geo.signal_enabled);
    assert!(effective.geo.action_enabled);
    assert!(effective.js.signal_enabled);
    assert!(effective.js.action_enabled);
    assert!(cfg.defence_mode_warnings().is_empty());
    assert_eq!(cfg.provider_backends.rate_limiter, ProviderBackend::Internal);
    assert_eq!(cfg.provider_backends.ban_store, ProviderBackend::Internal);
    assert_eq!(
        cfg.provider_backends.challenge_engine,
        ProviderBackend::Internal
    );
    assert_eq!(cfg.provider_backends.maze_tarpit, ProviderBackend::Internal);
    assert_eq!(
        cfg.provider_backends.fingerprint_signal,
        ProviderBackend::Internal
    );
}

#[test]
fn js_effective_mode_is_disabled_when_js_required_enforced_is_false() {
    let mut cfg = defaults().clone();
    cfg.js_required_enforced = false;
    cfg.defence_modes.js = ComposabilityMode::Both;

    assert!(!cfg.js_signal_enabled());
    assert!(!cfg.js_action_enabled());

    let effective = cfg.defence_modes_effective();
    assert_eq!(effective.js.configured, ComposabilityMode::Both);
    assert!(!effective.js.signal_enabled);
    assert!(!effective.js.action_enabled);
    assert!(effective.js.note.is_some());

    let warnings = cfg.defence_mode_warnings();
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].contains("js_required_enforced=false"));
}

#[test]
fn challenge_config_mutable_from_env_parses_values() {
    assert!(challenge_config_mutable_from_env(Some("1")));
    assert!(challenge_config_mutable_from_env(Some("true")));
    assert!(challenge_config_mutable_from_env(Some("TRUE")));
    assert!(!challenge_config_mutable_from_env(Some("0")));
    assert!(!challenge_config_mutable_from_env(Some("false")));
    assert!(!challenge_config_mutable_from_env(None));
}

#[test]
fn parse_admin_config_write_defaults_to_disabled() {
    assert!(!parse_admin_config_write_enabled(None));
    assert!(!parse_admin_config_write_enabled(Some("junk")));
    assert!(parse_admin_config_write_enabled(Some("true")));
    assert!(parse_admin_config_write_enabled(Some("1")));
    assert!(!parse_admin_config_write_enabled(Some("false")));
}

#[test]
fn https_enforced_reads_required_env_bool() {
    let _lock = crate::test_support::lock_env();
    std::env::set_var("SHUMA_ENFORCE_HTTPS", "false");
    assert!(!https_enforced());

    std::env::set_var("SHUMA_ENFORCE_HTTPS", "true");
    assert!(https_enforced());

    std::env::remove_var("SHUMA_ENFORCE_HTTPS");
}

#[test]
fn forwarded_header_trust_configured_requires_non_empty_secret() {
    let _lock = crate::test_support::lock_env();
    std::env::remove_var("SHUMA_FORWARDED_IP_SECRET");
    assert!(!forwarded_header_trust_configured());

    std::env::set_var("SHUMA_FORWARDED_IP_SECRET", "");
    assert!(!forwarded_header_trust_configured());

    std::env::set_var("SHUMA_FORWARDED_IP_SECRET", "test-secret");
    assert!(forwarded_header_trust_configured());

    std::env::remove_var("SHUMA_FORWARDED_IP_SECRET");
}

#[test]
fn load_config_missing_returns_error() {
    let _lock = crate::test_support::lock_env();
    let store = crate::test_support::InMemoryStore::default();
    let result = Config::load(&store, "default");
    assert!(matches!(result, Err(ConfigLoadError::MissingConfig)));
}

#[test]
fn load_config_reads_kv_only_without_tunable_env_overrides() {
    let _lock = crate::test_support::lock_env();
    let keys = ["SHUMA_RATE_LIMIT", "SHUMA_HONEYPOTS"];
    clear_env(&keys);
    std::env::set_var("SHUMA_RATE_LIMIT", "222");
    std::env::set_var("SHUMA_HONEYPOTS", "[\"/trap-a\",\"/trap-b\"]");

    let store = crate::test_support::InMemoryStore::default();
    let mut kv_cfg = defaults().clone();
    kv_cfg.rate_limit = 111;
    kv_cfg.honeypots = vec!["/kv-trap".to_string()];
    store
        .set("config:default", &serde_json::to_vec(&kv_cfg).unwrap())
        .unwrap();

    let cfg = Config::load(&store, "default").unwrap();
    assert_eq!(cfg.rate_limit, 111);
    assert_eq!(cfg.honeypots, vec!["/kv-trap".to_string()]);

    clear_env(&keys);
}

#[test]
fn runtime_config_cache_hits_within_ttl() {
    let _lock = crate::test_support::lock_env();
    clear_runtime_cache_for_tests();
    let store = CountingStore::default();
    store_config_with_rate_limit(&store, 101);

    let first = load_runtime_cached_for_tests(&store, "default", 100, 2).unwrap();
    let second = load_runtime_cached_for_tests(&store, "default", 101, 2).unwrap();

    assert_eq!(first.rate_limit, 101);
    assert_eq!(second.rate_limit, 101);
    assert_eq!(store.get_count(), 1);
    clear_runtime_cache_for_tests();
}

#[test]
fn runtime_config_cache_refreshes_after_ttl() {
    let _lock = crate::test_support::lock_env();
    clear_runtime_cache_for_tests();
    let store = CountingStore::default();
    store_config_with_rate_limit(&store, 111);

    let _ = load_runtime_cached_for_tests(&store, "default", 100, 2).unwrap();
    let _ = load_runtime_cached_for_tests(&store, "default", 103, 2).unwrap();

    assert_eq!(store.get_count(), 2);
    clear_runtime_cache_for_tests();
}

#[test]
fn runtime_config_cache_invalidation_forces_reload() {
    let _lock = crate::test_support::lock_env();
    clear_runtime_cache_for_tests();
    let store = CountingStore::default();
    store_config_with_rate_limit(&store, 120);

    let first = load_runtime_cached_for_tests(&store, "default", 100, 2).unwrap();
    assert_eq!(first.rate_limit, 120);
    assert_eq!(store.get_count(), 1);

    store_config_with_rate_limit(&store, 220);
    invalidate_runtime_cache("default");

    let refreshed = load_runtime_cached_for_tests(&store, "default", 101, 2).unwrap();

    assert_eq!(refreshed.rate_limit, 220);
    assert_eq!(store.get_count(), 2);
    clear_runtime_cache_for_tests();
}
