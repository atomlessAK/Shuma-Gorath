// src/js.rs
// JavaScript verification and quiz logic for WASM Bot Trap
// Handles JS-based bot detection and challenge/response for suspicious clients.

use spin_sdk::http::{Request, Response};
use spin_sdk::key_value::Store;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use base64::{engine::general_purpose, Engine as _};

/// Secret used for HMAC token generation for JS verification cookies.
const JS_SECRET: &str = "js-challenge-secret";

/// Generates a HMAC-based token for a given IP, used in the js_verified cookie.
fn make_token(ip: &str) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(JS_SECRET.as_bytes()).unwrap();
    mac.update(ip.as_bytes());
    let result = mac.finalize().into_bytes();
    general_purpose::STANDARD.encode(result)
}

/// Returns true if the request needs JS verification (no valid js_verified cookie).
/// Checks for a valid js_verified cookie matching the HMAC token for the IP.
pub fn needs_js_verification(req: &Request, _store: &Store, _site_id: &str, ip: &str) -> bool {
    // Check for a valid js_verified cookie
    if let Some(header) = req.header("cookie") {
        let cookie = header.as_str().unwrap_or("");
        for part in cookie.split(';') {
            let trimmed = part.trim();
            if let Some(val) = trimmed.strip_prefix("js_verified=") {
                if val == make_token(ip) {
                    return false;
                }
            }
        }
    }
    true
}

/// Returns a Response with a JS challenge page that sets the js_verified cookie for the client IP.
pub fn inject_js_challenge(ip: &str) -> Response {
        let token = make_token(ip);
        let html = format!(r#"
        <html><body>
        <script>
            document.cookie = 'js_verified={token}; path=/; SameSite=Strict';
      window.location.reload();
    </script>
    <noscript>Please enable JavaScript to continue.</noscript>
    </body></html>
    "#);
    Response::new(200, html)
}
