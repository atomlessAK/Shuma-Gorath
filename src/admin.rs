// src/admin.rs
// Admin API endpoints for WASM Bot Trap
// Provides HTTP endpoints for ban management and analytics, protected by API key auth.

use spin_sdk::http::{Request, Response};
use spin_sdk::key_value::Store;
use serde_json::json;

/// Returns true if the path is a valid admin endpoint (prevents path traversal/abuse).
fn sanitize_path(path: &str) -> bool {
    matches!(path, "/admin" | "/admin/ban" | "/admin/unban" | "/admin/analytics")
}

/// Handles all /admin API endpoints. Requires valid API key in Authorization header.
/// Supports:
///   - /admin/ban: List all bans for the site
///   - /admin/unban?ip=...: Remove a ban for an IP
///   - /admin/analytics: Return ban count
///   - /admin: API help
pub fn handle_admin(req: &Request) -> Response {
    // Require valid API key
    if !crate::auth::is_authorized(req) {
        return Response::new(401, "Unauthorized: Invalid or missing API key");
    }
    let path = req.path();
    if !sanitize_path(path) {
        return Response::new(400, "Bad Request: Invalid admin endpoint");
    }
    let store = Store::open_default().expect("open default store");
    let site_id = "default";

    match path {
        "/admin/ban" => {
            // List all bans for this site (keys starting with ban:site_id:)
            let mut bans = vec![];
            if let Ok(keys) = store.get_keys() {
                for k in keys {
                    if k.starts_with(&format!("ban:{}:", site_id)) {
                        if let Ok(Some(val)) = store.get(&k) {
                            if let Ok(ban) = serde_json::from_slice::<crate::ban::BanEntry>(&val) {
                                bans.push(json!({"ip": k.split(':').last().unwrap_or("?"), "reason": ban.reason, "expires": ban.expires}));
                            }
                        }
                    }
                }
            }
            let body = serde_json::to_string(&json!({"bans": bans})).unwrap();
            Response::new(200, body)
        }
        "/admin/unban" => {
            // Unban IP (expects ?ip=...)
            let ip = req.query().strip_prefix("ip=").unwrap_or("");
            if ip.is_empty() {
                return Response::new(400, "Missing ip param");
            }
            let key = format!("ban:{}:{}", site_id, ip);
            let _ = store.delete(&key);
            Response::new(200, "Unbanned")
        }
        "/admin/analytics" => {
            // Return simple analytics: ban count
            let mut ban_count = 0;
            if let Ok(keys) = store.get_keys() {
                for k in keys {
                    if k.starts_with(&format!("ban:{}:", site_id)) {
                        ban_count += 1;
                    }
                }
            }
            let body = serde_json::to_string(&json!({"ban_count": ban_count})).unwrap();
            Response::new(200, body)
        }
        "/admin" => {
            // API help endpoint
            Response::new(200, "WASM Bot Trap Admin API. Use /admin/ban, /admin/unban?ip=IP, /admin/analytics.")
        }
        _ => Response::new(404, "Not found"),
    }
}
