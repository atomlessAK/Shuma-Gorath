// src/config.rs
// Configuration and site settings for WASM Bot Trap
// Loads and manages per-site configuration (ban duration, rate limits, honeypots, etc.)

use spin_sdk::key_value::Store;
use std::env;

use serde::{Serialize, Deserialize};

/// Ban duration settings per ban type (in seconds)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BanDurations {
    pub honeypot: u64,      // Accessing honeypot URLs
    pub rate_limit: u64,    // Exceeding rate limits
    pub browser: u64,       // Outdated/suspicious browser
    pub admin: u64,         // Manual admin ban (default)
    #[serde(default = "default_cdp_ban_duration")]
    pub cdp: u64,           // CDP automation detection
}

fn default_cdp_ban_duration() -> u64 {
    43200  // 12 hours for CDP automation
}

impl Default for BanDurations {
    fn default() -> Self {
        BanDurations {
            honeypot: 86400,    // 24 hours - severe offense
            rate_limit: 3600,   // 1 hour - temporary
            browser: 21600,     // 6 hours - moderate
            admin: 21600,       // 6 hours - default for manual bans
            cdp: 43200,         // 12 hours - automation detected
        }
    }
}

impl BanDurations {
    /// Get duration for a specific ban type, with fallback to admin duration
    pub fn get(&self, ban_type: &str) -> u64 {
        match ban_type {
            "honeypot" => self.honeypot,
            "rate" | "rate_limit" => self.rate_limit,
            "browser" => self.browser,
            "cdp" | "cdp_automation" => self.cdp,
            _ => self.admin,
        }
    }
}

/// Configuration struct for a site, loaded from KV or defaults.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub ban_duration: u64,           // Legacy: single duration (kept for backward compatibility)
    pub ban_durations: BanDurations, // New: per-type durations
    pub rate_limit: u32,
    pub honeypots: Vec<String>,
    pub browser_block: Vec<(String, u32)>,
    pub browser_whitelist: Vec<(String, u32)>,
    pub geo_risk: Vec<String>,
    pub whitelist: Vec<String>,
    pub path_whitelist: Vec<String>,
    pub test_mode: bool,
    #[serde(default)]
    pub maze_enabled: bool,          // Enable link maze honeypot
    #[serde(default = "default_maze_auto_ban")]
    pub maze_auto_ban: bool,         // Auto-ban after threshold maze page hits
    #[serde(default = "default_maze_auto_ban_threshold")]
    pub maze_auto_ban_threshold: u32, // Number of maze pages before auto-ban
    
    // robots.txt configuration
    #[serde(default = "default_true")]
    pub robots_enabled: bool,           // Serve /robots.txt endpoint
    #[serde(default = "default_true")]
    pub robots_block_ai_training: bool, // Block AI training crawlers (GPTBot, CCBot, etc.)
    #[serde(default)]
    pub robots_block_ai_search: bool,   // Block AI search assistants (PerplexityBot, etc.)
    #[serde(default = "default_true")]
    pub robots_allow_search_engines: bool, // Allow legitimate search engines (Google, Bing, etc.)
    #[serde(default = "default_crawl_delay")]
    pub robots_crawl_delay: u32,        // Crawl-delay directive (seconds)
    
    // CDP (Chrome DevTools Protocol) detection configuration
    #[serde(default = "default_true")]
    pub cdp_detection_enabled: bool,     // Enable CDP automation detection
    #[serde(default = "default_true")]
    pub cdp_auto_ban: bool,              // Auto-ban when CDP automation detected
    #[serde(default = "default_cdp_threshold")]
    pub cdp_detection_threshold: f32,    // Score threshold for detection (0.0-1.0)

    #[serde(default = "default_pow_difficulty")]
    pub pow_difficulty: u8,             // PoW leading-zero bits
    #[serde(default = "default_pow_ttl_seconds")]
    pub pow_ttl_seconds: u64,           // PoW seed expiry in seconds
}

fn default_true() -> bool {
    true
}

fn default_cdp_threshold() -> f32 {
    0.8  // Default: 80% confidence required for detection
}

pub const POW_DIFFICULTY_MIN: u8 = 12;
pub const POW_DIFFICULTY_MAX: u8 = 20;
pub const POW_TTL_MIN: u64 = 30;
pub const POW_TTL_MAX: u64 = 300;

pub fn pow_config_mutable() -> bool {
    env::var("POW_CONFIG_MUTABLE")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn clamp_pow_difficulty(value: u8) -> u8 {
    value.clamp(POW_DIFFICULTY_MIN, POW_DIFFICULTY_MAX)
}

fn clamp_pow_ttl(value: u64) -> u64 {
    value.clamp(POW_TTL_MIN, POW_TTL_MAX)
}

fn default_pow_difficulty() -> u8 {
    let v = env::var("POW_DIFFICULTY")
        .ok()
        .and_then(|val| val.parse::<u8>().ok())
        .unwrap_or(15);
    clamp_pow_difficulty(v)
}

fn default_pow_ttl_seconds() -> u64 {
    let v = env::var("POW_TTL_SECONDS")
        .ok()
        .and_then(|val| val.parse::<u64>().ok())
        .unwrap_or(90);
    clamp_pow_ttl(v)
}

fn default_maze_auto_ban() -> bool {
    true
}

fn default_maze_auto_ban_threshold() -> u32 {
    50
}

fn default_crawl_delay() -> u32 {
    2
}

impl Config {
    /// Loads config for a site from the key-value store, or returns defaults if not set.
    pub fn load(store: &Store, site_id: &str) -> Self {
        let key = format!("config:{}", site_id);
        if let Ok(Some(val)) = store.get(&key) {
            if let Ok(mut cfg) = serde_json::from_slice::<Config>(&val) {
                // Allow override from env for test_mode
                if let Ok(val) = env::var("TEST_MODE") {
                    cfg.test_mode = val == "1" || val.eq_ignore_ascii_case("true");
                }
                if !pow_config_mutable() {
                    cfg.pow_difficulty = default_pow_difficulty();
                    cfg.pow_ttl_seconds = default_pow_ttl_seconds();
                }
                cfg.pow_difficulty = clamp_pow_difficulty(cfg.pow_difficulty);
                cfg.pow_ttl_seconds = clamp_pow_ttl(cfg.pow_ttl_seconds);
                return cfg;
            }
        }
        // Defaults for all config fields
        let test_mode = env::var("TEST_MODE").map(|v| v == "1" || v.eq_ignore_ascii_case("true")).unwrap_or(false);
        Config {
            ban_duration: 21600, // 6 hours (legacy default)
            ban_durations: BanDurations::default(),
            rate_limit: 80,
            honeypots: vec!["/bot-trap".to_string()],
            browser_block: vec![("Chrome".to_string(), 120), ("Firefox".to_string(), 115), ("Safari".to_string(), 15)],
            browser_whitelist: vec![],
            geo_risk: vec![],
            whitelist: vec![],
            path_whitelist: vec![],
            test_mode,
            maze_enabled: true,        // Maze enabled by default
            maze_auto_ban: true,       // Auto-ban crawlers after threshold
            maze_auto_ban_threshold: 50, // Default: 50 maze pages triggers ban
            // robots.txt defaults: block AI training, allow search engines
            robots_enabled: true,
            robots_block_ai_training: true,
            robots_block_ai_search: false,
            robots_allow_search_engines: true,
            robots_crawl_delay: 2,
            // CDP detection defaults: enabled with auto-ban
            cdp_detection_enabled: true,
            cdp_auto_ban: true,
            cdp_detection_threshold: 0.8,
            pow_difficulty: default_pow_difficulty(),
            pow_ttl_seconds: default_pow_ttl_seconds(),
        }
    }
    
    /// Get ban duration for a specific ban type
    pub fn get_ban_duration(&self, ban_type: &str) -> u64 {
        self.ban_durations.get(ban_type)
    }
}
