// src/config.rs
// Configuration and site settings for WASM Bot Trap
// Loads and manages per-site configuration (ban duration, rate limits, honeypots, etc.)

use std::env;

use serde::{Serialize, Deserialize};
use crate::challenge::KeyValueStore;

/// Weighted signal contributions for the unified botness score.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BotnessWeights {
    #[serde(default = "default_botness_weight_js_required")]
    pub js_required: u8,
    #[serde(default = "default_botness_weight_geo_risk")]
    pub geo_risk: u8,
    #[serde(default = "default_botness_weight_rate_medium")]
    pub rate_medium: u8,
    #[serde(default = "default_botness_weight_rate_high")]
    pub rate_high: u8,
}

impl Default for BotnessWeights {
    fn default() -> Self {
        BotnessWeights {
            js_required: default_botness_weight_js_required(),
            geo_risk: default_botness_weight_geo_risk(),
            rate_medium: default_botness_weight_rate_medium(),
            rate_high: default_botness_weight_rate_high(),
        }
    }
}

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
    #[serde(default = "default_challenge_threshold")]
    pub challenge_risk_threshold: u8,   // Risk score threshold for serving challenges
    #[serde(default = "default_maze_threshold")]
    pub botness_maze_threshold: u8,     // Risk score threshold for sending likely bots to maze
    #[serde(default)]
    pub botness_weights: BotnessWeights, // Signal weights for unified botness scoring
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
const CHALLENGE_THRESHOLD_MIN: u8 = 1;
const CHALLENGE_THRESHOLD_MAX: u8 = 10;
const CHALLENGE_THRESHOLD_DEFAULT: u8 = 3;
const MAZE_THRESHOLD_MIN: u8 = 1;
const MAZE_THRESHOLD_MAX: u8 = 10;
const MAZE_THRESHOLD_DEFAULT: u8 = 6;
const BOTNESS_WEIGHT_MIN: u8 = 0;
const BOTNESS_WEIGHT_MAX: u8 = 10;
const BOTNESS_WEIGHT_JS_REQUIRED_DEFAULT: u8 = 1;
const BOTNESS_WEIGHT_GEO_RISK_DEFAULT: u8 = 2;
const BOTNESS_WEIGHT_RATE_MEDIUM_DEFAULT: u8 = 1;
const BOTNESS_WEIGHT_RATE_HIGH_DEFAULT: u8 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigMode {
    Hybrid,
    EnvOnly,
}

pub(crate) fn parse_config_mode(value: Option<&str>) -> ConfigMode {
    match value.map(|v| v.trim().to_ascii_lowercase()).as_deref() {
        Some("env_only") => ConfigMode::EnvOnly,
        _ => ConfigMode::Hybrid,
    }
}

pub fn config_mode() -> ConfigMode {
    parse_config_mode(env::var("SHUMA_CONFIG_MODE").ok().as_deref())
}

pub fn config_mode_label() -> &'static str {
    match config_mode() {
        ConfigMode::Hybrid => "hybrid",
        ConfigMode::EnvOnly => "env_only",
    }
}

fn parse_bool_like(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn parse_bool_env(value: Option<&str>) -> bool {
    value.and_then(parse_bool_like).unwrap_or(false)
}

fn parse_bool_env_var(name: &str) -> Option<bool> {
    env::var(name).ok().and_then(|v| parse_bool_like(v.as_str()))
}

fn parse_u64_env_var(name: &str) -> Option<u64> {
    env::var(name).ok().and_then(|v| v.trim().parse::<u64>().ok())
}

fn parse_u32_env_var(name: &str) -> Option<u32> {
    env::var(name).ok().and_then(|v| v.trim().parse::<u32>().ok())
}

fn parse_u8_env_var(name: &str) -> Option<u8> {
    env::var(name).ok().and_then(|v| v.trim().parse::<u8>().ok())
}

fn parse_f32_env_var(name: &str) -> Option<f32> {
    env::var(name).ok().and_then(|v| v.trim().parse::<f32>().ok())
}

fn parse_string_list_value(raw: &str) -> Option<Vec<String>> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Some(Vec::new());
    }
    if let Ok(v) = serde_json::from_str::<Vec<String>>(trimmed) {
        return Some(v.into_iter().map(|item| item.trim().to_string()).filter(|item| !item.is_empty()).collect());
    }
    Some(
        trimmed
            .split(',')
            .map(|item| item.trim().to_string())
            .filter(|item| !item.is_empty())
            .collect(),
    )
}

fn parse_string_list_env_var(name: &str) -> Option<Vec<String>> {
    env::var(name).ok().and_then(|v| parse_string_list_value(v.as_str()))
}

fn parse_browser_rules_value(raw: &str) -> Option<Vec<(String, u32)>> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Some(Vec::new());
    }
    if let Ok(v) = serde_json::from_str::<Vec<(String, u32)>>(trimmed) {
        return Some(v.into_iter().filter(|(name, _)| !name.trim().is_empty()).collect());
    }
    let mut parsed = Vec::new();
    for entry in trimmed.split(',') {
        let item = entry.trim();
        if item.is_empty() {
            continue;
        }
        let (name, version) = item.split_once(':')?;
        let name = name.trim();
        if name.is_empty() {
            return None;
        }
        let version = version.trim().parse::<u32>().ok()?;
        parsed.push((name.to_string(), version));
    }
    Some(parsed)
}

fn parse_browser_rules_env_var(name: &str) -> Option<Vec<(String, u32)>> {
    env::var(name).ok().and_then(|v| parse_browser_rules_value(v.as_str()))
}

pub fn pow_config_mutable() -> bool {
    env::var("SHUMA_POW_CONFIG_MUTABLE")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

pub(crate) fn challenge_config_mutable_from_env(value: Option<&str>) -> bool {
    parse_bool_env(value)
}

pub fn challenge_config_mutable() -> bool {
    challenge_config_mutable_from_env(env::var("SHUMA_CHALLENGE_CONFIG_MUTABLE").ok().as_deref())
}

pub fn botness_config_mutable() -> bool {
    match env::var("SHUMA_BOTNESS_CONFIG_MUTABLE") {
        Ok(v) => parse_bool_env(Some(v.as_str())),
        Err(_) => challenge_config_mutable(),
    }
}

fn clamp_pow_difficulty(value: u8) -> u8 {
    value.clamp(POW_DIFFICULTY_MIN, POW_DIFFICULTY_MAX)
}

fn clamp_pow_ttl(value: u64) -> u64 {
    value.clamp(POW_TTL_MIN, POW_TTL_MAX)
}

fn clamp_challenge_threshold(value: u8) -> u8 {
    value.clamp(CHALLENGE_THRESHOLD_MIN, CHALLENGE_THRESHOLD_MAX)
}

fn clamp_maze_threshold(value: u8) -> u8 {
    value.clamp(MAZE_THRESHOLD_MIN, MAZE_THRESHOLD_MAX)
}

fn clamp_botness_weight(value: u8) -> u8 {
    value.clamp(BOTNESS_WEIGHT_MIN, BOTNESS_WEIGHT_MAX)
}

pub(crate) fn parse_challenge_threshold(value: Option<&str>) -> u8 {
    let parsed = value.and_then(|v| v.parse::<u8>().ok()).unwrap_or(CHALLENGE_THRESHOLD_DEFAULT);
    clamp_challenge_threshold(parsed)
}

pub(crate) fn parse_maze_threshold(value: Option<&str>) -> u8 {
    let parsed = value.and_then(|v| v.parse::<u8>().ok()).unwrap_or(MAZE_THRESHOLD_DEFAULT);
    clamp_maze_threshold(parsed)
}

pub(crate) fn parse_botness_weight(value: Option<&str>, default_value: u8) -> u8 {
    let parsed = value.and_then(|v| v.parse::<u8>().ok()).unwrap_or(default_value);
    clamp_botness_weight(parsed)
}

fn default_pow_difficulty() -> u8 {
    let v = env::var("SHUMA_POW_DIFFICULTY")
        .ok()
        .and_then(|val| val.parse::<u8>().ok())
        .unwrap_or(15);
    clamp_pow_difficulty(v)
}

fn default_pow_ttl_seconds() -> u64 {
    let v = env::var("SHUMA_POW_TTL_SECONDS")
        .ok()
        .and_then(|val| val.parse::<u64>().ok())
        .unwrap_or(90);
    clamp_pow_ttl(v)
}

fn default_challenge_threshold() -> u8 {
    parse_challenge_threshold(env::var("SHUMA_CHALLENGE_RISK_THRESHOLD").ok().as_deref())
}

fn default_maze_threshold() -> u8 {
    parse_maze_threshold(env::var("SHUMA_BOTNESS_MAZE_THRESHOLD").ok().as_deref())
}

fn default_botness_weight_js_required() -> u8 {
    parse_botness_weight(
        env::var("SHUMA_BOTNESS_WEIGHT_JS_REQUIRED").ok().as_deref(),
        BOTNESS_WEIGHT_JS_REQUIRED_DEFAULT,
    )
}

fn default_botness_weight_geo_risk() -> u8 {
    parse_botness_weight(
        env::var("SHUMA_BOTNESS_WEIGHT_GEO_RISK").ok().as_deref(),
        BOTNESS_WEIGHT_GEO_RISK_DEFAULT,
    )
}

fn default_botness_weight_rate_medium() -> u8 {
    parse_botness_weight(
        env::var("SHUMA_BOTNESS_WEIGHT_RATE_MEDIUM").ok().as_deref(),
        BOTNESS_WEIGHT_RATE_MEDIUM_DEFAULT,
    )
}

fn default_botness_weight_rate_high() -> u8 {
    parse_botness_weight(
        env::var("SHUMA_BOTNESS_WEIGHT_RATE_HIGH").ok().as_deref(),
        BOTNESS_WEIGHT_RATE_HIGH_DEFAULT,
    )
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

fn default_config() -> Config {
    Config {
        ban_duration: 21600, // 6 hours (legacy default)
        ban_durations: BanDurations::default(),
        rate_limit: 80,
        honeypots: vec!["/bot-trap".to_string()],
        browser_block: vec![
            ("Chrome".to_string(), 120),
            ("Firefox".to_string(), 115),
            ("Safari".to_string(), 15),
        ],
        browser_whitelist: vec![],
        geo_risk: vec![],
        whitelist: vec![],
        path_whitelist: vec![],
        test_mode: false,
        maze_enabled: true,
        maze_auto_ban: true,
        maze_auto_ban_threshold: 50,
        robots_enabled: true,
        robots_block_ai_training: true,
        robots_block_ai_search: false,
        robots_allow_search_engines: true,
        robots_crawl_delay: 2,
        cdp_detection_enabled: true,
        cdp_auto_ban: true,
        cdp_detection_threshold: 0.8,
        pow_difficulty: default_pow_difficulty(),
        pow_ttl_seconds: default_pow_ttl_seconds(),
        challenge_risk_threshold: default_challenge_threshold(),
        botness_maze_threshold: default_maze_threshold(),
        botness_weights: BotnessWeights::default(),
    }
}

fn apply_env_overrides(cfg: &mut Config) {
    if let Some(v) = parse_bool_env_var("SHUMA_TEST_MODE") {
        cfg.test_mode = v;
    }
    if let Some(v) = parse_u64_env_var("SHUMA_BAN_DURATION") {
        cfg.ban_duration = v;
    }
    if let Some(v) = parse_u64_env_var("SHUMA_BAN_DURATION_HONEYPOT") {
        cfg.ban_durations.honeypot = v;
    }
    if let Some(v) = parse_u64_env_var("SHUMA_BAN_DURATION_RATE_LIMIT") {
        cfg.ban_durations.rate_limit = v;
    }
    if let Some(v) = parse_u64_env_var("SHUMA_BAN_DURATION_BROWSER") {
        cfg.ban_durations.browser = v;
    }
    if let Some(v) = parse_u64_env_var("SHUMA_BAN_DURATION_ADMIN") {
        cfg.ban_durations.admin = v;
    }
    if let Some(v) = parse_u64_env_var("SHUMA_BAN_DURATION_CDP") {
        cfg.ban_durations.cdp = v;
    }
    if let Some(v) = parse_u32_env_var("SHUMA_RATE_LIMIT") {
        cfg.rate_limit = v;
    }
    if let Some(v) = parse_string_list_env_var("SHUMA_HONEYPOTS") {
        cfg.honeypots = v;
    }
    if let Some(v) = parse_browser_rules_env_var("SHUMA_BROWSER_BLOCK") {
        cfg.browser_block = v;
    }
    if let Some(v) = parse_browser_rules_env_var("SHUMA_BROWSER_WHITELIST") {
        cfg.browser_whitelist = v;
    }
    if let Some(v) = parse_string_list_env_var("SHUMA_GEO_RISK") {
        cfg.geo_risk = v;
    }
    if let Some(v) = parse_string_list_env_var("SHUMA_WHITELIST") {
        cfg.whitelist = v;
    }
    if let Some(v) = parse_string_list_env_var("SHUMA_PATH_WHITELIST") {
        cfg.path_whitelist = v;
    }
    if let Some(v) = parse_bool_env_var("SHUMA_MAZE_ENABLED") {
        cfg.maze_enabled = v;
    }
    if let Some(v) = parse_bool_env_var("SHUMA_MAZE_AUTO_BAN") {
        cfg.maze_auto_ban = v;
    }
    if let Some(v) = parse_u32_env_var("SHUMA_MAZE_AUTO_BAN_THRESHOLD") {
        cfg.maze_auto_ban_threshold = v;
    }
    if let Some(v) = parse_bool_env_var("SHUMA_ROBOTS_ENABLED") {
        cfg.robots_enabled = v;
    }
    if let Some(v) = parse_bool_env_var("SHUMA_ROBOTS_BLOCK_AI_TRAINING") {
        cfg.robots_block_ai_training = v;
    }
    if let Some(v) = parse_bool_env_var("SHUMA_ROBOTS_BLOCK_AI_SEARCH") {
        cfg.robots_block_ai_search = v;
    }
    if let Some(v) = parse_bool_env_var("SHUMA_ROBOTS_ALLOW_SEARCH_ENGINES") {
        cfg.robots_allow_search_engines = v;
    }
    if let Some(v) = parse_u32_env_var("SHUMA_ROBOTS_CRAWL_DELAY") {
        cfg.robots_crawl_delay = v;
    }
    if let Some(v) = parse_bool_env_var("SHUMA_CDP_DETECTION_ENABLED") {
        cfg.cdp_detection_enabled = v;
    }
    if let Some(v) = parse_bool_env_var("SHUMA_CDP_AUTO_BAN") {
        cfg.cdp_auto_ban = v;
    }
    if let Some(v) = parse_f32_env_var("SHUMA_CDP_DETECTION_THRESHOLD") {
        cfg.cdp_detection_threshold = v.clamp(0.0, 1.0);
    }
    if let Some(v) = parse_u8_env_var("SHUMA_POW_DIFFICULTY") {
        cfg.pow_difficulty = clamp_pow_difficulty(v);
    }
    if let Some(v) = parse_u64_env_var("SHUMA_POW_TTL_SECONDS") {
        cfg.pow_ttl_seconds = clamp_pow_ttl(v);
    }
    if let Some(v) = parse_u8_env_var("SHUMA_CHALLENGE_RISK_THRESHOLD") {
        cfg.challenge_risk_threshold = clamp_challenge_threshold(v);
    }
    if let Some(v) = parse_u8_env_var("SHUMA_BOTNESS_MAZE_THRESHOLD") {
        cfg.botness_maze_threshold = clamp_maze_threshold(v);
    }
    if let Some(v) = parse_u8_env_var("SHUMA_BOTNESS_WEIGHT_JS_REQUIRED") {
        cfg.botness_weights.js_required = clamp_botness_weight(v);
    }
    if let Some(v) = parse_u8_env_var("SHUMA_BOTNESS_WEIGHT_GEO_RISK") {
        cfg.botness_weights.geo_risk = clamp_botness_weight(v);
    }
    if let Some(v) = parse_u8_env_var("SHUMA_BOTNESS_WEIGHT_RATE_MEDIUM") {
        cfg.botness_weights.rate_medium = clamp_botness_weight(v);
    }
    if let Some(v) = parse_u8_env_var("SHUMA_BOTNESS_WEIGHT_RATE_HIGH") {
        cfg.botness_weights.rate_high = clamp_botness_weight(v);
    }
}

fn apply_mutability_policy(cfg: &mut Config) {
    if !pow_config_mutable() {
        cfg.pow_difficulty = default_pow_difficulty();
        cfg.pow_ttl_seconds = default_pow_ttl_seconds();
    }
    cfg.pow_difficulty = clamp_pow_difficulty(cfg.pow_difficulty);
    cfg.pow_ttl_seconds = clamp_pow_ttl(cfg.pow_ttl_seconds);

    if !botness_config_mutable() {
        cfg.challenge_risk_threshold = default_challenge_threshold();
        cfg.botness_maze_threshold = default_maze_threshold();
        cfg.botness_weights = BotnessWeights::default();
    } else {
        cfg.challenge_risk_threshold = clamp_challenge_threshold(cfg.challenge_risk_threshold);
        cfg.botness_maze_threshold = clamp_maze_threshold(cfg.botness_maze_threshold);
        cfg.botness_weights.js_required = clamp_botness_weight(cfg.botness_weights.js_required);
        cfg.botness_weights.geo_risk = clamp_botness_weight(cfg.botness_weights.geo_risk);
        cfg.botness_weights.rate_medium = clamp_botness_weight(cfg.botness_weights.rate_medium);
        cfg.botness_weights.rate_high = clamp_botness_weight(cfg.botness_weights.rate_high);
    }
}

impl Config {
    /// Loads config for a site from the key-value store, or returns defaults if not set.
    pub fn load(store: &impl KeyValueStore, site_id: &str) -> Self {
        let mut cfg = match config_mode() {
            ConfigMode::EnvOnly => default_config(),
            ConfigMode::Hybrid => {
                let key = format!("config:{}", site_id);
                if let Ok(Some(val)) = store.get(&key) {
                    if let Ok(cfg) = serde_json::from_slice::<Config>(&val) {
                        cfg
                    } else {
                        default_config()
                    }
                } else {
                    default_config()
                }
            }
        };

        apply_env_overrides(&mut cfg);
        apply_mutability_policy(&mut cfg);
        cfg
    }
    
    /// Get ban duration for a specific ban type
    pub fn get_ban_duration(&self, ban_type: &str) -> u64 {
        self.ban_durations.get(ban_type)
    }
}
