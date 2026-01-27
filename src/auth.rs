// src/auth.rs
// Simple API key authentication for admin endpoints
// Checks for a static Bearer token in the Authorization header for admin access.

use spin_sdk::http::Request;


/// Returns the admin API key: uses the API_KEY environment variable if set, otherwise falls back to the hardcoded dev key.
fn get_admin_api_key() -> String {
    // Use Spin's std::env::var to read the environment variable if present
    std::env::var("API_KEY").unwrap_or_else(|_| "changeme-supersecret".to_string())
}

/// Returns true if the request contains a valid admin API key in the Authorization header.
pub fn is_authorized(req: &Request) -> bool {
    if let Some(header) = req.header("authorization") {
        let val = header.as_str().unwrap_or("");
        let expected = format!("Bearer {}", get_admin_api_key());
        if val == expected {
            return true;
        }
    }
    false
}
