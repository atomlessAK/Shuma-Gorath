// src/observability/metrics.rs
// Prometheus-compatible metrics for WASM Bot Defence
// Stores counters in KV store and exports in Prometheus text format

use once_cell::sync::Lazy;
use spin_sdk::key_value::Store;
use std::collections::HashMap;
use std::sync::Mutex;

const METRICS_PREFIX: &str = "metrics:";
const BOTNESS_SIGNAL_KEYS: [&str; 5] = [
    "js_verification_required",
    "geo_risk",
    "rate_pressure_medium",
    "rate_pressure_high",
    "maze_behavior",
];
const SIGNAL_AVAILABILITY_STATES: [&str; 3] = ["active", "disabled", "unavailable"];
const DEFENCE_MODE_MODULES: [&str; 3] = ["rate", "geo", "js"];
const DEFENCE_MODE_VALUES: [&str; 4] = ["off", "signal", "enforce", "both"];
const EDGE_INTEGRATION_MODES: [&str; 3] = ["off", "advisory", "authoritative"];
const RATE_LIMITER_ROUTE_CLASSES: [&str; 2] = ["main_traffic", "admin_auth"];
const RATE_LIMITER_OUTAGE_MODES: [&str; 3] = ["fallback_internal", "fail_open", "fail_closed"];
const RATE_LIMITER_OUTAGE_ACTIONS: [&str; 3] = ["fallback_internal", "allow", "deny"];
const RATE_LIMITER_USAGE_FALLBACK_REASONS: [&str; 2] = ["backend_error", "backend_missing"];
const RATE_LIMITER_DRIFT_BANDS: [&str; 4] = ["delta_0", "delta_1_5", "delta_6_20", "delta_21_plus"];
const RATE_LIMITER_DECISIONS: [&str; 2] = ["allowed", "limited"];
const MAZE_TOKEN_OUTCOMES: [&str; 10] = [
    "entry",
    "validated",
    "invalid",
    "expired",
    "replay",
    "binding_mismatch",
    "depth_exceeded",
    "checkpoint_missing",
    "budget_exceeded",
    "micro_pow_failed",
];
const MAZE_CHECKPOINT_OUTCOMES: [&str; 4] = [
    "accepted",
    "invalid",
    "binding_mismatch",
    "method_not_allowed",
];
const MAZE_BUDGET_OUTCOMES: [&str; 3] = ["acquired", "saturated", "response_cap_exceeded"];
const MAZE_PROOF_OUTCOMES: [&str; 3] = ["required", "passed", "failed"];
const MONITORING_CHALLENGE_FAILURE_REASON_KEYS: [&str; 5] = [
    "incorrect",
    "expired_replay",
    "sequence_violation",
    "invalid_output",
    "forbidden",
];
const MONITORING_POW_OUTCOME_KEYS: [&str; 2] = ["success", "failure"];
const MONITORING_POW_FAILURE_REASON_KEYS: [&str; 5] = [
    "invalid_proof",
    "missing_seed_nonce",
    "sequence_violation",
    "expired_replay",
    "binding_timing_mismatch",
];
const MONITORING_RATE_OUTCOME_KEYS: [&str; 4] = ["limited", "banned", "fallback_allow", "fallback_deny"];
const MONITORING_GEO_ACTION_KEYS: [&str; 3] = ["block", "challenge", "maze"];
const PROVIDER_OBSERVED_COMBINATIONS: [(
    crate::providers::registry::ProviderCapability,
    crate::config::ProviderBackend,
    &str,
); 10] = [
    (
        crate::providers::registry::ProviderCapability::RateLimiter,
        crate::config::ProviderBackend::Internal,
        "internal",
    ),
    (
        crate::providers::registry::ProviderCapability::RateLimiter,
        crate::config::ProviderBackend::External,
        "external_redis_with_internal_fallback",
    ),
    (
        crate::providers::registry::ProviderCapability::BanStore,
        crate::config::ProviderBackend::Internal,
        "internal",
    ),
    (
        crate::providers::registry::ProviderCapability::BanStore,
        crate::config::ProviderBackend::External,
        "external_redis_with_internal_fallback",
    ),
    (
        crate::providers::registry::ProviderCapability::ChallengeEngine,
        crate::config::ProviderBackend::Internal,
        "internal",
    ),
    (
        crate::providers::registry::ProviderCapability::ChallengeEngine,
        crate::config::ProviderBackend::External,
        "external_stub_unsupported",
    ),
    (
        crate::providers::registry::ProviderCapability::MazeTarpit,
        crate::config::ProviderBackend::Internal,
        "internal",
    ),
    (
        crate::providers::registry::ProviderCapability::MazeTarpit,
        crate::config::ProviderBackend::External,
        "external_stub_unsupported",
    ),
    (
        crate::providers::registry::ProviderCapability::FingerprintSignal,
        crate::config::ProviderBackend::Internal,
        "internal",
    ),
    (
        crate::providers::registry::ProviderCapability::FingerprintSignal,
        crate::config::ProviderBackend::External,
        "external_akamai_with_internal_fallback",
    ),
];

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
    MazeTokenOutcomes,
    MazeCheckpointOutcomes,
    MazeBudgetOutcomes,
    MazeProofOutcomes,
    MazeEntropyVariants,
    CdpDetections,
    BotnessSignalState,
    DefenceModeEffective,
    EdgeIntegrationMode,
    ProviderImplementationEffective,
    RateLimiterBackendErrors,
    RateLimiterOutageDecisions,
    RateLimiterUsageFallback,
    RateLimiterStateDriftObservations,
    PolicyMatches,
    PolicySignals,
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
            MetricName::MazeTokenOutcomes => "maze_token_outcomes_total",
            MetricName::MazeCheckpointOutcomes => "maze_checkpoint_outcomes_total",
            MetricName::MazeBudgetOutcomes => "maze_budget_outcomes_total",
            MetricName::MazeProofOutcomes => "maze_proof_outcomes_total",
            MetricName::MazeEntropyVariants => "maze_entropy_variants_total",
            MetricName::CdpDetections => "cdp_detections_total",
            MetricName::BotnessSignalState => "botness_signal_state_total",
            MetricName::DefenceModeEffective => "defence_mode_effective_total",
            MetricName::EdgeIntegrationMode => "edge_integration_mode_total",
            MetricName::ProviderImplementationEffective => {
                "provider_implementation_effective_total"
            }
            MetricName::RateLimiterBackendErrors => "rate_limiter_backend_errors_total",
            MetricName::RateLimiterOutageDecisions => "rate_limiter_outage_decisions_total",
            MetricName::RateLimiterUsageFallback => "rate_limiter_usage_fallback_total",
            MetricName::RateLimiterStateDriftObservations => {
                "rate_limiter_state_drift_observations_total"
            }
            MetricName::PolicyMatches => "policy_matches_total",
            MetricName::PolicySignals => "policy_signals_total",
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
        let current: u64 = store
            .get(&k)
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

fn record_defence_mode_effective(
    store: &Store,
    module: &str,
    effective: &crate::config::DefenceModeEffective,
) {
    let label = format!(
        "{}:{}:{}:{}",
        module,
        effective.configured.as_str(),
        effective.signal_enabled as u8,
        effective.action_enabled as u8
    );
    increment(
        store,
        MetricName::DefenceModeEffective,
        Some(label.as_str()),
    );
}

pub fn record_botness_visibility(
    store: &Store,
    cfg: &crate::config::Config,
    assessment: &crate::BotnessAssessment,
) {
    for signal in &assessment.contributions {
        let label = format!("{}:{}", signal.key, signal.availability.as_str());
        increment(store, MetricName::BotnessSignalState, Some(label.as_str()));
    }

    let effective = cfg.defence_modes_effective();
    record_defence_mode_effective(store, "rate", &effective.rate);
    record_defence_mode_effective(store, "geo", &effective.geo);
    record_defence_mode_effective(store, "js", &effective.js);
    increment(
        store,
        MetricName::EdgeIntegrationMode,
        Some(cfg.edge_integration_mode.as_str()),
    );
}

pub fn record_provider_backend_visibility(
    store: &Store,
    registry: &crate::providers::registry::ProviderRegistry,
) {
    let capabilities = [
        crate::providers::registry::ProviderCapability::RateLimiter,
        crate::providers::registry::ProviderCapability::BanStore,
        crate::providers::registry::ProviderCapability::ChallengeEngine,
        crate::providers::registry::ProviderCapability::MazeTarpit,
        crate::providers::registry::ProviderCapability::FingerprintSignal,
    ];

    for capability in capabilities {
        let backend = registry.backend_for(capability);
        let implementation = registry.implementation_for(capability);
        let label = format!(
            "{}:{}:{}",
            capability.as_str(),
            backend.as_str(),
            implementation
        );
        increment(
            store,
            MetricName::ProviderImplementationEffective,
            Some(label.as_str()),
        );
    }
}

pub fn record_policy_signal(store: &Store, signal_id: crate::runtime::policy_taxonomy::SignalId) {
    increment(store, MetricName::PolicySignals, Some(signal_id.as_str()));
}

pub fn record_policy_match(
    store: &Store,
    policy_match: &crate::runtime::policy_taxonomy::PolicyMatch,
) {
    let label = format!(
        "{}:{}:{}",
        policy_match.level_id(),
        policy_match.action_id(),
        policy_match.detection_id()
    );
    increment(store, MetricName::PolicyMatches, Some(label.as_str()));
    for signal in policy_match.signal_ids() {
        increment(store, MetricName::PolicySignals, Some(signal));
    }
}

pub fn record_maze_token_outcome(store: &Store, outcome: &str) {
    increment(store, MetricName::MazeTokenOutcomes, Some(outcome));
}

pub fn record_maze_checkpoint_outcome(store: &Store, outcome: &str) {
    increment(store, MetricName::MazeCheckpointOutcomes, Some(outcome));
}

pub fn record_maze_budget_outcome(store: &Store, outcome: &str) {
    increment(store, MetricName::MazeBudgetOutcomes, Some(outcome));
}

pub fn record_maze_proof_outcome(store: &Store, outcome: &str) {
    increment(store, MetricName::MazeProofOutcomes, Some(outcome));
}

pub fn record_maze_entropy_variant(
    store: &Store,
    variant_family: &str,
    provider: &str,
    metadata_only: bool,
) {
    let label = format!("{}:{}:{}", variant_family, provider, metadata_only as u8);
    increment(store, MetricName::MazeEntropyVariants, Some(label.as_str()));
}

/// Get current value of a counter
fn get_counter(store: &Store, key: &str) -> u64 {
    store
        .get(key)
        .ok()
        .flatten()
        .and_then(|v| String::from_utf8(v).ok())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

fn collect_labeled_counters(store: &Store, metric: MetricName) -> Vec<(String, u64)> {
    let mut rows = Vec::new();
    let prefix = format!("{}{}:", METRICS_PREFIX, metric.as_str());

    if let Ok(keys) = store.get_keys() {
        for key in keys {
            let Some(label) = key.strip_prefix(prefix.as_str()) else {
                continue;
            };
            rows.push((label.to_string(), get_counter(store, &key)));
        }
    }

    rows.sort_by(|a, b| a.0.cmp(&b.0));
    rows
}

/// Count active bans (gauge)
fn count_active_bans(store: &Store) -> u64 {
    crate::enforcement::ban::list_active_bans_with_scan(store, "default").len() as u64
}

/// Generate Prometheus-format metrics output
pub fn render_metrics(store: &Store) -> String {
    let mut output = String::new();

    // Header
    output.push_str("# WASM Bot Defence Metrics\n");
    output.push_str("# TYPE bot_defence_requests_total counter\n");

    // Requests total
    let requests = get_counter(store, &format!("{}requests_total", METRICS_PREFIX));
    output.push_str(&format!("bot_defence_requests_total {}\n", requests));

    // Bans by reason
    output.push_str("\n# TYPE bot_defence_bans_total counter\n");
    output.push_str("# HELP bot_defence_bans_total Total number of IP bans by reason\n");
    for reason in &[
        "honeypot",
        "rate_limit",
        "browser",
        "admin",
        "maze_crawler",
        "cdp_automation",
    ] {
        let key = format!("{}bans_total:{}", METRICS_PREFIX, reason);
        let count = get_counter(store, &key);
        output.push_str(&format!(
            "bot_defence_bans_total{{reason=\"{}\"}} {}\n",
            reason, count
        ));
    }

    // Blocks total
    output.push_str("\n# TYPE bot_defence_blocks_total counter\n");
    let blocks = get_counter(store, &format!("{}blocks_total", METRICS_PREFIX));
    output.push_str(&format!("bot_defence_blocks_total {}\n", blocks));

    // Challenges total
    output.push_str("\n# TYPE bot_defence_challenges_total counter\n");
    let challenges = get_counter(store, &format!("{}challenges_total", METRICS_PREFIX));
    output.push_str(&format!("bot_defence_challenges_total {}\n", challenges));

    // Challenge outcomes
    output.push_str("\n# TYPE bot_defence_challenge_served_total counter\n");
    let challenge_served = get_counter(store, &format!("{}challenge_served_total", METRICS_PREFIX));
    output.push_str(&format!(
        "bot_defence_challenge_served_total {}\n",
        challenge_served
    ));

    output.push_str("\n# TYPE bot_defence_challenge_solved_total counter\n");
    let challenge_solved = get_counter(store, &format!("{}challenge_solved_total", METRICS_PREFIX));
    output.push_str(&format!(
        "bot_defence_challenge_solved_total {}\n",
        challenge_solved
    ));

    output.push_str("\n# TYPE bot_defence_challenge_incorrect_total counter\n");
    let challenge_incorrect = get_counter(
        store,
        &format!("{}challenge_incorrect_total", METRICS_PREFIX),
    );
    output.push_str(&format!(
        "bot_defence_challenge_incorrect_total {}\n",
        challenge_incorrect
    ));

    output.push_str("\n# TYPE bot_defence_challenge_expired_replay_total counter\n");
    let challenge_expired_replay = get_counter(
        store,
        &format!("{}challenge_expired_replay_total", METRICS_PREFIX),
    );
    output.push_str(&format!(
        "bot_defence_challenge_expired_replay_total {}\n",
        challenge_expired_replay
    ));

    output.push_str("\n# TYPE bot_defence_cdp_detections_total counter\n");
    output.push_str("# HELP bot_defence_cdp_detections_total Total CDP detection reports processed\n");
    // Keep parity with `/admin/cdp` stats while preserving the buffered metrics key path.
    let cdp_detections = get_counter(store, "cdp:detections")
        .max(get_counter(store, &format!("{}cdp_detections_total", METRICS_PREFIX)));
    output.push_str(&format!(
        "bot_defence_cdp_detections_total {}\n",
        cdp_detections
    ));

    // Whitelisted total
    output.push_str("\n# TYPE bot_defence_whitelisted_total counter\n");
    let whitelisted = get_counter(store, &format!("{}whitelisted_total", METRICS_PREFIX));
    output.push_str(&format!("bot_defence_whitelisted_total {}\n", whitelisted));

    // Test mode actions
    output.push_str("\n# TYPE bot_defence_test_mode_actions_total counter\n");
    let test_mode = get_counter(store, &format!("{}test_mode_actions_total", METRICS_PREFIX));
    output.push_str(&format!(
        "bot_defence_test_mode_actions_total {}\n",
        test_mode
    ));

    // Maze hits
    output.push_str("\n# TYPE bot_defence_maze_hits_total counter\n");
    output.push_str("# HELP bot_defence_maze_hits_total Total hits on maze pages\n");
    let maze_hits = get_counter(store, &format!("{}maze_hits_total", METRICS_PREFIX));
    output.push_str(&format!("bot_defence_maze_hits_total {}\n", maze_hits));

    output.push_str("\n# TYPE bot_defence_maze_token_outcomes_total counter\n");
    output.push_str(
        "# HELP bot_defence_maze_token_outcomes_total Maze traversal token outcomes by outcome label\n",
    );
    for outcome in MAZE_TOKEN_OUTCOMES {
        let key = format!("{}maze_token_outcomes_total:{}", METRICS_PREFIX, outcome);
        let count = get_counter(store, &key);
        output.push_str(&format!(
            "bot_defence_maze_token_outcomes_total{{outcome=\"{}\"}} {}\n",
            outcome, count
        ));
    }

    output.push_str("\n# TYPE bot_defence_maze_checkpoint_outcomes_total counter\n");
    output.push_str(
        "# HELP bot_defence_maze_checkpoint_outcomes_total Maze checkpoint submission outcomes\n",
    );
    for outcome in MAZE_CHECKPOINT_OUTCOMES {
        let key = format!(
            "{}maze_checkpoint_outcomes_total:{}",
            METRICS_PREFIX, outcome
        );
        let count = get_counter(store, &key);
        output.push_str(&format!(
            "bot_defence_maze_checkpoint_outcomes_total{{outcome=\"{}\"}} {}\n",
            outcome, count
        ));
    }

    output.push_str("\n# TYPE bot_defence_maze_budget_outcomes_total counter\n");
    output.push_str("# HELP bot_defence_maze_budget_outcomes_total Maze budget outcomes\n");
    for outcome in MAZE_BUDGET_OUTCOMES {
        let key = format!("{}maze_budget_outcomes_total:{}", METRICS_PREFIX, outcome);
        let count = get_counter(store, &key);
        output.push_str(&format!(
            "bot_defence_maze_budget_outcomes_total{{outcome=\"{}\"}} {}\n",
            outcome, count
        ));
    }

    output.push_str("\n# TYPE bot_defence_maze_proof_outcomes_total counter\n");
    output.push_str(
        "# HELP bot_defence_maze_proof_outcomes_total Maze proof-of-work outcomes for deep traversal\n",
    );
    for outcome in MAZE_PROOF_OUTCOMES {
        let key = format!("{}maze_proof_outcomes_total:{}", METRICS_PREFIX, outcome);
        let count = get_counter(store, &key);
        output.push_str(&format!(
            "bot_defence_maze_proof_outcomes_total{{outcome=\"{}\"}} {}\n",
            outcome, count
        ));
    }

    output.push_str("\n# TYPE bot_defence_maze_entropy_variants_total counter\n");
    output.push_str(
        "# HELP bot_defence_maze_entropy_variants_total Maze entropy variant families by provider and metadata posture\n",
    );
    for (label, count) in collect_labeled_counters(store, MetricName::MazeEntropyVariants) {
        let mut parts = label.splitn(3, ':');
        let variant = parts.next().unwrap_or("unknown");
        let provider = parts.next().unwrap_or("unknown");
        let metadata_only = parts.next().unwrap_or("unknown");
        output.push_str(&format!(
            "bot_defence_maze_entropy_variants_total{{variant=\"{}\",provider=\"{}\",metadata_only=\"{}\"}} {}\n",
            variant, provider, metadata_only, count
        ));
    }

    // Botness signal states
    output.push_str("\n# TYPE bot_defence_botness_signal_state_total counter\n");
    output.push_str(
        "# HELP bot_defence_botness_signal_state_total Botness signal state observations by signal key and availability\n",
    );
    for signal_key in BOTNESS_SIGNAL_KEYS {
        for state in SIGNAL_AVAILABILITY_STATES {
            let key = format!(
                "{}botness_signal_state_total:{}:{}",
                METRICS_PREFIX, signal_key, state
            );
            let count = get_counter(store, &key);
            output.push_str(&format!(
                "bot_defence_botness_signal_state_total{{signal=\"{}\",state=\"{}\"}} {}\n",
                signal_key, state, count
            ));
        }
    }

    // Effective defence modes (runtime-observed)
    output.push_str("\n# TYPE bot_defence_defence_mode_effective_total counter\n");
    output.push_str(
        "# HELP bot_defence_defence_mode_effective_total Observed effective defence mode combinations by module\n",
    );
    for module in DEFENCE_MODE_MODULES {
        for configured_mode in DEFENCE_MODE_VALUES {
            for signal_enabled in [false, true] {
                for action_enabled in [false, true] {
                    let key = format!(
                        "{}defence_mode_effective_total:{}:{}:{}:{}",
                        METRICS_PREFIX,
                        module,
                        configured_mode,
                        signal_enabled as u8,
                        action_enabled as u8
                    );
                    let count = get_counter(store, &key);
                    output.push_str(&format!(
                        "bot_defence_defence_mode_effective_total{{module=\"{}\",configured=\"{}\",signal_enabled=\"{}\",action_enabled=\"{}\"}} {}\n",
                        module,
                        configured_mode,
                        signal_enabled,
                        action_enabled,
                        count
                    ));
                }
            }
        }
    }

    // Edge integration mode observations
    output.push_str("\n# TYPE bot_defence_edge_integration_mode_total counter\n");
    output.push_str(
        "# HELP bot_defence_edge_integration_mode_total Observed configured edge integration mode\n",
    );
    for mode in EDGE_INTEGRATION_MODES {
        let key = format!("{}edge_integration_mode_total:{}", METRICS_PREFIX, mode);
        let count = get_counter(store, &key);
        output.push_str(&format!(
            "bot_defence_edge_integration_mode_total{{mode=\"{}\"}} {}\n",
            mode, count
        ));
    }

    // Active provider implementation observations
    output.push_str("\n# TYPE bot_defence_provider_implementation_effective_total counter\n");
    output.push_str(
        "# HELP bot_defence_provider_implementation_effective_total Observed active provider backend and implementation by capability\n",
    );
    for (capability, backend, implementation) in PROVIDER_OBSERVED_COMBINATIONS {
        let key = format!(
            "{}provider_implementation_effective_total:{}:{}:{}",
            METRICS_PREFIX,
            capability.as_str(),
            backend.as_str(),
            implementation
        );
        let count = get_counter(store, &key);
        output.push_str(&format!(
            "bot_defence_provider_implementation_effective_total{{capability=\"{}\",backend=\"{}\",implementation=\"{}\"}} {}\n",
            capability.as_str(),
            backend.as_str(),
            implementation,
            count
        ));
    }

    // External rate-limiter backend errors
    output.push_str("\n# TYPE bot_defence_rate_limiter_backend_errors_total counter\n");
    output.push_str(
        "# HELP bot_defence_rate_limiter_backend_errors_total External rate-limiter backend errors by route class\n",
    );
    for route_class in RATE_LIMITER_ROUTE_CLASSES {
        let key = format!(
            "{}rate_limiter_backend_errors_total:{}",
            METRICS_PREFIX, route_class
        );
        let count = get_counter(store, &key);
        output.push_str(&format!(
            "bot_defence_rate_limiter_backend_errors_total{{route_class=\"{}\"}} {}\n",
            route_class, count
        ));
    }

    // External rate-limiter outage decisions
    output.push_str("\n# TYPE bot_defence_rate_limiter_outage_decisions_total counter\n");
    output.push_str(
        "# HELP bot_defence_rate_limiter_outage_decisions_total Degraded external rate-limiter decisions by route class, outage mode, action, and decision\n",
    );
    for route_class in RATE_LIMITER_ROUTE_CLASSES {
        for mode in RATE_LIMITER_OUTAGE_MODES {
            for action in RATE_LIMITER_OUTAGE_ACTIONS {
                for decision in RATE_LIMITER_DECISIONS {
                    let key = format!(
                        "{}rate_limiter_outage_decisions_total:{}:{}:{}:{}",
                        METRICS_PREFIX, route_class, mode, action, decision
                    );
                    let count = get_counter(store, &key);
                    output.push_str(&format!(
                        "bot_defence_rate_limiter_outage_decisions_total{{route_class=\"{}\",mode=\"{}\",action=\"{}\",decision=\"{}\"}} {}\n",
                        route_class, mode, action, decision, count
                    ));
                }
            }
        }
    }

    // External rate-limiter usage fallback observations
    output.push_str("\n# TYPE bot_defence_rate_limiter_usage_fallback_total counter\n");
    output.push_str(
        "# HELP bot_defence_rate_limiter_usage_fallback_total External rate-limiter usage read fallback observations by route class and reason\n",
    );
    for route_class in RATE_LIMITER_ROUTE_CLASSES {
        for reason in RATE_LIMITER_USAGE_FALLBACK_REASONS {
            let key = format!(
                "{}rate_limiter_usage_fallback_total:{}:{}",
                METRICS_PREFIX, route_class, reason
            );
            let count = get_counter(store, &key);
            output.push_str(&format!(
                "bot_defence_rate_limiter_usage_fallback_total{{route_class=\"{}\",reason=\"{}\"}} {}\n",
                route_class, reason, count
            ));
        }
    }

    // External/local distributed state drift observations
    output.push_str("\n# TYPE bot_defence_rate_limiter_state_drift_observations_total counter\n");
    output.push_str(
        "# HELP bot_defence_rate_limiter_state_drift_observations_total Observed absolute drift bands between external distributed and local shadow rate counters\n",
    );
    for route_class in RATE_LIMITER_ROUTE_CLASSES {
        for band in RATE_LIMITER_DRIFT_BANDS {
            let key = format!(
                "{}rate_limiter_state_drift_observations_total:{}:{}",
                METRICS_PREFIX, route_class, band
            );
            let count = get_counter(store, &key);
            output.push_str(&format!(
                "bot_defence_rate_limiter_state_drift_observations_total{{route_class=\"{}\",delta_band=\"{}\"}} {}\n",
                route_class, band, count
            ));
        }
    }

    // Canonical policy matches
    output.push_str("\n# TYPE bot_defence_policy_matches_total counter\n");
    output.push_str(
        "# HELP bot_defence_policy_matches_total Canonical policy match observations by escalation level, action, and detection ID\n",
    );
    for (label, count) in collect_labeled_counters(store, MetricName::PolicyMatches) {
        let mut parts = label.splitn(3, ':');
        let level = parts.next().unwrap_or("unknown");
        let action = parts.next().unwrap_or("unknown");
        let detection = parts.next().unwrap_or("unknown");
        output.push_str(&format!(
            "bot_defence_policy_matches_total{{level=\"{}\",action=\"{}\",detection=\"{}\"}} {}\n",
            level, action, detection, count
        ));
    }

    // Canonical signal observations
    output.push_str("\n# TYPE bot_defence_policy_signals_total counter\n");
    output.push_str(
        "# HELP bot_defence_policy_signals_total Canonical signal ID observations across policy decisions\n",
    );
    for (signal, count) in collect_labeled_counters(store, MetricName::PolicySignals) {
        output.push_str(&format!(
            "bot_defence_policy_signals_total{{signal=\"{}\"}} {}\n",
            signal, count
        ));
    }

    let monitoring_summary = crate::observability::monitoring::summarize_metrics_window(store);

    output.push_str("\n# TYPE bot_defence_monitoring_challenge_failures_total counter\n");
    output.push_str(
        "# HELP bot_defence_monitoring_challenge_failures_total Monitoring challenge failures by normalized reason\n",
    );
    for reason in MONITORING_CHALLENGE_FAILURE_REASON_KEYS {
        let count = monitoring_summary
            .challenge
            .reasons
            .get(reason)
            .copied()
            .unwrap_or(0);
        output.push_str(&format!(
            "bot_defence_monitoring_challenge_failures_total{{reason=\"{}\"}} {}\n",
            reason, count
        ));
    }

    output.push_str("\n# TYPE bot_defence_monitoring_pow_verifications_total counter\n");
    output.push_str(
        "# HELP bot_defence_monitoring_pow_verifications_total Monitoring PoW verification outcomes\n",
    );
    for outcome in MONITORING_POW_OUTCOME_KEYS {
        let count = monitoring_summary
            .pow
            .outcomes
            .get(outcome)
            .copied()
            .unwrap_or(0);
        output.push_str(&format!(
            "bot_defence_monitoring_pow_verifications_total{{outcome=\"{}\"}} {}\n",
            outcome, count
        ));
    }

    output.push_str("\n# TYPE bot_defence_monitoring_pow_failures_total counter\n");
    output.push_str(
        "# HELP bot_defence_monitoring_pow_failures_total Monitoring PoW verification failures by normalized reason\n",
    );
    for reason in MONITORING_POW_FAILURE_REASON_KEYS {
        let count = monitoring_summary
            .pow
            .reasons
            .get(reason)
            .copied()
            .unwrap_or(0);
        output.push_str(&format!(
            "bot_defence_monitoring_pow_failures_total{{reason=\"{}\"}} {}\n",
            reason, count
        ));
    }

    output.push_str("\n# TYPE bot_defence_monitoring_rate_violations_total counter\n");
    output.push_str(
        "# HELP bot_defence_monitoring_rate_violations_total Monitoring rate-limit violations by normalized outcome\n",
    );
    for outcome in MONITORING_RATE_OUTCOME_KEYS {
        let count = monitoring_summary
            .rate
            .outcomes
            .get(outcome)
            .copied()
            .unwrap_or(0);
        output.push_str(&format!(
            "bot_defence_monitoring_rate_violations_total{{outcome=\"{}\"}} {}\n",
            outcome, count
        ));
    }

    output.push_str("\n# TYPE bot_defence_monitoring_geo_violations_total counter\n");
    output.push_str(
        "# HELP bot_defence_monitoring_geo_violations_total Monitoring GEO policy violations by normalized action\n",
    );
    for action in MONITORING_GEO_ACTION_KEYS {
        let count = monitoring_summary.geo.actions.get(action).copied().unwrap_or(0);
        output.push_str(&format!(
            "bot_defence_monitoring_geo_violations_total{{action=\"{}\"}} {}\n",
            action, count
        ));
    }

    // Active bans (gauge)
    output.push_str("\n# TYPE bot_defence_active_bans gauge\n");
    output.push_str("# HELP bot_defence_active_bans Current number of active (non-expired) bans\n");
    let active_bans = count_active_bans(store);
    output.push_str(&format!("bot_defence_active_bans {}\n", active_bans));

    // Test mode enabled (gauge, 0 or 1)
    output.push_str("\n# TYPE bot_defence_test_mode_enabled gauge\n");
    let test_mode_enabled = crate::config::load_runtime_cached(store, "default")
        .map(|cfg| if cfg.test_mode { 1 } else { 0 })
        .unwrap_or(0);
    output.push_str(&format!(
        "bot_defence_test_mode_enabled {}\n",
        test_mode_enabled
    ));

    output
}

/// Handle GET /metrics endpoint
pub fn handle_metrics(store: &Store) -> spin_sdk::http::Response {
    if crate::config::load_runtime_cached(store, "default").is_err() {
        return spin_sdk::http::Response::new(500, "Configuration unavailable");
    }
    let body = render_metrics(store);
    spin_sdk::http::Response::builder()
        .status(200)
        .header("Content-Type", "text/plain; version=0.0.4; charset=utf-8")
        .body(body)
        .build()
}
