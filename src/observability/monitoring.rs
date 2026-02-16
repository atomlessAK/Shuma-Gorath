use base64::{engine::general_purpose, Engine as _};
use once_cell::sync::Lazy;
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};
use std::sync::Mutex;

const MONITORING_PREFIX: &str = "monitoring:v1";
const MAX_WINDOW_HOURS: u64 = 24 * 30;
const MAX_TOP_LIMIT: usize = 50;

const CHALLENGE_REASON_KEYS: [&str; 5] = [
    "incorrect",
    "expired_replay",
    "sequence_violation",
    "invalid_output",
    "forbidden",
];
const POW_REASON_KEYS: [&str; 5] = [
    "invalid_proof",
    "missing_seed_nonce",
    "sequence_violation",
    "expired_replay",
    "binding_timing_mismatch",
];
const RATE_OUTCOME_KEYS: [&str; 4] = ["limited", "banned", "fallback_allow", "fallback_deny"];
const GEO_ACTION_KEYS: [&str; 3] = ["block", "challenge", "maze"];

static LAST_MONITORING_CLEANUP_HOUR: Lazy<Mutex<u64>> = Lazy::new(|| Mutex::new(0));

#[derive(Debug, Clone, Serialize, Default)]
pub(crate) struct CountEntry {
    pub label: String,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Default)]
pub(crate) struct TrendPoint {
    pub ts: u64,
    pub total: u64,
    pub reasons: BTreeMap<String, u64>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub(crate) struct HoneypotSummary {
    pub total_hits: u64,
    pub unique_crawlers: u64,
    pub top_crawlers: Vec<CountEntry>,
    pub top_paths: Vec<CountEntry>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub(crate) struct FailureSummary {
    pub total_failures: u64,
    pub unique_offenders: u64,
    pub top_offenders: Vec<CountEntry>,
    pub reasons: BTreeMap<String, u64>,
    pub trend: Vec<TrendPoint>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub(crate) struct RateSummary {
    pub total_violations: u64,
    pub unique_offenders: u64,
    pub top_offenders: Vec<CountEntry>,
    pub outcomes: BTreeMap<String, u64>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub(crate) struct GeoSummary {
    pub total_violations: u64,
    pub actions: BTreeMap<String, u64>,
    pub top_countries: Vec<CountEntry>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub(crate) struct MonitoringSummary {
    pub generated_at: u64,
    pub hours: u64,
    pub honeypot: HoneypotSummary,
    pub challenge: FailureSummary,
    pub pow: FailureSummary,
    pub rate: RateSummary,
    pub geo: GeoSummary,
}

fn now_ts() -> u64 {
    crate::admin::now_ts()
}

fn normalize_window_hours(hours: u64) -> u64 {
    hours.clamp(1, MAX_WINDOW_HOURS)
}

fn normalize_top_limit(limit: usize) -> usize {
    limit.clamp(1, MAX_TOP_LIMIT)
}

fn event_log_retention_hours() -> u64 {
    crate::config::event_log_retention_hours()
}

fn encode_dim(value: &str) -> String {
    general_purpose::URL_SAFE_NO_PAD.encode(value.as_bytes())
}

fn decode_dim(value: &str) -> String {
    general_purpose::URL_SAFE_NO_PAD
        .decode(value.as_bytes())
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .unwrap_or_else(|| value.to_string())
}

fn normalize_telemetry_path(path: &str) -> String {
    let mut normalized = path.split('?').next().unwrap_or(path).trim().to_string();
    if normalized.is_empty() {
        normalized = "/".to_string();
    }
    if !normalized.starts_with('/') {
        normalized = format!("/{}", normalized);
    }
    if normalized.len() > 120 {
        normalized.truncate(120);
    }
    normalized
}

fn normalize_challenge_reason(reason: &str) -> &'static str {
    match reason {
        "incorrect" => "incorrect",
        "expired_replay" => "expired_replay",
        "sequence_violation" => "sequence_violation",
        "invalid_output" => "invalid_output",
        "forbidden" => "forbidden",
        _ => "forbidden",
    }
}

fn normalize_pow_reason(reason: &str) -> &'static str {
    match reason {
        "invalid_proof" => "invalid_proof",
        "missing_seed_nonce" => "missing_seed_nonce",
        "sequence_violation" => "sequence_violation",
        "expired_replay" => "expired_replay",
        "binding_timing_mismatch" => "binding_timing_mismatch",
        _ => "sequence_violation",
    }
}

fn normalize_rate_outcome(outcome: &str) -> &'static str {
    match outcome {
        "limited" => "limited",
        "banned" => "banned",
        "fallback_allow" => "fallback_allow",
        "fallback_deny" => "fallback_deny",
        _ => "limited",
    }
}

fn normalize_geo_action(action: &str) -> &'static str {
    match action {
        "block" => "block",
        "challenge" => "challenge",
        "maze" => "maze",
        _ => "block",
    }
}

fn normalize_country(country: Option<&str>) -> String {
    country
        .map(str::trim)
        .map(str::to_ascii_uppercase)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "UNKNOWN".to_string())
}

fn increment_counter<S: crate::challenge::KeyValueStore>(store: &S, key: &str) {
    let current = store
        .get(key)
        .ok()
        .flatten()
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .and_then(|raw| raw.parse::<u64>().ok())
        .unwrap_or(0);
    let next = current.saturating_add(1);
    if let Err(err) = store.set(key, next.to_string().as_bytes()) {
        eprintln!("[monitoring] failed writing {}: {:?}", key, err);
    }
}

fn monitoring_key(section: &str, metric: &str, dimension: Option<&str>, hour: u64) -> String {
    if let Some(value) = dimension {
        return format!(
            "{}:{}:{}:{}:{}",
            MONITORING_PREFIX,
            section,
            metric,
            encode_dim(value),
            hour
        );
    }
    format!("{}:{}:{}:{}", MONITORING_PREFIX, section, metric, hour)
}

fn parse_monitoring_key(key: &str) -> Option<(String, String, Option<String>, u64)> {
    let stripped = key.strip_prefix(format!("{}:", MONITORING_PREFIX).as_str())?;
    let parts: Vec<&str> = stripped.split(':').collect();
    match parts.as_slice() {
        [section, metric, hour] => Some((
            section.to_string(),
            metric.to_string(),
            None,
            hour.parse::<u64>().ok()?,
        )),
        [section, metric, dimension, hour] => Some((
            section.to_string(),
            metric.to_string(),
            Some(decode_dim(dimension)),
            hour.parse::<u64>().ok()?,
        )),
        _ => None,
    }
}

fn maybe_cleanup_monitoring<S: crate::challenge::KeyValueStore>(store: &S, current_hour: u64) {
    let retention = event_log_retention_hours();
    if retention == 0 {
        return;
    }
    let mut last = LAST_MONITORING_CLEANUP_HOUR.lock().unwrap();
    if *last == current_hour {
        return;
    }
    *last = current_hour;

    let cutoff = current_hour.saturating_sub(retention);
    if let Ok(keys) = store.get_keys() {
        for key in keys {
            if !key.starts_with(MONITORING_PREFIX) {
                continue;
            }
            let Some((_, _, _, hour)) = parse_monitoring_key(key.as_str()) else {
                continue;
            };
            if hour < cutoff {
                if let Err(err) = store.delete(key.as_str()) {
                    eprintln!(
                        "[monitoring] failed deleting expired key {}: {:?}",
                        key, err
                    );
                }
            }
        }
    }
}

fn record_with_dimension<S: crate::challenge::KeyValueStore>(
    store: &S,
    section: &str,
    metric: &str,
    dimension: Option<&str>,
) {
    let hour = now_ts() / 3600;
    maybe_cleanup_monitoring(store, hour);
    let key = monitoring_key(section, metric, dimension, hour);
    increment_counter(store, key.as_str());
}

pub(crate) fn record_honeypot_hit<S: crate::challenge::KeyValueStore>(
    store: &S,
    ip: &str,
    path: &str,
) {
    let ip_bucket = crate::signals::ip_identity::bucket_ip(ip);
    let normalized_path = normalize_telemetry_path(path);
    record_with_dimension(store, "honeypot", "total", None);
    record_with_dimension(store, "honeypot", "ip", Some(ip_bucket.as_str()));
    record_with_dimension(store, "honeypot", "path", Some(normalized_path.as_str()));
}

pub(crate) fn record_challenge_failure<S: crate::challenge::KeyValueStore>(
    store: &S,
    ip: &str,
    reason: &str,
) {
    let normalized_reason = normalize_challenge_reason(reason);
    let ip_bucket = crate::signals::ip_identity::bucket_ip(ip);
    record_with_dimension(store, "challenge", "total", None);
    record_with_dimension(store, "challenge", "reason", Some(normalized_reason));
    record_with_dimension(store, "challenge", "ip", Some(ip_bucket.as_str()));
}

pub(crate) fn record_pow_failure<S: crate::challenge::KeyValueStore>(
    store: &S,
    ip: &str,
    reason: &str,
) {
    let normalized_reason = normalize_pow_reason(reason);
    let ip_bucket = crate::signals::ip_identity::bucket_ip(ip);
    record_with_dimension(store, "pow", "total", None);
    record_with_dimension(store, "pow", "reason", Some(normalized_reason));
    record_with_dimension(store, "pow", "ip", Some(ip_bucket.as_str()));
}

pub(crate) fn record_rate_violation<S: crate::challenge::KeyValueStore>(
    store: &S,
    ip: &str,
    outcome: &str,
) {
    let normalized_outcome = normalize_rate_outcome(outcome);
    let ip_bucket = crate::signals::ip_identity::bucket_ip(ip);
    record_with_dimension(store, "rate", "total", None);
    record_with_dimension(store, "rate", "outcome", Some(normalized_outcome));
    record_with_dimension(store, "rate", "ip", Some(ip_bucket.as_str()));
}

pub(crate) fn record_rate_outcome<S: crate::challenge::KeyValueStore>(store: &S, outcome: &str) {
    let normalized_outcome = normalize_rate_outcome(outcome);
    record_with_dimension(store, "rate", "outcome", Some(normalized_outcome));
}

pub(crate) fn record_geo_violation<S: crate::challenge::KeyValueStore>(
    store: &S,
    country: Option<&str>,
    action: &str,
) {
    let normalized_action = normalize_geo_action(action);
    let normalized_country = normalize_country(country);
    record_with_dimension(store, "geo", "total", None);
    record_with_dimension(store, "geo", "action", Some(normalized_action));
    record_with_dimension(store, "geo", "country", Some(normalized_country.as_str()));
}

fn build_seeded_map(keys: &[&str]) -> BTreeMap<String, u64> {
    let mut map = BTreeMap::new();
    for key in keys {
        map.insert((*key).to_string(), 0);
    }
    map
}

fn top_entries(map: &HashMap<String, u64>, limit: usize) -> Vec<CountEntry> {
    let mut rows: Vec<CountEntry> = map
        .iter()
        .map(|(label, count)| CountEntry {
            label: label.clone(),
            count: *count,
        })
        .collect();
    rows.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.label.cmp(&b.label)));
    rows.truncate(limit);
    rows
}

#[derive(Default)]
struct TrendAccumulator {
    totals: HashMap<u64, u64>,
    reasons: HashMap<u64, HashMap<String, u64>>,
}

fn build_trend(
    start_hour: u64,
    end_hour: u64,
    base_reasons: &[&str],
    accumulator: TrendAccumulator,
) -> Vec<TrendPoint> {
    let mut trend = Vec::new();
    for hour in start_hour..=end_hour {
        let mut reasons = build_seeded_map(base_reasons);
        if let Some(row) = accumulator.reasons.get(&hour) {
            for (reason, count) in row {
                let entry = reasons.entry(reason.clone()).or_insert(0);
                *entry = entry.saturating_add(*count);
            }
        }
        let reason_total = reasons.values().copied().sum::<u64>();
        let total = accumulator
            .totals
            .get(&hour)
            .copied()
            .unwrap_or(reason_total);
        trend.push(TrendPoint {
            ts: hour.saturating_mul(3600),
            total,
            reasons,
        });
    }
    trend
}

pub(crate) fn summarize_with_store<S: crate::challenge::KeyValueStore>(
    store: &S,
    hours: u64,
    limit: usize,
) -> MonitoringSummary {
    let now = now_ts();
    let hours = normalize_window_hours(hours);
    let top_limit = normalize_top_limit(limit);
    let end_hour = now / 3600;
    let start_hour = end_hour.saturating_sub(hours.saturating_sub(1));

    let mut honeypot_total = 0u64;
    let mut honeypot_ip_counts: HashMap<String, u64> = HashMap::new();
    let mut honeypot_path_counts: HashMap<String, u64> = HashMap::new();

    let mut challenge_total = 0u64;
    let mut challenge_ip_counts: HashMap<String, u64> = HashMap::new();
    let mut challenge_reason_counts: HashMap<String, u64> = HashMap::new();
    let mut challenge_trend = TrendAccumulator::default();

    let mut pow_total = 0u64;
    let mut pow_ip_counts: HashMap<String, u64> = HashMap::new();
    let mut pow_reason_counts: HashMap<String, u64> = HashMap::new();
    let mut pow_trend = TrendAccumulator::default();

    let mut rate_total = 0u64;
    let mut rate_ip_counts: HashMap<String, u64> = HashMap::new();
    let mut rate_outcomes: HashMap<String, u64> = HashMap::new();

    let mut geo_total = 0u64;
    let mut geo_actions: HashMap<String, u64> = HashMap::new();
    let mut geo_countries: HashMap<String, u64> = HashMap::new();

    if let Ok(keys) = store.get_keys() {
        for key in keys {
            if !key.starts_with(MONITORING_PREFIX) {
                continue;
            }
            let Some((section, metric, dimension, hour)) = parse_monitoring_key(key.as_str()) else {
                continue;
            };
            if hour < start_hour || hour > end_hour {
                continue;
            }
            let count = store
                .get(key.as_str())
                .ok()
                .flatten()
                .and_then(|bytes| String::from_utf8(bytes).ok())
                .and_then(|raw| raw.parse::<u64>().ok())
                .unwrap_or(0);
            if count == 0 {
                continue;
            }

            match section.as_str() {
                "honeypot" => match metric.as_str() {
                    "total" => honeypot_total = honeypot_total.saturating_add(count),
                    "ip" => {
                        if let Some(dim) = dimension {
                            let entry = honeypot_ip_counts.entry(dim).or_insert(0);
                            *entry = entry.saturating_add(count);
                        }
                    }
                    "path" => {
                        if let Some(dim) = dimension {
                            let entry = honeypot_path_counts.entry(dim).or_insert(0);
                            *entry = entry.saturating_add(count);
                        }
                    }
                    _ => {}
                },
                "challenge" => match metric.as_str() {
                    "total" => {
                        challenge_total = challenge_total.saturating_add(count);
                        let entry = challenge_trend.totals.entry(hour).or_insert(0);
                        *entry = entry.saturating_add(count);
                    }
                    "ip" => {
                        if let Some(dim) = dimension {
                            let entry = challenge_ip_counts.entry(dim).or_insert(0);
                            *entry = entry.saturating_add(count);
                        }
                    }
                    "reason" => {
                        if let Some(dim) = dimension {
                            let entry = challenge_reason_counts.entry(dim.clone()).or_insert(0);
                            *entry = entry.saturating_add(count);

                            let row = challenge_trend.reasons.entry(hour).or_default();
                            let reason_entry = row.entry(dim).or_insert(0);
                            *reason_entry = reason_entry.saturating_add(count);
                        }
                    }
                    _ => {}
                },
                "pow" => match metric.as_str() {
                    "total" => {
                        pow_total = pow_total.saturating_add(count);
                        let entry = pow_trend.totals.entry(hour).or_insert(0);
                        *entry = entry.saturating_add(count);
                    }
                    "ip" => {
                        if let Some(dim) = dimension {
                            let entry = pow_ip_counts.entry(dim).or_insert(0);
                            *entry = entry.saturating_add(count);
                        }
                    }
                    "reason" => {
                        if let Some(dim) = dimension {
                            let entry = pow_reason_counts.entry(dim.clone()).or_insert(0);
                            *entry = entry.saturating_add(count);

                            let row = pow_trend.reasons.entry(hour).or_default();
                            let reason_entry = row.entry(dim).or_insert(0);
                            *reason_entry = reason_entry.saturating_add(count);
                        }
                    }
                    _ => {}
                },
                "rate" => match metric.as_str() {
                    "total" => rate_total = rate_total.saturating_add(count),
                    "ip" => {
                        if let Some(dim) = dimension {
                            let entry = rate_ip_counts.entry(dim).or_insert(0);
                            *entry = entry.saturating_add(count);
                        }
                    }
                    "outcome" => {
                        if let Some(dim) = dimension {
                            let entry = rate_outcomes.entry(dim).or_insert(0);
                            *entry = entry.saturating_add(count);
                        }
                    }
                    _ => {}
                },
                "geo" => match metric.as_str() {
                    "total" => geo_total = geo_total.saturating_add(count),
                    "action" => {
                        if let Some(dim) = dimension {
                            let entry = geo_actions.entry(dim).or_insert(0);
                            *entry = entry.saturating_add(count);
                        }
                    }
                    "country" => {
                        if let Some(dim) = dimension {
                            let entry = geo_countries.entry(dim).or_insert(0);
                            *entry = entry.saturating_add(count);
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    let mut challenge_reason_map = build_seeded_map(&CHALLENGE_REASON_KEYS);
    for (key, value) in challenge_reason_counts {
        let entry = challenge_reason_map.entry(key).or_insert(0);
        *entry = entry.saturating_add(value);
    }

    let mut pow_reason_map = build_seeded_map(&POW_REASON_KEYS);
    for (key, value) in pow_reason_counts {
        let entry = pow_reason_map.entry(key).or_insert(0);
        *entry = entry.saturating_add(value);
    }

    let mut rate_outcome_map = build_seeded_map(&RATE_OUTCOME_KEYS);
    for (key, value) in rate_outcomes {
        let entry = rate_outcome_map.entry(key).or_insert(0);
        *entry = entry.saturating_add(value);
    }

    let mut geo_action_map = build_seeded_map(&GEO_ACTION_KEYS);
    for (key, value) in geo_actions {
        let entry = geo_action_map.entry(key).or_insert(0);
        *entry = entry.saturating_add(value);
    }

    MonitoringSummary {
        generated_at: now,
        hours,
        honeypot: HoneypotSummary {
            total_hits: honeypot_total,
            unique_crawlers: honeypot_ip_counts.len() as u64,
            top_crawlers: top_entries(&honeypot_ip_counts, top_limit),
            top_paths: top_entries(&honeypot_path_counts, top_limit),
        },
        challenge: FailureSummary {
            total_failures: challenge_total,
            unique_offenders: challenge_ip_counts.len() as u64,
            top_offenders: top_entries(&challenge_ip_counts, top_limit),
            reasons: challenge_reason_map,
            trend: build_trend(start_hour, end_hour, &CHALLENGE_REASON_KEYS, challenge_trend),
        },
        pow: FailureSummary {
            total_failures: pow_total,
            unique_offenders: pow_ip_counts.len() as u64,
            top_offenders: top_entries(&pow_ip_counts, top_limit),
            reasons: pow_reason_map,
            trend: build_trend(start_hour, end_hour, &POW_REASON_KEYS, pow_trend),
        },
        rate: RateSummary {
            total_violations: rate_total,
            unique_offenders: rate_ip_counts.len() as u64,
            top_offenders: top_entries(&rate_ip_counts, top_limit),
            outcomes: rate_outcome_map,
        },
        geo: GeoSummary {
            total_violations: geo_total,
            actions: geo_action_map,
            top_countries: top_entries(&geo_countries, top_limit),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::challenge::KeyValueStore;
    use std::collections::HashMap;

    #[derive(Default)]
    struct MockStore {
        map: Mutex<HashMap<String, Vec<u8>>>,
    }

    impl crate::challenge::KeyValueStore for MockStore {
        fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ()> {
            let map = self.map.lock().unwrap();
            Ok(map.get(key).cloned())
        }

        fn set(&self, key: &str, value: &[u8]) -> Result<(), ()> {
            let mut map = self.map.lock().unwrap();
            map.insert(key.to_string(), value.to_vec());
            Ok(())
        }

        fn delete(&self, key: &str) -> Result<(), ()> {
            let mut map = self.map.lock().unwrap();
            map.remove(key);
            Ok(())
        }

        fn get_keys(&self) -> Result<Vec<String>, ()> {
            let map = self.map.lock().unwrap();
            Ok(map.keys().cloned().collect())
        }
    }

    fn set_counter(store: &MockStore, key: &str, value: u64) {
        store
            .set(key, value.to_string().as_bytes())
            .expect("counter write should succeed");
    }

    #[test]
    fn summarize_returns_seeded_maps_when_empty() {
        let store = MockStore::default();
        let summary = summarize_with_store(&store, 24, 10);
        assert_eq!(summary.honeypot.total_hits, 0);
        assert_eq!(summary.challenge.total_failures, 0);
        assert_eq!(
            summary.challenge.reasons.get("incorrect").copied().unwrap_or(99),
            0
        );
        assert_eq!(
            summary.pow.reasons.get("invalid_proof").copied().unwrap_or(99),
            0
        );
        assert_eq!(
            summary.rate.outcomes.get("banned").copied().unwrap_or(99),
            0
        );
        assert_eq!(summary.geo.actions.get("maze").copied().unwrap_or(99), 0);
    }

    #[test]
    fn summarize_aggregates_dimension_counts() {
        let store = MockStore::default();
        let now_hour = now_ts() / 3600;

        let hp_ip = encode_dim("10.0.0.0");
        let hp_path = encode_dim("/instaban");
        set_counter(
            &store,
            format!("{}:honeypot:total:{}", MONITORING_PREFIX, now_hour).as_str(),
            3,
        );
        set_counter(
            &store,
            format!("{}:honeypot:ip:{}:{}", MONITORING_PREFIX, hp_ip, now_hour).as_str(),
            3,
        );
        set_counter(
            &store,
            format!(
                "{}:honeypot:path:{}:{}",
                MONITORING_PREFIX, hp_path, now_hour
            )
            .as_str(),
            3,
        );

        let challenge_reason = encode_dim("incorrect");
        let challenge_ip = encode_dim("198.51.100.0");
        set_counter(
            &store,
            format!("{}:challenge:total:{}", MONITORING_PREFIX, now_hour).as_str(),
            2,
        );
        set_counter(
            &store,
            format!(
                "{}:challenge:reason:{}:{}",
                MONITORING_PREFIX, challenge_reason, now_hour
            )
            .as_str(),
            2,
        );
        set_counter(
            &store,
            format!(
                "{}:challenge:ip:{}:{}",
                MONITORING_PREFIX, challenge_ip, now_hour
            )
            .as_str(),
            2,
        );

        let summary = summarize_with_store(&store, 24, 10);
        assert_eq!(summary.honeypot.total_hits, 3);
        assert_eq!(summary.honeypot.unique_crawlers, 1);
        assert_eq!(summary.honeypot.top_paths.first().map(|v| v.count), Some(3));
        assert_eq!(summary.challenge.total_failures, 2);
        assert_eq!(
            summary.challenge.reasons.get("incorrect").copied().unwrap_or(0),
            2
        );
        assert_eq!(summary.challenge.unique_offenders, 1);
        assert_eq!(summary.challenge.trend.last().map(|v| v.total), Some(2));
    }
}
