use once_cell::sync::Lazy;
use spin_sdk::http::{Method, Request};
use std::sync::Mutex;

use crate::{
    extract_health_client_ip, forwarded_ip_trusted, health_secret_authorized,
    response_with_optional_debug_headers,
};

static ENV_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

fn request_with_headers(path: &str, headers: &[(&str, &str)]) -> Request {
    request_with_method_and_headers(Method::Get, path, headers)
}

fn request_with_method_and_headers(
    method: Method,
    path: &str,
    headers: &[(&str, &str)],
) -> Request {
    let mut builder = Request::builder();
    builder.method(method).uri(path);
    for (key, value) in headers {
        builder.header(*key, *value);
    }
    builder.build()
}

fn has_header(resp: &spin_sdk::http::Response, name: &str) -> bool {
    resp.headers()
        .any(|(key, _)| key.eq_ignore_ascii_case(name))
}

#[test]
fn forwarded_headers_are_not_trusted_without_secret() {
    let _lock = ENV_MUTEX
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    std::env::remove_var("SHUMA_FORWARDED_IP_SECRET");

    let req = request_with_headers("/health", &[("x-forwarded-for", "127.0.0.1")]);
    assert!(!forwarded_ip_trusted(&req));
}

#[test]
fn health_internal_headers_hidden_by_default() {
    let _lock = ENV_MUTEX
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    std::env::remove_var("SHUMA_DEBUG_HEADERS");

    let resp = response_with_optional_debug_headers(200, "OK", "available", "open");

    assert!(!has_header(&resp, "X-KV-Status"));
    assert!(!has_header(&resp, "X-Shuma-Fail-Mode"));
}

#[test]
fn health_internal_headers_visible_when_enabled() {
    let _lock = ENV_MUTEX
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    std::env::set_var("SHUMA_DEBUG_HEADERS", "true");

    let resp = response_with_optional_debug_headers(200, "OK", "available", "open");

    assert!(has_header(&resp, "X-KV-Status"));
    assert!(has_header(&resp, "X-Shuma-Fail-Mode"));
}

#[test]
fn health_secret_not_required_when_unset() {
    let _lock = ENV_MUTEX
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    std::env::remove_var("SHUMA_HEALTH_SECRET");
    let req = request_with_headers("/health", &[("x-forwarded-for", "127.0.0.1")]);
    assert!(health_secret_authorized(&req));
}

#[test]
fn health_secret_required_when_configured() {
    let _lock = ENV_MUTEX
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    std::env::set_var("SHUMA_HEALTH_SECRET", "health-secret-value");

    let missing = request_with_headers("/health", &[("x-forwarded-for", "127.0.0.1")]);
    assert!(!health_secret_authorized(&missing));

    let wrong = request_with_headers(
        "/health",
        &[
            ("x-forwarded-for", "127.0.0.1"),
            ("x-shuma-health-secret", "wrong-value"),
        ],
    );
    assert!(!health_secret_authorized(&wrong));

    let correct = request_with_headers(
        "/health",
        &[
            ("x-forwarded-for", "127.0.0.1"),
            ("x-shuma-health-secret", "health-secret-value"),
        ],
    );
    assert!(health_secret_authorized(&correct));

    std::env::remove_var("SHUMA_HEALTH_SECRET");
}

#[test]
fn health_endpoint_rejects_when_health_secret_missing() {
    let _lock = ENV_MUTEX
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    std::env::set_var("SHUMA_FORWARDED_IP_SECRET", "test-forwarded-secret");
    std::env::set_var("SHUMA_HEALTH_SECRET", "health-secret-value");
    let req = request_with_headers(
        "/health",
        &[
            ("x-forwarded-for", "127.0.0.1"),
            ("x-shuma-forwarded-secret", "test-forwarded-secret"),
        ],
    );

    let resp = crate::handle_bot_defence_impl(&req);
    assert_eq!(*resp.status(), 403u16);

    std::env::remove_var("SHUMA_HEALTH_SECRET");
    std::env::remove_var("SHUMA_FORWARDED_IP_SECRET");
}

#[test]
fn https_enforcement_blocks_insecure_admin_requests() {
    let _lock = ENV_MUTEX
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    std::env::set_var("SHUMA_ENFORCE_HTTPS", "true");
    std::env::remove_var("SHUMA_FORWARDED_IP_SECRET");

    let req = request_with_method_and_headers(Method::Get, "/admin/config", &[]);
    let resp = crate::handle_bot_defence_impl(&req);

    assert_eq!(*resp.status(), 403u16);
    assert_eq!(
        String::from_utf8(resp.into_body()).unwrap(),
        "HTTPS required"
    );

    std::env::remove_var("SHUMA_ENFORCE_HTTPS");
}

#[test]
fn https_enforcement_allows_trusted_forwarded_https_to_reach_admin_auth() {
    let _lock = ENV_MUTEX
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    std::env::set_var("SHUMA_ENFORCE_HTTPS", "true");
    std::env::set_var("SHUMA_FORWARDED_IP_SECRET", "test-forwarded-secret");
    std::env::set_var("SHUMA_API_KEY", "test-admin-key");

    let req = request_with_method_and_headers(
        Method::Get,
        "/admin/config",
        &[
            ("x-shuma-forwarded-secret", "test-forwarded-secret"),
            ("x-forwarded-proto", "https"),
        ],
    );
    let resp = crate::handle_bot_defence_impl(&req);

    assert_eq!(*resp.status(), 401u16);

    std::env::remove_var("SHUMA_ENFORCE_HTTPS");
    std::env::remove_var("SHUMA_FORWARDED_IP_SECRET");
    std::env::remove_var("SHUMA_API_KEY");
}

#[test]
fn admin_options_preflight_is_rejected_without_cors_headers() {
    let _lock = ENV_MUTEX
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let req = request_with_method_and_headers(
        Method::Options,
        "/admin/config",
        &[
            ("origin", "https://example.com"),
            ("access-control-request-method", "POST"),
            (
                "access-control-request-headers",
                "authorization,content-type",
            ),
        ],
    );

    let resp = crate::handle_bot_defence_impl(&req);

    assert_eq!(*resp.status(), 403u16);
    assert!(!has_header(&resp, "Access-Control-Allow-Origin"));
    assert!(!has_header(&resp, "Access-Control-Allow-Methods"));
    assert!(!has_header(&resp, "Access-Control-Allow-Headers"));
}

#[test]
fn forwarded_headers_are_trusted_with_matching_secret() {
    let _lock = ENV_MUTEX
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    std::env::set_var("SHUMA_FORWARDED_IP_SECRET", "test-forwarded-secret");
    let req = request_with_headers(
        "/health",
        &[
            ("x-forwarded-for", "127.0.0.1"),
            ("x-shuma-forwarded-secret", "test-forwarded-secret"),
        ],
    );
    assert!(forwarded_ip_trusted(&req));
}

#[test]
fn health_ip_extraction_rejects_multi_hop_forwarded_for() {
    let _lock = ENV_MUTEX
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    std::env::set_var("SHUMA_FORWARDED_IP_SECRET", "test-forwarded-secret");
    let req = request_with_headers(
        "/health",
        &[
            ("x-forwarded-for", "127.0.0.1, 203.0.113.10"),
            ("x-shuma-forwarded-secret", "test-forwarded-secret"),
        ],
    );

    assert_eq!(extract_health_client_ip(&req), "unknown");
    std::env::remove_var("SHUMA_FORWARDED_IP_SECRET");
}

#[test]
fn geo_headers_are_ignored_when_forwarding_not_trusted() {
    let _lock = ENV_MUTEX
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    std::env::remove_var("SHUMA_FORWARDED_IP_SECRET");
    let req = request_with_headers("/health", &[("x-geo-country", "US")]);

    let cfg = crate::config::defaults().clone();
    let assessment = crate::assess_geo_request(&req, &cfg);
    assert!(!assessment.headers_trusted);
    assert_eq!(assessment.country, None);
    assert!(!assessment.scored_risk);
}

#[test]
fn geo_headers_are_used_when_forwarding_is_trusted() {
    let _lock = ENV_MUTEX
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    std::env::set_var("SHUMA_FORWARDED_IP_SECRET", "test-forwarded-secret");
    let req = request_with_headers(
        "/health",
        &[
            ("x-geo-country", " us "),
            ("x-shuma-forwarded-secret", "test-forwarded-secret"),
        ],
    );

    let mut cfg = crate::config::defaults().clone();
    cfg.geo_risk = vec!["US".to_string()];
    let assessment = crate::assess_geo_request(&req, &cfg);
    assert!(assessment.headers_trusted);
    assert_eq!(assessment.country.as_deref(), Some("US"));
    assert!(assessment.scored_risk);
    std::env::remove_var("SHUMA_FORWARDED_IP_SECRET");
}

#[test]
fn invalid_bool_env_returns_500_without_panicking() {
    let _lock = ENV_MUTEX
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    std::env::set_var("SHUMA_VALIDATE_ENV_IN_TESTS", "true");
    std::env::set_var("SHUMA_API_KEY", "test-admin-key");
    std::env::set_var("SHUMA_JS_SECRET", "test-js-secret");
    std::env::set_var("SHUMA_FORWARDED_IP_SECRET", "test-forwarded-secret");
    std::env::set_var("SHUMA_EVENT_LOG_RETENTION_HOURS", "168");
    std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "false");
    std::env::set_var("SHUMA_KV_STORE_FAIL_OPEN", "true");
    std::env::set_var("SHUMA_ENFORCE_HTTPS", "not-a-bool");
    std::env::set_var("SHUMA_DEBUG_HEADERS", "false");
    std::env::set_var("SHUMA_POW_CONFIG_MUTABLE", "false");
    std::env::set_var("SHUMA_CHALLENGE_CONFIG_MUTABLE", "false");
    std::env::set_var("SHUMA_BOTNESS_CONFIG_MUTABLE", "false");

    let req = request_with_method_and_headers(Method::Get, "/health", &[]);
    let result = std::panic::catch_unwind(|| crate::handle_bot_defence_impl(&req));
    assert!(result.is_ok(), "handler panicked on invalid bool env");

    let resp = result.unwrap();
    assert_eq!(*resp.status(), 500u16);
    assert_eq!(
        String::from_utf8(resp.into_body()).unwrap(),
        "Server configuration error"
    );

    std::env::remove_var("SHUMA_VALIDATE_ENV_IN_TESTS");
    std::env::remove_var("SHUMA_API_KEY");
    std::env::remove_var("SHUMA_JS_SECRET");
    std::env::remove_var("SHUMA_FORWARDED_IP_SECRET");
    std::env::remove_var("SHUMA_EVENT_LOG_RETENTION_HOURS");
    std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
    std::env::remove_var("SHUMA_KV_STORE_FAIL_OPEN");
    std::env::remove_var("SHUMA_ENFORCE_HTTPS");
    std::env::remove_var("SHUMA_DEBUG_HEADERS");
    std::env::remove_var("SHUMA_POW_CONFIG_MUTABLE");
    std::env::remove_var("SHUMA_CHALLENGE_CONFIG_MUTABLE");
    std::env::remove_var("SHUMA_BOTNESS_CONFIG_MUTABLE");
}
