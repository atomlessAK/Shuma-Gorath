use crate::challenge::KeyValueStore;
use crate::whitelist;
use rand::Rng as _;
use serde::{Deserialize, Serialize};
use spin_sdk::http::{Method, Request};
use std::time::{SystemTime, UNIX_EPOCH};

const INSECURE_DEFAULT_API_KEY: &str = "changeme-supersecret";
const ADMIN_SESSION_COOKIE_NAME: &str = "shuma_admin_session";
const ADMIN_SESSION_KEY_PREFIX: &str = "admin_session:";
const ADMIN_SESSION_TTL_SECONDS: u64 = 3600;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AdminSessionRecord {
    csrf_token: String,
    expires_at: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdminAuthMethod {
    BearerToken,
    SessionCookie,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminAuthResult {
    pub method: Option<AdminAuthMethod>,
    pub csrf_token: Option<String>,
    pub session_id: Option<String>,
}

impl AdminAuthResult {
    pub fn unauthorized() -> Self {
        Self {
            method: None,
            csrf_token: None,
            session_id: None,
        }
    }

    pub fn is_authorized(&self) -> bool {
        self.method.is_some()
    }

    pub fn requires_csrf(&self, req: &Request) -> bool {
        self.method == Some(AdminAuthMethod::SessionCookie) && method_is_write(req.method())
    }
}

fn now_ts() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn method_is_write(method: &Method) -> bool {
    matches!(
        method,
        Method::Post | Method::Put | Method::Patch | Method::Delete
    )
}

fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.bytes().zip(b.bytes()) {
        diff |= x ^ y;
    }
    diff == 0
}

fn to_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

fn random_hex(num_bytes: usize) -> String {
    let mut rng = rand::rng();
    let mut bytes = vec![0u8; num_bytes];
    rng.fill(bytes.as_mut_slice());
    to_hex(&bytes)
}

fn session_store_key(session_id: &str) -> String {
    format!("{}{}", ADMIN_SESSION_KEY_PREFIX, session_id)
}

fn parse_cookie(req: &Request, key: &str) -> Option<String> {
    let cookie_header = req.header("cookie")?.as_str()?;
    for part in cookie_header.split(';') {
        let trimmed = part.trim();
        let mut kv = trimmed.splitn(2, '=');
        let k = kv.next()?.trim();
        let v = kv.next()?.trim();
        if k == key && !v.is_empty() {
            return Some(v.to_string());
        }
    }
    None
}

fn get_admin_api_key() -> Option<String> {
    let key = std::env::var("SHUMA_API_KEY").ok()?;
    let key = key.trim();
    if key.is_empty() {
        return None;
    }
    if key == INSECURE_DEFAULT_API_KEY {
        return None;
    }
    Some(key.to_string())
}

pub fn is_admin_api_key_configured() -> bool {
    get_admin_api_key().is_some()
}

pub fn verify_admin_api_key_candidate(candidate: &str) -> bool {
    let Some(expected) = get_admin_api_key() else {
        return false;
    };
    let candidate = candidate.trim();
    constant_time_eq(candidate, &expected)
}

fn bearer_token(req: &Request) -> Option<String> {
    let header = req.header("authorization")?.as_str()?;
    let prefix = "Bearer ";
    if !header.starts_with(prefix) {
        return None;
    }
    Some(header[prefix.len()..].trim().to_string())
}

pub fn is_bearer_authorized(req: &Request) -> bool {
    let Some(candidate) = bearer_token(req) else {
        return false;
    };
    verify_admin_api_key_candidate(&candidate)
}

pub fn has_admin_session_cookie(req: &Request) -> bool {
    parse_cookie(req, ADMIN_SESSION_COOKIE_NAME).is_some()
}

pub fn get_admin_id(req: &Request) -> String {
    if is_bearer_authorized(req) || has_admin_session_cookie(req) {
        "admin".to_string()
    } else {
        "-".to_string()
    }
}

fn load_session_record<S: KeyValueStore>(
    store: &S,
    session_id: &str,
) -> Option<AdminSessionRecord> {
    let key = session_store_key(session_id);
    let raw = store.get(&key).ok()??;
    let parsed = serde_json::from_slice::<AdminSessionRecord>(&raw).ok()?;
    if parsed.expires_at <= now_ts() {
        let _ = store.delete(&key);
        return None;
    }
    Some(parsed)
}

pub fn authenticate_admin<S: KeyValueStore>(req: &Request, store: &S) -> AdminAuthResult {
    if is_bearer_authorized(req) {
        return AdminAuthResult {
            method: Some(AdminAuthMethod::BearerToken),
            csrf_token: None,
            session_id: None,
        };
    }

    let Some(session_id) = parse_cookie(req, ADMIN_SESSION_COOKIE_NAME) else {
        return AdminAuthResult::unauthorized();
    };
    let Some(record) = load_session_record(store, &session_id) else {
        return AdminAuthResult::unauthorized();
    };
    AdminAuthResult {
        method: Some(AdminAuthMethod::SessionCookie),
        csrf_token: Some(record.csrf_token),
        session_id: Some(session_id),
    }
}

pub fn validate_session_csrf(req: &Request, expected_csrf: &str) -> bool {
    let Some(header) = req.header("x-shuma-csrf").and_then(|v| v.as_str()) else {
        return false;
    };
    constant_time_eq(header.trim(), expected_csrf)
}

pub fn create_admin_session<S: KeyValueStore>(store: &S) -> Result<(String, String, u64), ()> {
    let session_id = random_hex(32);
    let csrf_token = random_hex(16);
    let expires_at = now_ts().saturating_add(ADMIN_SESSION_TTL_SECONDS);
    let record = AdminSessionRecord {
        csrf_token: csrf_token.clone(),
        expires_at,
    };
    let value = serde_json::to_vec(&record).map_err(|_| ())?;
    store.set(&session_store_key(&session_id), &value)?;
    Ok((session_id, csrf_token, expires_at))
}

pub fn clear_admin_session<S: KeyValueStore>(store: &S, req: &Request) -> Result<(), ()> {
    if let Some(session_id) = parse_cookie(req, ADMIN_SESSION_COOKIE_NAME) {
        store.delete(&session_store_key(&session_id))?;
    }
    Ok(())
}

pub fn admin_session_cookie_name() -> &'static str {
    ADMIN_SESSION_COOKIE_NAME
}

pub fn admin_session_ttl_seconds() -> u64 {
    ADMIN_SESSION_TTL_SECONDS
}

/// Returns true if admin access is allowed from this IP.
/// If SHUMA_ADMIN_IP_ALLOWLIST is unset, all IPs are allowed (auth still required).
pub fn is_admin_ip_allowed(req: &Request) -> bool {
    let list = match std::env::var("SHUMA_ADMIN_IP_ALLOWLIST") {
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
    use std::collections::HashMap;
    use std::sync::Mutex;

    static ENV_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[derive(Default)]
    struct MockStore {
        data: Mutex<HashMap<String, Vec<u8>>>,
    }

    impl KeyValueStore for MockStore {
        fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ()> {
            Ok(self.data.lock().unwrap().get(key).cloned())
        }

        fn set(&self, key: &str, value: &[u8]) -> Result<(), ()> {
            self.data
                .lock()
                .unwrap()
                .insert(key.to_string(), value.to_vec());
            Ok(())
        }

        fn delete(&self, key: &str) -> Result<(), ()> {
            self.data.lock().unwrap().remove(key);
            Ok(())
        }
    }

    fn request_with_auth(auth_header: Option<&str>) -> Request {
        let mut builder = Request::builder();
        builder.method(Method::Get).uri("/admin/config");
        if let Some(auth) = auth_header {
            builder.header("authorization", auth);
        }
        builder.build()
    }

    #[test]
    fn unauthorized_when_api_key_missing() {
        let _lock = ENV_MUTEX.lock().expect("failed to lock env mutex");
        std::env::remove_var("SHUMA_API_KEY");
        let req = request_with_auth(Some("Bearer any-key"));
        assert!(!is_bearer_authorized(&req));
    }

    #[test]
    fn unauthorized_when_api_key_is_insecure_default() {
        let _lock = ENV_MUTEX.lock().expect("failed to lock env mutex");
        std::env::set_var("SHUMA_API_KEY", INSECURE_DEFAULT_API_KEY);
        let req = request_with_auth(Some("Bearer changeme-supersecret"));
        assert!(!is_bearer_authorized(&req));
    }

    #[test]
    fn unauthorized_when_api_key_is_insecure_default_always() {
        let _lock = ENV_MUTEX.lock().expect("failed to lock env mutex");
        std::env::set_var("SHUMA_API_KEY", INSECURE_DEFAULT_API_KEY);
        let req = request_with_auth(Some("Bearer changeme-supersecret"));
        assert!(!is_bearer_authorized(&req));
    }

    #[test]
    fn authorized_when_api_key_is_explicitly_set() {
        let _lock = ENV_MUTEX.lock().expect("failed to lock env mutex");
        std::env::set_var("SHUMA_API_KEY", "test-admin-key");
        let req = request_with_auth(Some("Bearer test-admin-key"));
        assert!(is_bearer_authorized(&req));
    }

    #[test]
    fn create_and_authenticate_cookie_session() {
        let _lock = ENV_MUTEX.lock().expect("failed to lock env mutex");
        std::env::set_var("SHUMA_API_KEY", "test-admin-key");
        let store = MockStore::default();
        let (session_id, csrf_token, _expires) =
            create_admin_session(&store).expect("session should be created");
        assert!(!session_id.is_empty());
        assert!(!csrf_token.is_empty());

        let mut builder = Request::builder();
        builder
            .method(Method::Post)
            .uri("/admin/config")
            .header(
                "cookie",
                format!("{}={}", admin_session_cookie_name(), session_id),
            )
            .header("x-shuma-csrf", csrf_token.as_str());
        let req = builder.build();

        let auth = authenticate_admin(&req, &store);
        assert_eq!(auth.method, Some(AdminAuthMethod::SessionCookie));
        assert!(auth.requires_csrf(&req));
        assert!(validate_session_csrf(
            &req,
            auth.csrf_token.as_deref().unwrap_or("")
        ));
    }
}
