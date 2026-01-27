// src/config.rs
// Configuration and site settings for WASM Bot Trap
// Loads and manages per-site configuration (ban duration, rate limits, honeypots, etc.)

use spin_sdk::key_value::Store;

use serde::{Serialize, Deserialize};

/// Configuration struct for a site, loaded from KV or defaults.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub ban_duration: u64,
    pub rate_limit: u32,
    pub honeypots: Vec<String>,
    pub browser_block: Vec<(String, u32)>,
    pub geo_risk: Vec<String>,
    pub whitelist: Vec<String>,
    pub path_whitelist: Vec<String>,
    pub test_mode: bool,
}

impl Config {
    /// Loads config for a site from the key-value store, or returns defaults if not set.
    pub fn load(store: &Store, site_id: &str) -> Self {
        let key = format!("config:{}", site_id);
        if let Ok(Some(val)) = store.get(&key) {
            if let Ok(mut cfg) = serde_json::from_slice::<Config>(&val) {
                // Allow override from env for test_mode
                if let Ok(val) = std::env::var("TEST_MODE") {
                    cfg.test_mode = val == "1" || val.eq_ignore_ascii_case("true");
                }
                return cfg;
            }
        }
        // Defaults for all config fields
        let test_mode = std::env::var("TEST_MODE").map(|v| v == "1" || v.eq_ignore_ascii_case("true")).unwrap_or(false);
        Config {
            ban_duration: 21600, // 6 hours
            rate_limit: 80,
            honeypots: vec!["/bot-trap".to_string()],
            browser_block: vec![("Chrome".to_string(), 120), ("Firefox".to_string(), 115), ("Safari".to_string(), 15)],
            geo_risk: vec![],
            whitelist: vec![],
            path_whitelist: vec![],
            test_mode,
        }
    }
}
