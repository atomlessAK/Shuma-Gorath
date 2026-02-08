/// Returns a simple admin identifier for event logging (e.g., 'admin' if authorized, '-' otherwise)
pub fn get_admin_id(req: &Request) -> String {
    if is_authorized(req) {
        "admin".to_string()
    } else {
        "-".to_string()
    }
}
// src/auth.rs
// Simple API key authentication for admin endpoints
// Checks for a static Bearer token in the Authorization header for admin access.

use spin_sdk::http::Request;
use crate::whitelist;

const INSECURE_DEFAULT_API_KEY: &str = "changeme-supersecret";

/// Returns the admin API key only when explicitly configured and non-default.
fn get_admin_api_key() -> Option<String> {
    let key = std::env::var("API_KEY").ok()?;
    let key = key.trim();
    if key.is_empty() || key == INSECURE_DEFAULT_API_KEY {
        return None;
    }
    Some(key.to_string())
}

pub fn is_admin_api_key_configured() -> bool {
    get_admin_api_key().is_some()
}

/// Returns true if the request contains a valid admin API key in the Authorization header.
/// Uses constant-time comparison to prevent timing attacks.
pub fn is_authorized(req: &Request) -> bool {
    let key = match get_admin_api_key() {
        Some(key) => key,
        None => return false,
    };

    if let Some(header) = req.header("authorization") {
        let val = header.as_str().unwrap_or("");
        let expected = format!("Bearer {}", key);
        // Use constant-time comparison to prevent timing attacks
        if val.len() == expected.len() {
            let mut result = 0u8;
            for (a, b) in val.bytes().zip(expected.bytes()) {
                result |= a ^ b;
            }
            return result == 0;
        }
    }
    false
}

/// Returns true if admin access is allowed from this IP.
/// If ADMIN_IP_ALLOWLIST is unset, all IPs are allowed (auth still required).
pub fn is_admin_ip_allowed(req: &Request) -> bool {
    let list = match std::env::var("ADMIN_IP_ALLOWLIST") {
        Ok(v) => {
            let items: Vec<String> = v
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if items.is_empty() {
                return true;
            }
            items
        }
        Err(_) => return true,
    };

    let ip = crate::extract_client_ip(req);
    whitelist::is_whitelisted(&ip, &list)
}

#[cfg(test)]
mod tests {
    use super::*;
    use once_cell::sync::Lazy;
    use spin_sdk::http::Request;
    use std::sync::Mutex;

    static ENV_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    fn request_with_auth(auth_header: Option<&str>) -> Request {
        let mut builder = Request::builder();
        builder.method(spin_sdk::http::Method::Get).uri("/admin/config");
        if let Some(auth) = auth_header {
            builder.header("authorization", auth);
        }
        builder.build()
    }

    #[test]
    fn unauthorized_when_api_key_missing() {
        let _lock = ENV_MUTEX.lock().expect("failed to lock env mutex");
        std::env::remove_var("API_KEY");
        let req = request_with_auth(Some("Bearer any-key"));
        assert!(!is_authorized(&req));
    }

    #[test]
    fn unauthorized_when_api_key_is_insecure_default() {
        let _lock = ENV_MUTEX.lock().expect("failed to lock env mutex");
        std::env::set_var("API_KEY", INSECURE_DEFAULT_API_KEY);
        let req = request_with_auth(Some("Bearer changeme-supersecret"));
        assert!(!is_authorized(&req));
    }

    #[test]
    fn authorized_when_api_key_is_explicitly_set() {
        let _lock = ENV_MUTEX.lock().expect("failed to lock env mutex");
        std::env::set_var("API_KEY", "test-admin-key");
        let req = request_with_auth(Some("Bearer test-admin-key"));
        assert!(is_authorized(&req));
    }
}
