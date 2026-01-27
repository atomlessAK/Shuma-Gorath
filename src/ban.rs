// src/ban.rs
// Ban list management for WASM Bot Trap
// Handles persistent IP bans, expiry, and ban reasons using the Spin key-value store.

use spin_sdk::key_value::Store;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Serialize, Deserialize};

/// Represents a ban entry for an IP address, including reason and expiry timestamp.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BanEntry {
    pub reason: String,
    pub expires: u64,
}


/// Checks if an IP is currently banned for a given site.
/// Returns true if the ban is active, false otherwise. Cleans up expired/invalid bans.
pub fn is_banned(store: &Store, site_id: &str, ip: &str) -> bool {
    let key = format!("ban:{}:{}", site_id, ip);
    match store.get(&key) {
        Ok(Some(val)) => {
            if let Ok(json) = serde_json::from_slice::<BanEntry>(&val) {
                let now = now_ts();
                if json.expires > now {
                    // log: ban_check
                    return true;
                } else {
                    let _ = store.delete(&key);
                }
            } else {
                let _ = store.delete(&key);
            }
        }
        Ok(None) => {}
        Err(_) => {}
    }
    false
}

/// Bans an IP for a given site, reason, and duration (in seconds).
/// Stores the ban entry in the key-value store.
pub fn ban_ip(store: &Store, site_id: &str, ip: &str, reason: &str, duration_secs: u64) {
    let key = format!("ban:{}:{}", site_id, ip);
    let entry = BanEntry {
        reason: reason.to_string(),
        expires: now_ts() + duration_secs,
    };
    if let Ok(val) = serde_json::to_vec(&entry) {
        let _ = store.set(&key, &val);
        // log: ban_add
    }
}

/// Returns the current UNIX timestamp in seconds (used for ban expiry).
fn now_ts() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}
