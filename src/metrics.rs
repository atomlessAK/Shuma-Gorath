// src/metrics.rs
// Prometheus-compatible metrics for WASM Bot Trap
// Stores counters in KV store and exports in Prometheus text format

use spin_sdk::key_value::Store;
use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;

const METRICS_PREFIX: &str = "metrics:";

/// Metric types we track
#[derive(Debug, Clone, Copy)]
pub enum MetricName {
    RequestsTotal,
    BansTotal,
    BlocksTotal,
    ChallengesTotal,
    ChallengeServedTotal,
    ChallengeSolvedTotal,
    ChallengeIncorrectTotal,
    ChallengeExpiredReplayTotal,
    WhitelistedTotal,
    TestModeActions,
    MazeHits,
    CdpDetections,
}

impl MetricName {
    fn as_str(&self) -> &'static str {
        match self {
            MetricName::RequestsTotal => "requests_total",
            MetricName::BansTotal => "bans_total",
            MetricName::BlocksTotal => "blocks_total",
            MetricName::ChallengesTotal => "challenges_total",
            MetricName::ChallengeServedTotal => "challenge_served_total",
            MetricName::ChallengeSolvedTotal => "challenge_solved_total",
            MetricName::ChallengeIncorrectTotal => "challenge_incorrect_total",
            MetricName::ChallengeExpiredReplayTotal => "challenge_expired_replay_total",
            MetricName::WhitelistedTotal => "whitelisted_total",
            MetricName::TestModeActions => "test_mode_actions_total",
            MetricName::MazeHits => "maze_hits_total",
            MetricName::CdpDetections => "cdp_detections_total",
        }
    }
}

// In-memory buffer for metric increments to avoid a KV write per request.
// This buffer is flushed to KV when it reaches `FLUSH_KEY_COUNT` distinct keys
// or when an individual buffered counter reaches `FLUSH_VALUE_THRESHOLD`.
static METRICS_BUFFER: Lazy<Mutex<HashMap<String, u64>>> = Lazy::new(|| Mutex::new(HashMap::new()));
const FLUSH_KEY_COUNT: usize = 50;
const FLUSH_VALUE_THRESHOLD: u64 = 10;

/// Increment a counter metric, optionally with a label.
/// This updates an in-memory buffer and flushes to KV on thresholds.
pub fn increment(store: &Store, metric: MetricName, label: Option<&str>) {
    let key = match label {
        Some(l) => format!("{}{}:{}", METRICS_PREFIX, metric.as_str(), l),
        None => format!("{}{}", METRICS_PREFIX, metric.as_str()),
    };

    // Update in-memory buffer
    {
        let mut buf = METRICS_BUFFER.lock().unwrap();
        let v = buf.entry(key.clone()).or_insert(0);
        *v = v.saturating_add(1);
        // if this key reached threshold, flush
        if *v >= FLUSH_VALUE_THRESHOLD || buf.len() >= FLUSH_KEY_COUNT {
            // drop lock then flush below
        } else {
            return;
        }
    }

    // Flush buffer to KV
    let mut to_flush = HashMap::new();
    {
        let mut buf = METRICS_BUFFER.lock().unwrap();
        std::mem::swap(&mut to_flush, &mut *buf);
    }

    // Apply buffered increments to KV
    for (k, v) in to_flush.into_iter() {
        // read current
        let current: u64 = store.get(&k)
            .ok()
            .flatten()
            .and_then(|val| String::from_utf8(val).ok())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let new = current.saturating_add(v);
        if let Err(e) = store.set(&k, new.to_string().as_bytes()) {
            // If a write fails, log and re-insert the delta back into buffer for retry
            eprintln!("[metrics] failed to write metric {} -> {}: {:?}", k, new, e);
            let mut buf = METRICS_BUFFER.lock().unwrap();
            let entry = buf.entry(k).or_insert(0);
            *entry = entry.saturating_add(v);
        }
    }
}

/// Get current value of a counter
fn get_counter(store: &Store, key: &str) -> u64 {
    store.get(key)
        .ok()
        .flatten()
        .and_then(|v| String::from_utf8(v).ok())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

/// Count active bans (gauge)
fn count_active_bans(store: &Store) -> u64 {
    crate::ban::list_active_bans_with_scan(store, "default").len() as u64
}

/// Generate Prometheus-format metrics output
pub fn render_metrics(store: &Store) -> String {
    let mut output = String::new();
    
    // Header
    output.push_str("# WASM Bot Trap Metrics\n");
    output.push_str("# TYPE bot_trap_requests_total counter\n");
    
    // Requests total
    let requests = get_counter(store, &format!("{}requests_total", METRICS_PREFIX));
    output.push_str(&format!("bot_trap_requests_total {}\n", requests));
    
    // Bans by reason
    output.push_str("\n# TYPE bot_trap_bans_total counter\n");
    output.push_str("# HELP bot_trap_bans_total Total number of IP bans by reason\n");
    for reason in &["honeypot", "rate_limit", "browser", "admin", "maze_crawler", "cdp_automation"] {
        let key = format!("{}bans_total:{}", METRICS_PREFIX, reason);
        let count = get_counter(store, &key);
        output.push_str(&format!("bot_trap_bans_total{{reason=\"{}\"}} {}\n", reason, count));
    }
    
    // Blocks total
    output.push_str("\n# TYPE bot_trap_blocks_total counter\n");
    let blocks = get_counter(store, &format!("{}blocks_total", METRICS_PREFIX));
    output.push_str(&format!("bot_trap_blocks_total {}\n", blocks));
    
    // Challenges total
    output.push_str("\n# TYPE bot_trap_challenges_total counter\n");
    let challenges = get_counter(store, &format!("{}challenges_total", METRICS_PREFIX));
    output.push_str(&format!("bot_trap_challenges_total {}\n", challenges));

    // Challenge outcomes
    output.push_str("\n# TYPE bot_trap_challenge_served_total counter\n");
    let challenge_served = get_counter(store, &format!("{}challenge_served_total", METRICS_PREFIX));
    output.push_str(&format!("bot_trap_challenge_served_total {}\n", challenge_served));

    output.push_str("\n# TYPE bot_trap_challenge_solved_total counter\n");
    let challenge_solved = get_counter(store, &format!("{}challenge_solved_total", METRICS_PREFIX));
    output.push_str(&format!("bot_trap_challenge_solved_total {}\n", challenge_solved));

    output.push_str("\n# TYPE bot_trap_challenge_incorrect_total counter\n");
    let challenge_incorrect = get_counter(store, &format!("{}challenge_incorrect_total", METRICS_PREFIX));
    output.push_str(&format!("bot_trap_challenge_incorrect_total {}\n", challenge_incorrect));

    output.push_str("\n# TYPE bot_trap_challenge_expired_replay_total counter\n");
    let challenge_expired_replay = get_counter(store, &format!("{}challenge_expired_replay_total", METRICS_PREFIX));
    output.push_str(&format!("bot_trap_challenge_expired_replay_total {}\n", challenge_expired_replay));
    
    // Whitelisted total
    output.push_str("\n# TYPE bot_trap_whitelisted_total counter\n");
    let whitelisted = get_counter(store, &format!("{}whitelisted_total", METRICS_PREFIX));
    output.push_str(&format!("bot_trap_whitelisted_total {}\n", whitelisted));
    
    // Test mode actions
    output.push_str("\n# TYPE bot_trap_test_mode_actions_total counter\n");
    let test_mode = get_counter(store, &format!("{}test_mode_actions_total", METRICS_PREFIX));
    output.push_str(&format!("bot_trap_test_mode_actions_total {}\n", test_mode));
    
    // Maze hits
    output.push_str("\n# TYPE bot_trap_maze_hits_total counter\n");
    output.push_str("# HELP bot_trap_maze_hits_total Total hits on link maze honeypot pages\n");
    let maze_hits = get_counter(store, &format!("{}maze_hits_total", METRICS_PREFIX));
    output.push_str(&format!("bot_trap_maze_hits_total {}\n", maze_hits));
    
    // Active bans (gauge)
    output.push_str("\n# TYPE bot_trap_active_bans gauge\n");
    output.push_str("# HELP bot_trap_active_bans Current number of active (non-expired) bans\n");
    let active_bans = count_active_bans(store);
    output.push_str(&format!("bot_trap_active_bans {}\n", active_bans));
    
    // Test mode enabled (gauge, 0 or 1)
    output.push_str("\n# TYPE bot_trap_test_mode_enabled gauge\n");
    let cfg = crate::config::Config::load(store, "default");
    let test_mode_enabled = if cfg.test_mode { 1 } else { 0 };
    output.push_str(&format!("bot_trap_test_mode_enabled {}\n", test_mode_enabled));
    
    output
}

/// Handle GET /metrics endpoint
pub fn handle_metrics(store: &Store) -> spin_sdk::http::Response {
    let body = render_metrics(store);
    spin_sdk::http::Response::builder()
        .status(200)
        .header("Content-Type", "text/plain; version=0.0.4; charset=utf-8")
        .body(body)
        .build()
}
