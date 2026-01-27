// src/auth.rs
// Simple API key authentication for admin endpoints
// Checks for a static Bearer token in the Authorization header for admin access.

use spin_sdk::http::Request;

/// Static admin API key for demonstration. In production, use Spin secrets/config.
const ADMIN_API_KEY: &str = "changeme-supersecret"; // TODO: Use Spin secret/config in production

/// Returns true if the request contains a valid admin API key in the Authorization header.
pub fn is_authorized(req: &Request) -> bool {
    if let Some(header) = req.header("authorization") {
        let val = header.as_str().unwrap_or("");
        if val == format!("Bearer {}", ADMIN_API_KEY) {
            return true;
        }
    }
    false
}
