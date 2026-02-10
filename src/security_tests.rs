use once_cell::sync::Lazy;
use spin_sdk::http::{Method, Request};
use std::sync::Mutex;

use crate::{forwarded_ip_trusted, response_with_optional_debug_headers};

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
    resp.headers().any(|(key, _)| key.eq_ignore_ascii_case(name))
}

#[test]
fn forwarded_headers_are_not_trusted_without_secret() {
    let _lock = ENV_MUTEX.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    std::env::remove_var("SHUMA_FORWARDED_IP_SECRET");

    let req = request_with_headers("/health", &[("x-forwarded-for", "127.0.0.1")]);
    assert!(!forwarded_ip_trusted(&req));
}

#[test]
fn health_internal_headers_hidden_by_default() {
    let _lock = ENV_MUTEX.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    std::env::remove_var("SHUMA_DEBUG_HEADERS");

    let resp = response_with_optional_debug_headers(200, "OK", "available", "open");

    assert!(!has_header(&resp, "X-KV-Status"));
    assert!(!has_header(&resp, "X-Shuma-Fail-Mode"));
}

#[test]
fn health_internal_headers_visible_when_enabled() {
    let _lock = ENV_MUTEX.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    std::env::set_var("SHUMA_DEBUG_HEADERS", "true");

    let resp = response_with_optional_debug_headers(200, "OK", "available", "open");

    assert!(has_header(&resp, "X-KV-Status"));
    assert!(has_header(&resp, "X-Shuma-Fail-Mode"));
}

#[test]
fn admin_options_preflight_is_rejected_without_cors_headers() {
    let _lock = ENV_MUTEX.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let req = request_with_method_and_headers(
        Method::Options,
        "/admin/config",
        &[
            ("origin", "https://example.com"),
            ("access-control-request-method", "POST"),
            ("access-control-request-headers", "authorization,content-type"),
        ],
    );

    let resp = crate::handle_bot_trap_impl(&req);

    assert_eq!(*resp.status(), 403u16);
    assert!(!has_header(&resp, "Access-Control-Allow-Origin"));
    assert!(!has_header(&resp, "Access-Control-Allow-Methods"));
    assert!(!has_header(&resp, "Access-Control-Allow-Headers"));
}

#[test]
fn forwarded_headers_are_trusted_with_matching_secret() {
    let _lock = ENV_MUTEX.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
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
fn geo_headers_are_ignored_when_forwarding_not_trusted() {
    let _lock = ENV_MUTEX.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    std::env::remove_var("SHUMA_FORWARDED_IP_SECRET");
    let req = request_with_headers("/health", &[("x-geo-country", "US")]);

    let cfg = crate::config::Config::load(&MockStore::default(), "default");
    let assessment = crate::assess_geo_request(&req, &cfg);
    assert!(!assessment.headers_trusted);
    assert_eq!(assessment.country, None);
    assert!(!assessment.scored_risk);
}

#[derive(Default)]
struct MockStore {
    map: std::sync::Mutex<std::collections::HashMap<String, Vec<u8>>>,
}

impl crate::challenge::KeyValueStore for MockStore {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ()> {
        let map = self.map.lock().unwrap();
        Ok(map.get(key).cloned())
    }

    fn set(&self, key: &str, value: &[u8]) -> Result<(), ()> {
        let mut map = self.map.lock().unwrap();
        map.insert(key.to_string(), value.to_vec());
        Ok(())
    }

    fn delete(&self, key: &str) -> Result<(), ()> {
        let mut map = self.map.lock().unwrap();
        map.remove(key);
        Ok(())
    }
}

#[test]
fn geo_headers_are_used_when_forwarding_is_trusted() {
    let _lock = ENV_MUTEX.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    std::env::set_var("SHUMA_FORWARDED_IP_SECRET", "test-forwarded-secret");
    let req = request_with_headers(
        "/health",
        &[
            ("x-geo-country", " us "),
            ("x-shuma-forwarded-secret", "test-forwarded-secret"),
        ],
    );

    let mut cfg = crate::config::Config::load(&MockStore::default(), "default");
    cfg.geo_risk = vec!["US".to_string()];
    let assessment = crate::assess_geo_request(&req, &cfg);
    assert!(assessment.headers_trusted);
    assert_eq!(assessment.country.as_deref(), Some("US"));
    assert!(assessment.scored_risk);
    std::env::remove_var("SHUMA_FORWARDED_IP_SECRET");
}
