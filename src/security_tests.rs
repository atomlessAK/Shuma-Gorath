use once_cell::sync::Lazy;
use spin_sdk::http::{Method, Request};
use std::sync::Mutex;

use crate::{forwarded_ip_trusted, response_with_optional_debug_headers};

static ENV_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

fn request_with_headers(path: &str, headers: &[(&str, &str)]) -> Request {
    let mut builder = Request::builder();
    builder.method(Method::Get).uri(path);
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
