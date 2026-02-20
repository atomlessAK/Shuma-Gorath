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
        let map = self
            .map
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        Ok(map.get(key).cloned())
    }

    fn set(&self, key: &str, value: &[u8]) -> Result<(), ()> {
        let mut map = self
            .map
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        map.insert(key.to_string(), value.to_vec());
        Ok(())
    }

    fn delete(&self, key: &str) -> Result<(), ()> {
        let mut map = self
            .map
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
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
fn parse_ip_range_policy_mode_accepts_expected_values() {
    assert_eq!(
        parse_ip_range_policy_mode("off"),
        Some(IpRangePolicyMode::Off)
    );
    assert_eq!(
        parse_ip_range_policy_mode("advisory"),
        Some(IpRangePolicyMode::Advisory)
    );
    assert_eq!(
        parse_ip_range_policy_mode("enforce"),
        Some(IpRangePolicyMode::Enforce)
    );
    assert_eq!(
        parse_ip_range_policy_mode(" EnFoRcE "),
        Some(IpRangePolicyMode::Enforce)
    );
    assert_eq!(parse_ip_range_policy_mode("invalid"), None);
}

#[test]
fn parse_ip_range_policy_action_accepts_expected_values() {
    assert_eq!(
        parse_ip_range_policy_action("forbidden_403"),
        Some(IpRangePolicyAction::Forbidden403)
    );
    assert_eq!(
        parse_ip_range_policy_action("custom_message"),
        Some(IpRangePolicyAction::CustomMessage)
    );
    assert_eq!(
        parse_ip_range_policy_action("drop_connection"),
        Some(IpRangePolicyAction::DropConnection)
    );
    assert_eq!(
        parse_ip_range_policy_action("redirect_308"),
        Some(IpRangePolicyAction::Redirect308)
    );
    assert_eq!(
        parse_ip_range_policy_action("rate_limit"),
        Some(IpRangePolicyAction::RateLimit)
    );
    assert_eq!(
        parse_ip_range_policy_action("honeypot"),
        Some(IpRangePolicyAction::Honeypot)
    );
    assert_eq!(parse_ip_range_policy_action("maze"), Some(IpRangePolicyAction::Maze));
    assert_eq!(
        parse_ip_range_policy_action("tarpit"),
        Some(IpRangePolicyAction::Tarpit)
    );
    assert_eq!(parse_ip_range_policy_action("invalid"), None);
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
fn parse_edge_integration_mode_accepts_expected_values() {
    assert_eq!(
        parse_edge_integration_mode("off"),
        Some(EdgeIntegrationMode::Off)
    );
    assert_eq!(
        parse_edge_integration_mode("advisory"),
        Some(EdgeIntegrationMode::Advisory)
    );
    assert_eq!(
        parse_edge_integration_mode("authoritative"),
        Some(EdgeIntegrationMode::Authoritative)
    );
    assert_eq!(
        parse_edge_integration_mode("  AuThOrItAtIvE "),
        Some(EdgeIntegrationMode::Authoritative)
    );
    assert_eq!(parse_edge_integration_mode("invalid"), None);
    assert_eq!(EdgeIntegrationMode::Off.as_str(), "off");
    assert_eq!(EdgeIntegrationMode::Advisory.as_str(), "advisory");
    assert_eq!(EdgeIntegrationMode::Authoritative.as_str(), "authoritative");
}

#[test]
fn parse_rate_limiter_outage_mode_accepts_expected_values() {
    assert_eq!(
        parse_rate_limiter_outage_mode("fallback_internal"),
        Some(RateLimiterOutageMode::FallbackInternal)
    );
    assert_eq!(
        parse_rate_limiter_outage_mode("fail_open"),
        Some(RateLimiterOutageMode::FailOpen)
    );
    assert_eq!(
        parse_rate_limiter_outage_mode("fail_closed"),
        Some(RateLimiterOutageMode::FailClosed)
    );
    assert_eq!(
        parse_rate_limiter_outage_mode("  FAIL_OPEN "),
        Some(RateLimiterOutageMode::FailOpen)
    );
    assert_eq!(parse_rate_limiter_outage_mode("invalid"), None);
    assert_eq!(
        RateLimiterOutageMode::FallbackInternal.as_str(),
        "fallback_internal"
    );
    assert_eq!(RateLimiterOutageMode::FailOpen.as_str(), "fail_open");
    assert_eq!(RateLimiterOutageMode::FailClosed.as_str(), "fail_closed");
}

#[test]
fn parse_redis_url_accepts_expected_values() {
    assert_eq!(
        parse_redis_url("redis://localhost:6379"),
        Some("redis://localhost:6379".to_string())
    );
    assert_eq!(
        parse_redis_url(" rediss://cache.example:6379 "),
        Some("rediss://cache.example:6379".to_string())
    );
    assert_eq!(parse_redis_url("http://example.com"), None);
    assert_eq!(parse_redis_url(""), None);
}

#[test]
fn defaults_enable_both_signal_and_action_paths() {
    let cfg = defaults().clone();
    assert_eq!(cfg.edge_integration_mode, EdgeIntegrationMode::Off);
    assert!(cfg.js_required_enforced);
    assert!(cfg.honeypot_enabled);
    assert!(cfg.challenge_puzzle_enabled);
    assert_eq!(cfg.defence_modes.js, ComposabilityMode::Both);
    assert_eq!(cfg.defence_modes.geo, ComposabilityMode::Both);
    assert_eq!(cfg.defence_modes.rate, ComposabilityMode::Both);
    assert_eq!(cfg.ip_range_policy_mode, IpRangePolicyMode::Off);
    assert!(cfg.ip_range_emergency_allowlist.is_empty());
    assert!(cfg.ip_range_custom_rules.is_empty());
    assert!(cfg.ip_range_managed_policies.is_empty());
    assert_eq!(cfg.ip_range_managed_max_staleness_hours, 168);
    assert!(!cfg.ip_range_allow_stale_managed_enforce);
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
    assert_eq!(
        cfg.provider_backends.rate_limiter,
        ProviderBackend::Internal
    );
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
fn enterprise_state_guardrail_errors_without_exception_for_unsynced_multi_instance() {
    let _lock = crate::test_support::lock_env();
    clear_env(&[
        "SHUMA_ENTERPRISE_MULTI_INSTANCE",
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "SHUMA_RATE_LIMITER_REDIS_URL",
        "SHUMA_BAN_STORE_REDIS_URL",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH",
    ]);
    std::env::set_var("SHUMA_ENTERPRISE_MULTI_INSTANCE", "true");

    let cfg = defaults().clone();
    let error = cfg.enterprise_state_guardrail_error();
    assert!(error.is_some());
    assert!(error
        .unwrap()
        .contains("SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED=true"));
    assert!(cfg.enterprise_state_guardrail_warnings().is_empty());

    clear_env(&[
        "SHUMA_ENTERPRISE_MULTI_INSTANCE",
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "SHUMA_RATE_LIMITER_REDIS_URL",
        "SHUMA_BAN_STORE_REDIS_URL",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH",
    ]);
}

#[test]
fn enterprise_state_guardrail_warns_for_exceptioned_advisory_unsynced_posture() {
    let _lock = crate::test_support::lock_env();
    clear_env(&[
        "SHUMA_ENTERPRISE_MULTI_INSTANCE",
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "SHUMA_RATE_LIMITER_REDIS_URL",
        "SHUMA_BAN_STORE_REDIS_URL",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH",
    ]);
    std::env::set_var("SHUMA_ENTERPRISE_MULTI_INSTANCE", "true");
    std::env::set_var(
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "true",
    );

    let mut cfg = defaults().clone();
    cfg.edge_integration_mode = EdgeIntegrationMode::Advisory;
    assert_eq!(cfg.enterprise_state_guardrail_error(), None);
    let warnings = cfg.enterprise_state_guardrail_warnings();
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].contains("explicit advisory/off exception"));

    clear_env(&[
        "SHUMA_ENTERPRISE_MULTI_INSTANCE",
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "SHUMA_RATE_LIMITER_REDIS_URL",
        "SHUMA_BAN_STORE_REDIS_URL",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH",
    ]);
}

#[test]
fn enterprise_state_guardrail_errors_for_authoritative_unsynced_posture_even_with_exception() {
    let _lock = crate::test_support::lock_env();
    clear_env(&[
        "SHUMA_ENTERPRISE_MULTI_INSTANCE",
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "SHUMA_RATE_LIMITER_REDIS_URL",
        "SHUMA_BAN_STORE_REDIS_URL",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH",
    ]);
    std::env::set_var("SHUMA_ENTERPRISE_MULTI_INSTANCE", "true");
    std::env::set_var(
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "true",
    );

    let mut cfg = defaults().clone();
    cfg.edge_integration_mode = EdgeIntegrationMode::Authoritative;
    let error = cfg.enterprise_state_guardrail_error();
    assert!(error.is_some());
    assert!(error.unwrap().contains("authoritative mode"));
    assert!(cfg.enterprise_state_guardrail_warnings().is_empty());

    clear_env(&[
        "SHUMA_ENTERPRISE_MULTI_INSTANCE",
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "SHUMA_RATE_LIMITER_REDIS_URL",
        "SHUMA_BAN_STORE_REDIS_URL",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH",
    ]);
}

#[test]
fn enterprise_state_guardrail_is_clear_for_synced_multi_instance_posture() {
    let _lock = crate::test_support::lock_env();
    clear_env(&[
        "SHUMA_ENTERPRISE_MULTI_INSTANCE",
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "SHUMA_RATE_LIMITER_REDIS_URL",
        "SHUMA_BAN_STORE_REDIS_URL",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH",
    ]);
    std::env::set_var("SHUMA_ENTERPRISE_MULTI_INSTANCE", "true");
    std::env::set_var("SHUMA_RATE_LIMITER_REDIS_URL", "redis://redis:6379");
    std::env::set_var("SHUMA_BAN_STORE_REDIS_URL", "redis://redis:6379");

    let mut cfg = defaults().clone();
    cfg.provider_backends.rate_limiter = ProviderBackend::External;
    cfg.provider_backends.ban_store = ProviderBackend::External;
    cfg.edge_integration_mode = EdgeIntegrationMode::Authoritative;
    assert_eq!(cfg.enterprise_state_guardrail_error(), None);
    assert!(cfg.enterprise_state_guardrail_warnings().is_empty());

    clear_env(&[
        "SHUMA_ENTERPRISE_MULTI_INSTANCE",
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "SHUMA_RATE_LIMITER_REDIS_URL",
        "SHUMA_BAN_STORE_REDIS_URL",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH",
    ]);
}

#[test]
fn enterprise_state_guardrail_requires_redis_url_for_external_rate_limiter() {
    let _lock = crate::test_support::lock_env();
    clear_env(&[
        "SHUMA_ENTERPRISE_MULTI_INSTANCE",
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "SHUMA_RATE_LIMITER_REDIS_URL",
        "SHUMA_BAN_STORE_REDIS_URL",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH",
    ]);
    std::env::set_var("SHUMA_ENTERPRISE_MULTI_INSTANCE", "true");
    std::env::set_var(
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "true",
    );

    let mut cfg = defaults().clone();
    cfg.provider_backends.rate_limiter = ProviderBackend::External;
    cfg.edge_integration_mode = EdgeIntegrationMode::Advisory;

    let error = cfg.enterprise_state_guardrail_error();
    assert!(error.is_some());
    assert!(error.unwrap().contains("SHUMA_RATE_LIMITER_REDIS_URL"));

    clear_env(&[
        "SHUMA_ENTERPRISE_MULTI_INSTANCE",
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "SHUMA_RATE_LIMITER_REDIS_URL",
        "SHUMA_BAN_STORE_REDIS_URL",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH",
    ]);
}

#[test]
fn enterprise_state_guardrail_requires_redis_url_for_external_ban_store() {
    let _lock = crate::test_support::lock_env();
    clear_env(&[
        "SHUMA_ENTERPRISE_MULTI_INSTANCE",
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "SHUMA_RATE_LIMITER_REDIS_URL",
        "SHUMA_BAN_STORE_REDIS_URL",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH",
    ]);
    std::env::set_var("SHUMA_ENTERPRISE_MULTI_INSTANCE", "true");
    std::env::set_var(
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "true",
    );

    let mut cfg = defaults().clone();
    cfg.provider_backends.ban_store = ProviderBackend::External;
    cfg.edge_integration_mode = EdgeIntegrationMode::Advisory;

    let error = cfg.enterprise_state_guardrail_error();
    assert!(error.is_some());
    assert!(error.unwrap().contains("SHUMA_BAN_STORE_REDIS_URL"));

    clear_env(&[
        "SHUMA_ENTERPRISE_MULTI_INSTANCE",
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "SHUMA_RATE_LIMITER_REDIS_URL",
        "SHUMA_BAN_STORE_REDIS_URL",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH",
    ]);
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
fn parse_admin_config_write_defaults_to_disabled() {
    assert!(!parse_admin_config_write_enabled(None));
    assert!(!parse_admin_config_write_enabled(Some("junk")));
    assert!(parse_admin_config_write_enabled(Some("true")));
    assert!(parse_admin_config_write_enabled(Some("1")));
    assert!(!parse_admin_config_write_enabled(Some("false")));
}

#[test]
fn validate_env_rejects_invalid_optional_enterprise_bool() {
    let _lock = crate::test_support::lock_env();
    clear_env(&[
        "SHUMA_VALIDATE_ENV_IN_TESTS",
        "SHUMA_API_KEY",
        "SHUMA_JS_SECRET",
        "SHUMA_FORWARDED_IP_SECRET",
        "SHUMA_EVENT_LOG_RETENTION_HOURS",
        "SHUMA_ADMIN_CONFIG_WRITE_ENABLED",
        "SHUMA_KV_STORE_FAIL_OPEN",
        "SHUMA_ENFORCE_HTTPS",
        "SHUMA_DEBUG_HEADERS",
        "SHUMA_ENTERPRISE_MULTI_INSTANCE",
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "SHUMA_RATE_LIMITER_REDIS_URL",
        "SHUMA_BAN_STORE_REDIS_URL",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH",
    ]);

    std::env::set_var("SHUMA_VALIDATE_ENV_IN_TESTS", "true");
    std::env::set_var("SHUMA_API_KEY", "test-admin-key");
    std::env::set_var("SHUMA_JS_SECRET", "test-js-secret");
    std::env::set_var("SHUMA_FORWARDED_IP_SECRET", "test-forwarded-secret");
    std::env::set_var("SHUMA_EVENT_LOG_RETENTION_HOURS", "168");
    std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "false");
    std::env::set_var("SHUMA_KV_STORE_FAIL_OPEN", "true");
    std::env::set_var("SHUMA_ENFORCE_HTTPS", "false");
    std::env::set_var("SHUMA_DEBUG_HEADERS", "false");
    std::env::set_var("SHUMA_ENTERPRISE_MULTI_INSTANCE", "definitely-not-bool");

    let result = validate_env_only_once();
    assert!(result.is_err());
    assert!(result
        .err()
        .unwrap()
        .contains("SHUMA_ENTERPRISE_MULTI_INSTANCE"));

    clear_env(&[
        "SHUMA_VALIDATE_ENV_IN_TESTS",
        "SHUMA_API_KEY",
        "SHUMA_JS_SECRET",
        "SHUMA_FORWARDED_IP_SECRET",
        "SHUMA_EVENT_LOG_RETENTION_HOURS",
        "SHUMA_ADMIN_CONFIG_WRITE_ENABLED",
        "SHUMA_KV_STORE_FAIL_OPEN",
        "SHUMA_ENFORCE_HTTPS",
        "SHUMA_DEBUG_HEADERS",
        "SHUMA_ENTERPRISE_MULTI_INSTANCE",
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "SHUMA_RATE_LIMITER_REDIS_URL",
        "SHUMA_BAN_STORE_REDIS_URL",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH",
    ]);
}

#[test]
fn validate_env_rejects_invalid_optional_redis_url() {
    let _lock = crate::test_support::lock_env();
    clear_env(&[
        "SHUMA_VALIDATE_ENV_IN_TESTS",
        "SHUMA_API_KEY",
        "SHUMA_JS_SECRET",
        "SHUMA_FORWARDED_IP_SECRET",
        "SHUMA_EVENT_LOG_RETENTION_HOURS",
        "SHUMA_ADMIN_CONFIG_WRITE_ENABLED",
        "SHUMA_KV_STORE_FAIL_OPEN",
        "SHUMA_ENFORCE_HTTPS",
        "SHUMA_DEBUG_HEADERS",
        "SHUMA_ENTERPRISE_MULTI_INSTANCE",
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "SHUMA_RATE_LIMITER_REDIS_URL",
        "SHUMA_BAN_STORE_REDIS_URL",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH",
    ]);

    std::env::set_var("SHUMA_VALIDATE_ENV_IN_TESTS", "true");
    std::env::set_var("SHUMA_API_KEY", "test-admin-key");
    std::env::set_var("SHUMA_JS_SECRET", "test-js-secret");
    std::env::set_var("SHUMA_FORWARDED_IP_SECRET", "test-forwarded-secret");
    std::env::set_var("SHUMA_EVENT_LOG_RETENTION_HOURS", "168");
    std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "false");
    std::env::set_var("SHUMA_KV_STORE_FAIL_OPEN", "true");
    std::env::set_var("SHUMA_ENFORCE_HTTPS", "false");
    std::env::set_var("SHUMA_DEBUG_HEADERS", "false");
    std::env::set_var("SHUMA_RATE_LIMITER_REDIS_URL", "https://not-redis.example");

    let result = validate_env_only_once();
    assert!(result.is_err());
    assert!(result
        .err()
        .unwrap()
        .contains("SHUMA_RATE_LIMITER_REDIS_URL"));

    clear_env(&[
        "SHUMA_VALIDATE_ENV_IN_TESTS",
        "SHUMA_API_KEY",
        "SHUMA_JS_SECRET",
        "SHUMA_FORWARDED_IP_SECRET",
        "SHUMA_EVENT_LOG_RETENTION_HOURS",
        "SHUMA_ADMIN_CONFIG_WRITE_ENABLED",
        "SHUMA_KV_STORE_FAIL_OPEN",
        "SHUMA_ENFORCE_HTTPS",
        "SHUMA_DEBUG_HEADERS",
        "SHUMA_ENTERPRISE_MULTI_INSTANCE",
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "SHUMA_RATE_LIMITER_REDIS_URL",
        "SHUMA_BAN_STORE_REDIS_URL",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH",
    ]);
}

#[test]
fn validate_env_rejects_invalid_optional_ban_store_redis_url() {
    let _lock = crate::test_support::lock_env();
    clear_env(&[
        "SHUMA_VALIDATE_ENV_IN_TESTS",
        "SHUMA_API_KEY",
        "SHUMA_JS_SECRET",
        "SHUMA_FORWARDED_IP_SECRET",
        "SHUMA_EVENT_LOG_RETENTION_HOURS",
        "SHUMA_ADMIN_CONFIG_WRITE_ENABLED",
        "SHUMA_KV_STORE_FAIL_OPEN",
        "SHUMA_ENFORCE_HTTPS",
        "SHUMA_DEBUG_HEADERS",
        "SHUMA_ENTERPRISE_MULTI_INSTANCE",
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "SHUMA_RATE_LIMITER_REDIS_URL",
        "SHUMA_BAN_STORE_REDIS_URL",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH",
    ]);

    std::env::set_var("SHUMA_VALIDATE_ENV_IN_TESTS", "true");
    std::env::set_var("SHUMA_API_KEY", "test-admin-key");
    std::env::set_var("SHUMA_JS_SECRET", "test-js-secret");
    std::env::set_var("SHUMA_FORWARDED_IP_SECRET", "test-forwarded-secret");
    std::env::set_var("SHUMA_EVENT_LOG_RETENTION_HOURS", "168");
    std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "false");
    std::env::set_var("SHUMA_KV_STORE_FAIL_OPEN", "true");
    std::env::set_var("SHUMA_ENFORCE_HTTPS", "false");
    std::env::set_var("SHUMA_DEBUG_HEADERS", "false");
    std::env::set_var("SHUMA_BAN_STORE_REDIS_URL", "https://not-redis.example");

    let result = validate_env_only_once();
    assert!(result.is_err());
    assert!(result.err().unwrap().contains("SHUMA_BAN_STORE_REDIS_URL"));

    clear_env(&[
        "SHUMA_VALIDATE_ENV_IN_TESTS",
        "SHUMA_API_KEY",
        "SHUMA_JS_SECRET",
        "SHUMA_FORWARDED_IP_SECRET",
        "SHUMA_EVENT_LOG_RETENTION_HOURS",
        "SHUMA_ADMIN_CONFIG_WRITE_ENABLED",
        "SHUMA_KV_STORE_FAIL_OPEN",
        "SHUMA_ENFORCE_HTTPS",
        "SHUMA_DEBUG_HEADERS",
        "SHUMA_ENTERPRISE_MULTI_INSTANCE",
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "SHUMA_RATE_LIMITER_REDIS_URL",
        "SHUMA_BAN_STORE_REDIS_URL",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH",
    ]);
}

#[test]
fn validate_env_rejects_invalid_optional_rate_limiter_outage_mode() {
    let _lock = crate::test_support::lock_env();
    clear_env(&[
        "SHUMA_VALIDATE_ENV_IN_TESTS",
        "SHUMA_API_KEY",
        "SHUMA_JS_SECRET",
        "SHUMA_FORWARDED_IP_SECRET",
        "SHUMA_EVENT_LOG_RETENTION_HOURS",
        "SHUMA_ADMIN_CONFIG_WRITE_ENABLED",
        "SHUMA_KV_STORE_FAIL_OPEN",
        "SHUMA_ENFORCE_HTTPS",
        "SHUMA_DEBUG_HEADERS",
        "SHUMA_ENTERPRISE_MULTI_INSTANCE",
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "SHUMA_RATE_LIMITER_REDIS_URL",
        "SHUMA_BAN_STORE_REDIS_URL",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH",
    ]);

    std::env::set_var("SHUMA_VALIDATE_ENV_IN_TESTS", "true");
    std::env::set_var("SHUMA_API_KEY", "test-admin-key");
    std::env::set_var("SHUMA_JS_SECRET", "test-js-secret");
    std::env::set_var("SHUMA_FORWARDED_IP_SECRET", "test-forwarded-secret");
    std::env::set_var("SHUMA_EVENT_LOG_RETENTION_HOURS", "168");
    std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "false");
    std::env::set_var("SHUMA_KV_STORE_FAIL_OPEN", "true");
    std::env::set_var("SHUMA_ENFORCE_HTTPS", "false");
    std::env::set_var("SHUMA_DEBUG_HEADERS", "false");
    std::env::set_var("SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN", "invalid-mode");

    let result = validate_env_only_once();
    assert!(result.is_err());
    assert!(result
        .err()
        .unwrap()
        .contains("SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN"));

    clear_env(&[
        "SHUMA_VALIDATE_ENV_IN_TESTS",
        "SHUMA_API_KEY",
        "SHUMA_JS_SECRET",
        "SHUMA_FORWARDED_IP_SECRET",
        "SHUMA_EVENT_LOG_RETENTION_HOURS",
        "SHUMA_ADMIN_CONFIG_WRITE_ENABLED",
        "SHUMA_KV_STORE_FAIL_OPEN",
        "SHUMA_ENFORCE_HTTPS",
        "SHUMA_DEBUG_HEADERS",
        "SHUMA_ENTERPRISE_MULTI_INSTANCE",
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "SHUMA_RATE_LIMITER_REDIS_URL",
        "SHUMA_BAN_STORE_REDIS_URL",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH",
    ]);
}

#[test]
fn validate_env_accepts_empty_optional_redis_url() {
    let _lock = crate::test_support::lock_env();
    clear_env(&[
        "SHUMA_VALIDATE_ENV_IN_TESTS",
        "SHUMA_API_KEY",
        "SHUMA_JS_SECRET",
        "SHUMA_FORWARDED_IP_SECRET",
        "SHUMA_EVENT_LOG_RETENTION_HOURS",
        "SHUMA_ADMIN_CONFIG_WRITE_ENABLED",
        "SHUMA_KV_STORE_FAIL_OPEN",
        "SHUMA_ENFORCE_HTTPS",
        "SHUMA_DEBUG_HEADERS",
        "SHUMA_ENTERPRISE_MULTI_INSTANCE",
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "SHUMA_RATE_LIMITER_REDIS_URL",
        "SHUMA_BAN_STORE_REDIS_URL",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH",
    ]);

    std::env::set_var("SHUMA_VALIDATE_ENV_IN_TESTS", "true");
    std::env::set_var("SHUMA_API_KEY", "test-admin-key");
    std::env::set_var("SHUMA_JS_SECRET", "test-js-secret");
    std::env::set_var("SHUMA_FORWARDED_IP_SECRET", "test-forwarded-secret");
    std::env::set_var("SHUMA_EVENT_LOG_RETENTION_HOURS", "168");
    std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "false");
    std::env::set_var("SHUMA_KV_STORE_FAIL_OPEN", "true");
    std::env::set_var("SHUMA_ENFORCE_HTTPS", "false");
    std::env::set_var("SHUMA_DEBUG_HEADERS", "false");
    std::env::set_var("SHUMA_RATE_LIMITER_REDIS_URL", "");

    let result = validate_env_only_once();
    assert!(result.is_ok());

    clear_env(&[
        "SHUMA_VALIDATE_ENV_IN_TESTS",
        "SHUMA_API_KEY",
        "SHUMA_JS_SECRET",
        "SHUMA_FORWARDED_IP_SECRET",
        "SHUMA_EVENT_LOG_RETENTION_HOURS",
        "SHUMA_ADMIN_CONFIG_WRITE_ENABLED",
        "SHUMA_KV_STORE_FAIL_OPEN",
        "SHUMA_ENFORCE_HTTPS",
        "SHUMA_DEBUG_HEADERS",
        "SHUMA_ENTERPRISE_MULTI_INSTANCE",
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "SHUMA_RATE_LIMITER_REDIS_URL",
        "SHUMA_BAN_STORE_REDIS_URL",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH",
    ]);
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
fn load_config_defaults_honeypot_enabled_when_key_missing() {
    let _lock = crate::test_support::lock_env();
    let store = crate::test_support::InMemoryStore::default();
    let mut kv_cfg_value = serde_json::to_value(defaults().clone()).unwrap();
    kv_cfg_value
        .as_object_mut()
        .expect("config json object")
        .remove("honeypot_enabled");
    store
        .set("config:default", &serde_json::to_vec(&kv_cfg_value).unwrap())
        .unwrap();

    let cfg = Config::load(&store, "default").unwrap();
    assert!(cfg.honeypot_enabled);
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
