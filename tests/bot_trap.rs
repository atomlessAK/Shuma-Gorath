// tests/bot_trap.rs
// Integration tests for WASM Bot Trap

use spin_sdk::http::{Request, Response, Method};
use wasm_bot_trap::handle_bot_trap_impl;

fn make_request(path: &str, ip: &str, ua: &str) -> Request {
    let mut req = Request::new(Method::Get, path);
    req.headers().insert("x-forwarded-for", ip.parse().unwrap());
    req.headers().insert("user-agent", ua.parse().unwrap());
    req
}

#[test]
fn test_whitelisted_ip() {
    // Should return 200 for whitelisted IP
    let req = make_request("/", "1.2.3.4", "TestUA");
    let resp = handle_bot_trap_impl(&req);
    assert_eq!(resp.status(), 200);
}

#[test]
fn test_banned_ip() {
    // Should return 403 for banned IP (simulate ban)
    // TODO: Insert ban in store, then test
}

#[test]
fn test_honeypot() {
    // Should ban and block on honeypot path
    let req = make_request("/bot-trap", "5.6.7.8", "TestUA");
    let resp = handle_bot_trap_impl(&req);
    assert_eq!(resp.status(), 403);
}

#[test]
fn test_rate_limit() {
    // Should ban and block if rate limit exceeded
    // TODO: Simulate rate limit exceeded
}

#[test]
fn test_js_challenge() {
    // Should inject JS challenge if needed
    // TODO: Simulate missing js_verified cookie
}

#[test]
fn test_admin_unauthorized() {
    // Should return 401 for missing API key
    let req = make_request("/admin", "1.2.3.4", "TestUA");
    let resp = handle_bot_trap_impl(&req);
    assert_eq!(resp.status(), 401);
}
