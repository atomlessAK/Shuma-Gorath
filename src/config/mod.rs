// src/config.rs
// Configuration and site settings for WASM Bot Defence.
// Tunables are loaded from KV; defaults are defined in config/defaults.env.

#[cfg(not(test))]
use std::time::{SystemTime, UNIX_EPOCH};
use std::{collections::HashMap, env, sync::Mutex};

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::challenge::KeyValueStore;

const DEFAULTS_ENV_TEXT: &str = include_str!("../../config/defaults.env");

pub const POW_DIFFICULTY_MIN: u8 = 12;
pub const POW_DIFFICULTY_MAX: u8 = 20;
pub const POW_TTL_MIN: u64 = 30;
pub const POW_TTL_MAX: u64 = 300;
const CHALLENGE_THRESHOLD_MIN: u8 = 1;
const CHALLENGE_THRESHOLD_MAX: u8 = 10;
const MAZE_THRESHOLD_MIN: u8 = 1;
const MAZE_THRESHOLD_MAX: u8 = 10;
const BOTNESS_WEIGHT_MIN: u8 = 0;
const BOTNESS_WEIGHT_MAX: u8 = 10;
const CHALLENGE_TRANSFORM_COUNT_MIN: u8 = 4;
const CHALLENGE_TRANSFORM_COUNT_MAX: u8 = 8;
#[cfg(not(test))]
const CONFIG_CACHE_TTL_SECONDS: u64 = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigLoadError {
    StoreUnavailable,
    MissingConfig,
    InvalidConfig,
}

impl ConfigLoadError {
    pub fn user_message(&self) -> &'static str {
        match self {
            ConfigLoadError::StoreUnavailable => "Configuration unavailable (KV store error)",
            ConfigLoadError::MissingConfig => {
                "Configuration unavailable (missing KV config; run setup/config-seed)"
            }
            ConfigLoadError::InvalidConfig => "Configuration unavailable (invalid KV config)",
        }
    }
}

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
    #[serde(default = "default_ban_duration_honeypot")]
    pub honeypot: u64,
    #[serde(default = "default_ban_duration_rate_limit")]
    pub rate_limit: u64,
    #[serde(default = "default_ban_duration_browser")]
    pub browser: u64,
    #[serde(default = "default_ban_duration_admin")]
    pub admin: u64,
    #[serde(default = "default_ban_duration_cdp")]
    pub cdp: u64,
}

impl Default for BanDurations {
    fn default() -> Self {
        BanDurations {
            honeypot: default_ban_duration_honeypot(),
            rate_limit: default_ban_duration_rate_limit(),
            browser: default_ban_duration_browser(),
            admin: default_ban_duration_admin(),
            cdp: default_ban_duration_cdp(),
        }
    }
}

impl BanDurations {
    /// Get duration for a specific ban type, with fallback to admin duration.
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

/// Configuration struct for a site, loaded from KV.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    #[serde(default = "default_ban_duration")]
    pub ban_duration: u64,
    #[serde(default)]
    pub ban_durations: BanDurations,
    #[serde(default = "default_rate_limit")]
    pub rate_limit: u32,
    #[serde(default = "default_honeypots")]
    pub honeypots: Vec<String>,
    #[serde(default = "default_browser_block")]
    pub browser_block: Vec<(String, u32)>,
    #[serde(default = "default_browser_whitelist")]
    pub browser_whitelist: Vec<(String, u32)>,
    #[serde(default = "default_geo_risk")]
    pub geo_risk: Vec<String>,
    #[serde(default = "default_geo_allow")]
    pub geo_allow: Vec<String>,
    #[serde(default = "default_geo_challenge")]
    pub geo_challenge: Vec<String>,
    #[serde(default = "default_geo_maze")]
    pub geo_maze: Vec<String>,
    #[serde(default = "default_geo_block")]
    pub geo_block: Vec<String>,
    #[serde(default = "default_whitelist")]
    pub whitelist: Vec<String>,
    #[serde(default = "default_path_whitelist")]
    pub path_whitelist: Vec<String>,
    #[serde(default = "default_test_mode")]
    pub test_mode: bool,
    #[serde(default = "default_maze_enabled")]
    pub maze_enabled: bool,
    #[serde(default = "default_maze_auto_ban")]
    pub maze_auto_ban: bool,
    #[serde(default = "default_maze_auto_ban_threshold")]
    pub maze_auto_ban_threshold: u32,
    #[serde(default = "default_robots_enabled")]
    pub robots_enabled: bool,
    #[serde(default = "default_robots_block_ai_training")]
    pub robots_block_ai_training: bool,
    #[serde(default = "default_robots_block_ai_search")]
    pub robots_block_ai_search: bool,
    #[serde(default = "default_robots_allow_search_engines")]
    pub robots_allow_search_engines: bool,
    #[serde(default = "default_robots_crawl_delay")]
    pub robots_crawl_delay: u32,
    #[serde(default = "default_cdp_detection_enabled")]
    pub cdp_detection_enabled: bool,
    #[serde(default = "default_cdp_auto_ban")]
    pub cdp_auto_ban: bool,
    #[serde(default = "default_cdp_threshold")]
    pub cdp_detection_threshold: f32,
    #[serde(default = "default_js_required_enforced")]
    pub js_required_enforced: bool,
    #[serde(default = "default_pow_enabled")]
    pub pow_enabled: bool,
    #[serde(default = "default_pow_difficulty")]
    pub pow_difficulty: u8,
    #[serde(default = "default_pow_ttl_seconds")]
    pub pow_ttl_seconds: u64,
    #[serde(default = "default_challenge_transform_count")]
    pub challenge_transform_count: u8,
    #[serde(default = "default_challenge_threshold")]
    pub challenge_risk_threshold: u8,
    #[serde(default = "default_maze_threshold")]
    pub botness_maze_threshold: u8,
    #[serde(default)]
    pub botness_weights: BotnessWeights,
}

#[derive(Debug, Clone)]
struct CachedConfig {
    loaded_at: u64,
    config: Config,
}

impl Config {
    /// Loads config for a site from KV only.
    pub fn load(store: &impl KeyValueStore, site_id: &str) -> Result<Self, ConfigLoadError> {
        let key = format!("config:{}", site_id);
        let val = store
            .get(&key)
            .map_err(|_| ConfigLoadError::StoreUnavailable)?
            .ok_or(ConfigLoadError::MissingConfig)?;

        let mut cfg =
            serde_json::from_slice::<Config>(&val).map_err(|_| ConfigLoadError::InvalidConfig)?;
        clamp_config_values(&mut cfg);
        Ok(cfg)
    }

    /// Returns ban duration for a specific ban type.
    pub fn get_ban_duration(&self, ban_type: &str) -> u64 {
        self.ban_durations.get(ban_type)
    }
}

static RUNTIME_CONFIG_CACHE: Lazy<Mutex<HashMap<String, CachedConfig>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[cfg(not(test))]
fn now_ts() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn load_cached_with_now(
    store: &impl KeyValueStore,
    site_id: &str,
    now: u64,
    ttl_seconds: u64,
) -> Result<Config, ConfigLoadError> {
    {
        let cache = RUNTIME_CONFIG_CACHE.lock().unwrap();
        if let Some(entry) = cache.get(site_id) {
            let age = now.saturating_sub(entry.loaded_at);
            if age <= ttl_seconds {
                return Ok(entry.config.clone());
            }
        }
    }

    let config = Config::load(store, site_id)?;
    let mut cache = RUNTIME_CONFIG_CACHE.lock().unwrap();
    cache.insert(
        site_id.to_string(),
        CachedConfig {
            loaded_at: now,
            config: config.clone(),
        },
    );
    Ok(config)
}

pub fn load_runtime_cached(
    store: &impl KeyValueStore,
    site_id: &str,
) -> Result<Config, ConfigLoadError> {
    #[cfg(test)]
    {
        return Config::load(store, site_id);
    }
    #[cfg(not(test))]
    {
        load_cached_with_now(store, site_id, now_ts(), CONFIG_CACHE_TTL_SECONDS)
    }
}

pub fn invalidate_runtime_cache(site_id: &str) {
    let mut cache = RUNTIME_CONFIG_CACHE.lock().unwrap();
    cache.remove(site_id);
}

#[cfg(test)]
pub(crate) fn clear_runtime_cache_for_tests() {
    let mut cache = RUNTIME_CONFIG_CACHE.lock().unwrap();
    cache.clear();
}

#[cfg(test)]
pub(crate) fn load_runtime_cached_for_tests(
    store: &impl KeyValueStore,
    site_id: &str,
    now: u64,
    ttl_seconds: u64,
) -> Result<Config, ConfigLoadError> {
    load_cached_with_now(store, site_id, now, ttl_seconds)
}

static DEFAULTS_MAP: Lazy<Result<HashMap<String, String>, String>> =
    Lazy::new(|| parse_defaults_env_map(DEFAULTS_ENV_TEXT));

static DEFAULT_CONFIG: Lazy<Config> = Lazy::new(|| {
    let mut cfg = Config {
        ban_duration: defaults_u64("SHUMA_BAN_DURATION"),
        ban_durations: BanDurations {
            honeypot: defaults_u64("SHUMA_BAN_DURATION_HONEYPOT"),
            rate_limit: defaults_u64("SHUMA_BAN_DURATION_RATE_LIMIT"),
            browser: defaults_u64("SHUMA_BAN_DURATION_BROWSER"),
            admin: defaults_u64("SHUMA_BAN_DURATION_ADMIN"),
            cdp: defaults_u64("SHUMA_BAN_DURATION_CDP"),
        },
        rate_limit: defaults_u32("SHUMA_RATE_LIMIT"),
        honeypots: defaults_string_list("SHUMA_HONEYPOTS"),
        browser_block: defaults_browser_rules("SHUMA_BROWSER_BLOCK"),
        browser_whitelist: defaults_browser_rules("SHUMA_BROWSER_WHITELIST"),
        geo_risk: defaults_country_list("SHUMA_GEO_RISK_COUNTRIES"),
        geo_allow: defaults_country_list("SHUMA_GEO_ALLOW_COUNTRIES"),
        geo_challenge: defaults_country_list("SHUMA_GEO_CHALLENGE_COUNTRIES"),
        geo_maze: defaults_country_list("SHUMA_GEO_MAZE_COUNTRIES"),
        geo_block: defaults_country_list("SHUMA_GEO_BLOCK_COUNTRIES"),
        whitelist: defaults_string_list("SHUMA_WHITELIST"),
        path_whitelist: defaults_string_list("SHUMA_PATH_WHITELIST"),
        test_mode: defaults_bool("SHUMA_TEST_MODE"),
        maze_enabled: defaults_bool("SHUMA_MAZE_ENABLED"),
        maze_auto_ban: defaults_bool("SHUMA_MAZE_AUTO_BAN"),
        maze_auto_ban_threshold: defaults_u32("SHUMA_MAZE_AUTO_BAN_THRESHOLD"),
        robots_enabled: defaults_bool("SHUMA_ROBOTS_ENABLED"),
        robots_block_ai_training: defaults_bool("SHUMA_ROBOTS_BLOCK_AI_TRAINING"),
        robots_block_ai_search: defaults_bool("SHUMA_ROBOTS_BLOCK_AI_SEARCH"),
        robots_allow_search_engines: defaults_bool("SHUMA_ROBOTS_ALLOW_SEARCH_ENGINES"),
        robots_crawl_delay: defaults_u32("SHUMA_ROBOTS_CRAWL_DELAY"),
        cdp_detection_enabled: defaults_bool("SHUMA_CDP_DETECTION_ENABLED"),
        cdp_auto_ban: defaults_bool("SHUMA_CDP_AUTO_BAN"),
        cdp_detection_threshold: defaults_f32("SHUMA_CDP_DETECTION_THRESHOLD"),
        js_required_enforced: defaults_bool("SHUMA_JS_REQUIRED_ENFORCED"),
        pow_enabled: defaults_bool("SHUMA_POW_ENABLED"),
        pow_difficulty: defaults_u8("SHUMA_POW_DIFFICULTY"),
        pow_ttl_seconds: defaults_u64("SHUMA_POW_TTL_SECONDS"),
        challenge_transform_count: defaults_u8("SHUMA_CHALLENGE_TRANSFORM_COUNT"),
        challenge_risk_threshold: defaults_u8("SHUMA_CHALLENGE_RISK_THRESHOLD"),
        botness_maze_threshold: defaults_u8("SHUMA_BOTNESS_MAZE_THRESHOLD"),
        botness_weights: BotnessWeights {
            js_required: defaults_u8("SHUMA_BOTNESS_WEIGHT_JS_REQUIRED"),
            geo_risk: defaults_u8("SHUMA_BOTNESS_WEIGHT_GEO_RISK"),
            rate_medium: defaults_u8("SHUMA_BOTNESS_WEIGHT_RATE_MEDIUM"),
            rate_high: defaults_u8("SHUMA_BOTNESS_WEIGHT_RATE_HIGH"),
        },
    };
    clamp_config_values(&mut cfg);
    cfg
});

static ENV_VALIDATION_RESULT: Lazy<Result<(), String>> = Lazy::new(validate_env_only_impl);

pub fn defaults() -> &'static Config {
    &DEFAULT_CONFIG
}

pub fn validate_env_only_once() -> Result<(), String> {
    if cfg!(test) {
        if validate_env_in_tests_enabled() {
            return validate_env_only_impl();
        }
        return Ok(());
    }
    match &*ENV_VALIDATION_RESULT {
        Ok(()) => Ok(()),
        Err(msg) => Err(msg.clone()),
    }
}

fn validate_env_only_impl() -> Result<(), String> {
    validate_non_empty("SHUMA_API_KEY")?;
    validate_non_empty("SHUMA_JS_SECRET")?;
    validate_non_empty("SHUMA_FORWARDED_IP_SECRET")?;
    validate_u64_var("SHUMA_EVENT_LOG_RETENTION_HOURS")?;

    validate_bool_like_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED")?;
    validate_bool_like_var("SHUMA_KV_STORE_FAIL_OPEN")?;
    validate_bool_like_var("SHUMA_ENFORCE_HTTPS")?;
    validate_bool_like_var("SHUMA_DEBUG_HEADERS")?;
    validate_bool_like_var("SHUMA_POW_CONFIG_MUTABLE")?;
    validate_bool_like_var("SHUMA_CHALLENGE_CONFIG_MUTABLE")?;
    validate_bool_like_var("SHUMA_BOTNESS_CONFIG_MUTABLE")?;

    Ok(())
}

fn validate_env_in_tests_enabled() -> bool {
    if !cfg!(test) {
        return false;
    }
    env::var("SHUMA_VALIDATE_ENV_IN_TESTS")
        .ok()
        .and_then(|v| parse_bool_like(v.as_str()))
        .unwrap_or(false)
}

fn validate_non_empty(name: &str) -> Result<(), String> {
    let value = env::var(name).map_err(|_| format!("Missing required env var {}", name))?;
    if value.trim().is_empty() {
        return Err(format!("Invalid empty env var {}", name));
    }
    Ok(())
}

fn validate_bool_like_var(name: &str) -> Result<(), String> {
    let value = env::var(name).map_err(|_| format!("Missing required env var {}", name))?;
    if parse_bool_like(&value).is_none() {
        return Err(format!("Invalid boolean env var {}={}", name, value));
    }
    Ok(())
}

fn validate_u64_var(name: &str) -> Result<(), String> {
    let value = env::var(name).map_err(|_| format!("Missing required env var {}", name))?;
    if value.trim().parse::<u64>().is_err() {
        return Err(format!("Invalid integer env var {}={}", name, value));
    }
    Ok(())
}

#[cfg(test)]
pub(crate) fn parse_admin_config_write_enabled(value: Option<&str>) -> bool {
    value.and_then(parse_bool_like).unwrap_or(false)
}

pub fn admin_config_write_enabled() -> bool {
    env_bool_required("SHUMA_ADMIN_CONFIG_WRITE_ENABLED")
}

pub fn https_enforced() -> bool {
    env_bool_required("SHUMA_ENFORCE_HTTPS")
}

pub fn forwarded_header_trust_configured() -> bool {
    env::var("SHUMA_FORWARDED_IP_SECRET")
        .ok()
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false)
}

pub fn kv_store_fail_open() -> bool {
    env_bool_required("SHUMA_KV_STORE_FAIL_OPEN")
}

fn parse_bool_like(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

#[cfg(test)]
fn parse_bool_env(value: Option<&str>) -> bool {
    value.and_then(parse_bool_like).unwrap_or(false)
}

pub fn pow_config_mutable() -> bool {
    env_bool_required("SHUMA_POW_CONFIG_MUTABLE")
}

#[cfg(test)]
pub(crate) fn challenge_config_mutable_from_env(value: Option<&str>) -> bool {
    parse_bool_env(value)
}

pub fn challenge_config_mutable() -> bool {
    env_bool_required("SHUMA_CHALLENGE_CONFIG_MUTABLE")
}

pub fn botness_config_mutable() -> bool {
    env_bool_required("SHUMA_BOTNESS_CONFIG_MUTABLE")
}

pub fn event_log_retention_hours() -> u64 {
    env_u64_required("SHUMA_EVENT_LOG_RETENTION_HOURS")
}

pub fn env_string_required(name: &str) -> String {
    if cfg!(test) {
        return env::var(name).ok().unwrap_or_else(|| defaults_raw(name));
    }
    env::var(name).unwrap_or_else(|_| panic!("Missing required env var {}", name))
}

fn env_bool_required(name: &str) -> bool {
    if cfg!(test) {
        return env::var(name)
            .ok()
            .and_then(|v| parse_bool_like(v.as_str()))
            .unwrap_or_else(|| defaults_bool(name));
    }
    let value = env::var(name).unwrap_or_else(|_| panic!("Missing required env var {}", name));
    parse_bool_like(&value).unwrap_or_else(|| panic!("Invalid boolean env var {}={}", name, value))
}

fn env_u64_required(name: &str) -> u64 {
    if cfg!(test) {
        return env::var(name)
            .ok()
            .and_then(|v| v.trim().parse::<u64>().ok())
            .unwrap_or_else(|| defaults_u64(name));
    }
    let value = env::var(name).unwrap_or_else(|_| panic!("Missing required env var {}", name));
    value
        .trim()
        .parse::<u64>()
        .unwrap_or_else(|_| panic!("Invalid integer env var {}={}", name, value))
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

fn clamp_challenge_transform_count(value: u8) -> u8 {
    value.clamp(CHALLENGE_TRANSFORM_COUNT_MIN, CHALLENGE_TRANSFORM_COUNT_MAX)
}

fn clamp_config_values(cfg: &mut Config) {
    cfg.pow_difficulty = clamp_pow_difficulty(cfg.pow_difficulty);
    cfg.pow_ttl_seconds = clamp_pow_ttl(cfg.pow_ttl_seconds);
    cfg.challenge_transform_count = clamp_challenge_transform_count(cfg.challenge_transform_count);
    cfg.challenge_risk_threshold = clamp_challenge_threshold(cfg.challenge_risk_threshold);
    cfg.botness_maze_threshold = clamp_maze_threshold(cfg.botness_maze_threshold);
    cfg.botness_weights.js_required = clamp_botness_weight(cfg.botness_weights.js_required);
    cfg.botness_weights.geo_risk = clamp_botness_weight(cfg.botness_weights.geo_risk);
    cfg.botness_weights.rate_medium = clamp_botness_weight(cfg.botness_weights.rate_medium);
    cfg.botness_weights.rate_high = clamp_botness_weight(cfg.botness_weights.rate_high);
    cfg.cdp_detection_threshold = cfg.cdp_detection_threshold.clamp(0.0, 1.0);
}

#[cfg(test)]
pub(crate) fn parse_challenge_threshold(value: Option<&str>) -> u8 {
    let parsed = value
        .and_then(|v| v.parse::<u8>().ok())
        .unwrap_or_else(default_challenge_threshold);
    clamp_challenge_threshold(parsed)
}

#[cfg(test)]
pub(crate) fn parse_maze_threshold(value: Option<&str>) -> u8 {
    let parsed = value
        .and_then(|v| v.parse::<u8>().ok())
        .unwrap_or_else(default_maze_threshold);
    clamp_maze_threshold(parsed)
}

#[cfg(test)]
pub(crate) fn parse_botness_weight(value: Option<&str>, default_value: u8) -> u8 {
    let parsed = value
        .and_then(|v| v.parse::<u8>().ok())
        .unwrap_or(default_value);
    clamp_botness_weight(parsed)
}

fn parse_defaults_env_map(input: &str) -> Result<HashMap<String, String>, String> {
    let mut map = HashMap::new();
    for (index, raw_line) in input.lines().enumerate() {
        let line_no = index + 1;
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, raw_value) = line
            .split_once('=')
            .ok_or_else(|| format!("Invalid defaults line {}: missing '='", line_no))?;

        let key = key.trim();
        if key.is_empty() {
            return Err(format!("Invalid defaults line {}: empty key", line_no));
        }
        if !key
            .chars()
            .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_')
        {
            return Err(format!(
                "Invalid defaults key '{}' on line {}",
                key, line_no
            ));
        }

        let mut value = raw_value.trim().to_string();
        if let Some((head, _)) = value.split_once(" #") {
            value = head.trim().to_string();
        }
        if value.len() >= 2 {
            let first = value.as_bytes()[0] as char;
            let last = value.as_bytes()[value.len() - 1] as char;
            if (first == '"' && last == '"') || (first == '\'' && last == '\'') {
                value = value[1..value.len() - 1].to_string();
            }
        }

        map.insert(key.to_string(), value);
    }
    Ok(map)
}

fn defaults_map() -> &'static HashMap<String, String> {
    match &*DEFAULTS_MAP {
        Ok(map) => map,
        Err(err) => panic!("Invalid config/defaults.env: {}", err),
    }
}

fn defaults_raw(key: &str) -> String {
    defaults_map()
        .get(key)
        .cloned()
        .unwrap_or_else(|| panic!("Missing required defaults key {}", key))
}

fn defaults_bool(key: &str) -> bool {
    parse_bool_like(defaults_raw(key).as_str())
        .unwrap_or_else(|| panic!("Invalid boolean default for {}", key))
}

fn defaults_u64(key: &str) -> u64 {
    defaults_raw(key)
        .trim()
        .parse::<u64>()
        .unwrap_or_else(|_| panic!("Invalid integer default for {}", key))
}

fn defaults_u32(key: &str) -> u32 {
    defaults_raw(key)
        .trim()
        .parse::<u32>()
        .unwrap_or_else(|_| panic!("Invalid integer default for {}", key))
}

fn defaults_u8(key: &str) -> u8 {
    defaults_raw(key)
        .trim()
        .parse::<u8>()
        .unwrap_or_else(|_| panic!("Invalid integer default for {}", key))
}

fn defaults_f32(key: &str) -> f32 {
    defaults_raw(key)
        .trim()
        .parse::<f32>()
        .unwrap_or_else(|_| panic!("Invalid float default for {}", key))
}

fn parse_string_list_value(raw: &str) -> Option<Vec<String>> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Some(Vec::new());
    }
    if let Ok(v) = serde_json::from_str::<Vec<String>>(trimmed) {
        return Some(
            v.into_iter()
                .map(|item| item.trim().to_string())
                .filter(|item| !item.is_empty())
                .collect(),
        );
    }
    Some(
        trimmed
            .split(',')
            .map(|item| item.trim().to_string())
            .filter(|item| !item.is_empty())
            .collect(),
    )
}

fn parse_browser_rules_value(raw: &str) -> Option<Vec<(String, u32)>> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Some(Vec::new());
    }
    if let Ok(v) = serde_json::from_str::<Vec<(String, u32)>>(trimmed) {
        return Some(
            v.into_iter()
                .filter(|(name, _)| !name.trim().is_empty())
                .collect(),
        );
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

fn defaults_string_list(key: &str) -> Vec<String> {
    parse_string_list_value(defaults_raw(key).as_str())
        .unwrap_or_else(|| panic!("Invalid list default for {}", key))
}

fn defaults_country_list(key: &str) -> Vec<String> {
    crate::geo::normalize_country_list(&defaults_string_list(key))
}

fn defaults_browser_rules(key: &str) -> Vec<(String, u32)> {
    parse_browser_rules_value(defaults_raw(key).as_str())
        .unwrap_or_else(|| panic!("Invalid browser rules default for {}", key))
}

fn default_ban_duration() -> u64 {
    defaults_u64("SHUMA_BAN_DURATION")
}

fn default_ban_duration_honeypot() -> u64 {
    defaults_u64("SHUMA_BAN_DURATION_HONEYPOT")
}

fn default_ban_duration_rate_limit() -> u64 {
    defaults_u64("SHUMA_BAN_DURATION_RATE_LIMIT")
}

fn default_ban_duration_browser() -> u64 {
    defaults_u64("SHUMA_BAN_DURATION_BROWSER")
}

fn default_ban_duration_admin() -> u64 {
    defaults_u64("SHUMA_BAN_DURATION_ADMIN")
}

fn default_ban_duration_cdp() -> u64 {
    defaults_u64("SHUMA_BAN_DURATION_CDP")
}

fn default_rate_limit() -> u32 {
    defaults_u32("SHUMA_RATE_LIMIT")
}

fn default_honeypots() -> Vec<String> {
    defaults_string_list("SHUMA_HONEYPOTS")
}

fn default_browser_block() -> Vec<(String, u32)> {
    defaults_browser_rules("SHUMA_BROWSER_BLOCK")
}

fn default_browser_whitelist() -> Vec<(String, u32)> {
    defaults_browser_rules("SHUMA_BROWSER_WHITELIST")
}

fn default_geo_risk() -> Vec<String> {
    defaults_country_list("SHUMA_GEO_RISK_COUNTRIES")
}

fn default_geo_allow() -> Vec<String> {
    defaults_country_list("SHUMA_GEO_ALLOW_COUNTRIES")
}

fn default_geo_challenge() -> Vec<String> {
    defaults_country_list("SHUMA_GEO_CHALLENGE_COUNTRIES")
}

fn default_geo_maze() -> Vec<String> {
    defaults_country_list("SHUMA_GEO_MAZE_COUNTRIES")
}

fn default_geo_block() -> Vec<String> {
    defaults_country_list("SHUMA_GEO_BLOCK_COUNTRIES")
}

fn default_whitelist() -> Vec<String> {
    defaults_string_list("SHUMA_WHITELIST")
}

fn default_path_whitelist() -> Vec<String> {
    defaults_string_list("SHUMA_PATH_WHITELIST")
}

fn default_test_mode() -> bool {
    defaults_bool("SHUMA_TEST_MODE")
}

fn default_maze_enabled() -> bool {
    defaults_bool("SHUMA_MAZE_ENABLED")
}

fn default_maze_auto_ban() -> bool {
    defaults_bool("SHUMA_MAZE_AUTO_BAN")
}

fn default_maze_auto_ban_threshold() -> u32 {
    defaults_u32("SHUMA_MAZE_AUTO_BAN_THRESHOLD")
}

fn default_robots_enabled() -> bool {
    defaults_bool("SHUMA_ROBOTS_ENABLED")
}

fn default_robots_block_ai_training() -> bool {
    defaults_bool("SHUMA_ROBOTS_BLOCK_AI_TRAINING")
}

fn default_robots_block_ai_search() -> bool {
    defaults_bool("SHUMA_ROBOTS_BLOCK_AI_SEARCH")
}

fn default_robots_allow_search_engines() -> bool {
    defaults_bool("SHUMA_ROBOTS_ALLOW_SEARCH_ENGINES")
}

fn default_robots_crawl_delay() -> u32 {
    defaults_u32("SHUMA_ROBOTS_CRAWL_DELAY")
}

fn default_cdp_detection_enabled() -> bool {
    defaults_bool("SHUMA_CDP_DETECTION_ENABLED")
}

fn default_cdp_auto_ban() -> bool {
    defaults_bool("SHUMA_CDP_AUTO_BAN")
}

fn default_cdp_threshold() -> f32 {
    defaults_f32("SHUMA_CDP_DETECTION_THRESHOLD")
}

fn default_js_required_enforced() -> bool {
    defaults_bool("SHUMA_JS_REQUIRED_ENFORCED")
}

fn default_pow_enabled() -> bool {
    defaults_bool("SHUMA_POW_ENABLED")
}

fn default_pow_difficulty() -> u8 {
    clamp_pow_difficulty(defaults_u8("SHUMA_POW_DIFFICULTY"))
}

fn default_pow_ttl_seconds() -> u64 {
    clamp_pow_ttl(defaults_u64("SHUMA_POW_TTL_SECONDS"))
}

fn default_challenge_transform_count() -> u8 {
    clamp_challenge_transform_count(defaults_u8("SHUMA_CHALLENGE_TRANSFORM_COUNT"))
}

fn default_challenge_threshold() -> u8 {
    clamp_challenge_threshold(defaults_u8("SHUMA_CHALLENGE_RISK_THRESHOLD"))
}

fn default_maze_threshold() -> u8 {
    clamp_maze_threshold(defaults_u8("SHUMA_BOTNESS_MAZE_THRESHOLD"))
}

fn default_botness_weight_js_required() -> u8 {
    clamp_botness_weight(defaults_u8("SHUMA_BOTNESS_WEIGHT_JS_REQUIRED"))
}

fn default_botness_weight_geo_risk() -> u8 {
    clamp_botness_weight(defaults_u8("SHUMA_BOTNESS_WEIGHT_GEO_RISK"))
}

fn default_botness_weight_rate_medium() -> u8 {
    clamp_botness_weight(defaults_u8("SHUMA_BOTNESS_WEIGHT_RATE_MEDIUM"))
}

fn default_botness_weight_rate_high() -> u8 {
    clamp_botness_weight(defaults_u8("SHUMA_BOTNESS_WEIGHT_RATE_HIGH"))
}

#[cfg(test)]
mod tests;
