// src/rate.rs
// Rate limiting for WASM Bot Trap

use spin_sdk::key_value::Store;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn check_rate_limit(store: &Store, site_id: &str, ip: &str, limit: u32) -> bool {
    let key = format!("rate:{}:{}", site_id, ip);
    let now = now_ts();
    let window = now / 60; // 1-minute window
    let window_key = format!("{}:{}", key, window);
    let count = store.get(&window_key).ok().flatten().and_then(|v| String::from_utf8(v).ok()).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
    if count >= limit {
        return false;
    }
    let _ = store.set(&window_key, (count + 1).to_string().as_bytes());
    true
}

fn now_ts() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}
