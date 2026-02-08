// src/ban.rs
// Ban list management for WASM Bot Trap
// Handles persistent IP bans, expiry, and ban reasons using the Spin key-value store.

use crate::challenge::KeyValueStore;
use spin_sdk::key_value::Store;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Serialize, Deserialize};

/// Structured signal snapshot captured when a ban is created.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BanFingerprint {
    #[serde(default)]
    pub score: Option<u8>,
    #[serde(default)]
    pub signals: Vec<String>,
    #[serde(default)]
    pub summary: Option<String>,
}

/// Represents a ban entry for an IP address, including reason and expiry timestamp.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BanEntry {
    pub reason: String,
    pub expires: u64,
    #[serde(default = "now_ts")]
    pub banned_at: u64,
    #[serde(default)]
    pub fingerprint: Option<BanFingerprint>,
}

fn ban_index_key(site_id: &str) -> String {
    format!("ban_index:{}", site_id)
}

fn load_ban_index(store: &impl KeyValueStore, site_id: &str) -> Vec<String> {
    let key = ban_index_key(site_id);
    store
        .get(&key)
        .ok()
        .flatten()
        .and_then(|v| serde_json::from_slice::<Vec<String>>(&v).ok())
        .unwrap_or_default()
}

fn save_ban_index(store: &impl KeyValueStore, site_id: &str, index: &[String]) {
    if let Ok(val) = serde_json::to_vec(index) {
        let _ = store.set(&ban_index_key(site_id), &val);
    }
}

fn add_to_ban_index(store: &impl KeyValueStore, site_id: &str, ip: &str) {
    let mut index = load_ban_index(store, site_id);
    if !index.iter().any(|v| v == ip) {
        index.push(ip.to_string());
        save_ban_index(store, site_id, &index);
    }
}

fn remove_from_ban_index(store: &impl KeyValueStore, site_id: &str, ip: &str) {
    let mut index = load_ban_index(store, site_id);
    let before = index.len();
    index.retain(|v| v != ip);
    if index.len() != before {
        save_ban_index(store, site_id, &index);
    }
}

/// Returns all active bans and prunes expired/missing entries from the index.
pub fn list_active_bans(store: &impl KeyValueStore, site_id: &str) -> Vec<(String, BanEntry)> {
    let index = load_ban_index(store, site_id);
    let original_len = index.len();
    let now = now_ts();
    let mut active = Vec::new();
    let mut new_index = Vec::new();
    let mut changed = false;

    for ip in index {
        let key = format!("ban:{}:{}", site_id, ip);
        match store.get(&key) {
            Ok(Some(val)) => {
                if let Ok(entry) = serde_json::from_slice::<BanEntry>(&val) {
                    if entry.expires > now {
                        new_index.push(ip.clone());
                        active.push((ip, entry));
                    } else {
                        let _ = store.delete(&key);
                        changed = true;
                    }
                } else {
                    let _ = store.delete(&key);
                    changed = true;
                }
            }
            _ => {
                changed = true;
            }
        }
    }

    if changed || new_index.len() != original_len {
        save_ban_index(store, site_id, &new_index);
    }

    active
}

/// Store-aware variant that can rebuild the index from existing ban keys when empty.
pub fn list_active_bans_with_scan(store: &Store, site_id: &str) -> Vec<(String, BanEntry)> {
    let mut active = list_active_bans(store, site_id);
    if !active.is_empty() {
        return active;
    }

    // If index is empty but bans exist (pre-index migration), rebuild once.
    let mut rebuilt_index = Vec::new();
    let now = now_ts();
    if let Ok(keys) = store.get_keys() {
        for k in keys {
            if k.starts_with(&format!("ban:{}:", site_id)) {
                if let Ok(Some(val)) = store.get(&k) {
                    if let Ok(entry) = serde_json::from_slice::<BanEntry>(&val) {
                        if entry.expires > now {
                            if let Some(ip) = k.split(':').last() {
                                rebuilt_index.push(ip.to_string());
                                active.push((ip.to_string(), entry));
                            }
                        } else {
                            let _ = store.delete(&k);
                        }
                    } else {
                        let _ = store.delete(&k);
                    }
                }
            }
        }
    }

    if !rebuilt_index.is_empty() {
        save_ban_index(store, site_id, &rebuilt_index);
    }

    active
}

/// Checks if an IP is currently banned for a given site.
/// Returns true if the ban is active, false otherwise. Cleans up expired/invalid bans.
pub fn is_banned(store: &impl KeyValueStore, site_id: &str, ip: &str) -> bool {
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
            // Keep index clean when we delete here.
            remove_from_ban_index(store, site_id, ip);
        }
        Ok(None) => {}
        Err(_) => {}
    }
    false
}

/// Bans an IP for a given site, reason, and duration (in seconds).
/// Stores the ban entry in the key-value store.
#[allow(dead_code)]
pub fn ban_ip(store: &impl KeyValueStore, site_id: &str, ip: &str, reason: &str, duration_secs: u64) {
    ban_ip_with_fingerprint(store, site_id, ip, reason, duration_secs, None);
}

pub fn ban_ip_with_fingerprint(
    store: &impl KeyValueStore,
    site_id: &str,
    ip: &str,
    reason: &str,
    duration_secs: u64,
    fingerprint: Option<BanFingerprint>,
) {
    let key = format!("ban:{}:{}", site_id, ip);
    let ts = now_ts();
    let entry = BanEntry {
        reason: reason.to_string(),
        expires: ts + duration_secs,
        banned_at: ts,
        fingerprint,
    };
    if let Ok(val) = serde_json::to_vec(&entry) {
        let _ = store.set(&key, &val);
        add_to_ban_index(store, site_id, ip);
        // log: ban_add
    }
}

/// Unbans an IP for a given site by removing its ban entry from the key-value store.
pub fn unban_ip(store: &impl KeyValueStore, site_id: &str, ip: &str) {
    let key = format!("ban:{}:{}", site_id, ip);
    let _ = store.delete(&key);
    remove_from_ban_index(store, site_id, ip);
}

/// Returns the current UNIX timestamp in seconds (used for ban expiry).
fn now_ts() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}
