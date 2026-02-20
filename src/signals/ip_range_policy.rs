use crate::config::{
    Config, IpRangeManagedPolicy, IpRangePolicyAction, IpRangePolicyMode, IpRangePolicyRule,
};
use ipnet::IpNet;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::net::IpAddr;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

const MANAGED_IP_RANGES_TEXT: &str = include_str!("../../config/managed_ip_ranges.json");
const CACHE_MAX_ENTRIES: usize = 16;
const MIN_IPV4_PREFIX_LEN: u8 = 8;
const MIN_IPV6_PREFIX_LEN: u8 = 24;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum MatchSource {
    CustomRule,
    ManagedSet,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MatchDetails {
    pub source: MatchSource,
    pub source_id: String,
    pub action: IpRangePolicyAction,
    pub matched_cidr: String,
    pub redirect_url: Option<String>,
    pub custom_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Evaluation {
    NoMatch,
    EmergencyAllowlisted { matched_cidr: String },
    Matched(MatchDetails),
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(crate) struct ManagedSetMetadata {
    pub id: String,
    pub label: String,
    pub provider: String,
    pub source_url: String,
    pub source_timestamp: Option<String>,
    pub source_timestamp_unix: Option<u64>,
    pub version: String,
    pub cidr_count: usize,
    pub catalog_age_hours: u64,
    pub catalog_stale: bool,
    pub managed_max_staleness_hours: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct ManagedCatalog {
    catalog_version: String,
    generated_at: String,
    generated_at_unix: u64,
    sets: Vec<ManagedSet>,
}

#[derive(Debug, Clone, Deserialize)]
struct ManagedSet {
    id: String,
    label: String,
    provider: String,
    source_url: String,
    source_timestamp: Option<String>,
    source_timestamp_unix: Option<u64>,
    version: String,
    cidrs: Vec<String>,
}

#[derive(Debug, Clone)]
struct CompiledRule {
    source: MatchSource,
    source_id: String,
    action: IpRangePolicyAction,
    redirect_url: Option<String>,
    custom_message: Option<String>,
    nets: Vec<IpNet>,
}

#[derive(Debug, Clone, Default)]
struct CompiledPolicy {
    emergency_allowlist: Vec<IpNet>,
    custom_rules: Vec<CompiledRule>,
    managed_rules: Vec<CompiledRule>,
}

static MANAGED_CATALOG: Lazy<ManagedCatalog> = Lazy::new(|| {
    serde_json::from_str::<ManagedCatalog>(MANAGED_IP_RANGES_TEXT)
        .unwrap_or_else(|err| panic!("Invalid managed IP range catalog: {}", err))
});

static COMPILED_POLICY_CACHE: Lazy<Mutex<HashMap<u64, CompiledPolicy>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

fn current_unix() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_secs(),
        Err(_) => 0,
    }
}

fn managed_catalog_age_hours(now_unix: u64) -> u64 {
    if now_unix <= MANAGED_CATALOG.generated_at_unix {
        return 0;
    }
    (now_unix - MANAGED_CATALOG.generated_at_unix) / 3600
}

fn managed_catalog_is_stale(max_staleness_hours: u64, now_unix: u64) -> bool {
    managed_catalog_age_hours(now_unix) > max_staleness_hours
}

pub(crate) fn managed_set_metadata_with_staleness(
    managed_max_staleness_hours: u64,
) -> Vec<ManagedSetMetadata> {
    let now_unix = current_unix();
    let catalog_age_hours = managed_catalog_age_hours(now_unix);
    let catalog_stale = managed_catalog_is_stale(managed_max_staleness_hours, now_unix);
    MANAGED_CATALOG
        .sets
        .iter()
        .map(|set| ManagedSetMetadata {
            id: set.id.clone(),
            label: set.label.clone(),
            provider: set.provider.clone(),
            source_url: set.source_url.clone(),
            source_timestamp: set.source_timestamp.clone(),
            source_timestamp_unix: set.source_timestamp_unix,
            version: set.version.clone(),
            cidr_count: set.cidrs.len(),
            catalog_age_hours,
            catalog_stale,
            managed_max_staleness_hours,
        })
        .collect()
}

pub(crate) fn managed_catalog_version() -> String {
    MANAGED_CATALOG.catalog_version.clone()
}

pub(crate) fn managed_catalog_generated_at() -> String {
    MANAGED_CATALOG.generated_at.clone()
}

pub(crate) fn managed_catalog_generated_at_unix() -> u64 {
    MANAGED_CATALOG.generated_at_unix
}

pub(crate) fn has_managed_set(set_id: &str) -> bool {
    let normalized = set_id.trim().to_ascii_lowercase();
    MANAGED_CATALOG
        .sets
        .iter()
        .any(|set| set.id.eq_ignore_ascii_case(normalized.as_str()))
}

fn policy_cache_key(cfg: &Config) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    cfg.ip_range_policy_mode.as_str().hash(&mut hasher);
    if let Ok(bytes) = serde_json::to_vec(&cfg.ip_range_emergency_allowlist) {
        bytes.hash(&mut hasher);
    }
    if let Ok(bytes) = serde_json::to_vec(&cfg.ip_range_custom_rules) {
        bytes.hash(&mut hasher);
    }
    if let Ok(bytes) = serde_json::to_vec(&cfg.ip_range_managed_policies) {
        bytes.hash(&mut hasher);
    }
    MANAGED_CATALOG.catalog_version.hash(&mut hasher);
    MANAGED_CATALOG.generated_at_unix.hash(&mut hasher);
    hasher.finish()
}

fn sanitize_redirect_url(value: Option<&str>) -> Option<String> {
    let trimmed = value?.trim();
    if trimmed.is_empty() {
        return None;
    }
    let lower = trimmed.to_ascii_lowercase();
    if !lower.starts_with("https://") && !lower.starts_with("http://") {
        return None;
    }
    if trimmed.len() > 512 {
        return None;
    }
    Some(trimmed.to_string())
}

fn sanitize_custom_message(value: Option<&str>) -> Option<String> {
    let raw = value?.trim();
    if raw.is_empty() {
        return None;
    }
    let mut out = String::new();
    for ch in raw.chars() {
        if ch.is_control() && ch != '\n' && ch != '\r' && ch != '\t' {
            continue;
        }
        out.push(ch);
        if out.chars().count() >= 280 {
            break;
        }
    }
    let trimmed = out.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn cidr_is_too_broad(net: &IpNet) -> bool {
    match net {
        IpNet::V4(v4) => v4.prefix_len() < MIN_IPV4_PREFIX_LEN,
        IpNet::V6(v6) => v6.prefix_len() < MIN_IPV6_PREFIX_LEN,
    }
}

pub(crate) fn parse_acceptable_cidr(raw: &str) -> Option<IpNet> {
    let candidate = raw.split('#').next().unwrap_or("").trim();
    if candidate.is_empty() {
        return None;
    }
    let net = candidate.parse::<IpNet>().ok()?;
    if cidr_is_too_broad(&net) {
        return None;
    }
    Some(net)
}

fn parse_cidr_list(cidrs: &[String]) -> Vec<IpNet> {
    let mut parsed = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for raw in cidrs {
        let Some(net) = parse_acceptable_cidr(raw) else {
            continue;
        };
        let canonical = net.to_string();
        if seen.insert(canonical) {
            parsed.push(net);
        }
    }
    parsed
}

fn normalize_rule_id(raw: &str, fallback_index: usize) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return format!("custom_rule_{}", fallback_index);
    }
    let normalized = trimmed
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '_' || *ch == '-')
        .collect::<String>();
    if normalized.is_empty() {
        format!("custom_rule_{}", fallback_index)
    } else {
        normalized
    }
}

fn compile_custom_rule(rule: &IpRangePolicyRule, index: usize) -> Option<CompiledRule> {
    if !rule.enabled {
        return None;
    }
    let nets = parse_cidr_list(&rule.cidrs);
    if nets.is_empty() {
        return None;
    }
    Some(CompiledRule {
        source: MatchSource::CustomRule,
        source_id: normalize_rule_id(rule.id.as_str(), index + 1),
        action: rule.action,
        redirect_url: sanitize_redirect_url(rule.redirect_url.as_deref()),
        custom_message: sanitize_custom_message(rule.custom_message.as_deref()),
        nets,
    })
}

fn compile_managed_rule(policy: &IpRangeManagedPolicy) -> Option<CompiledRule> {
    if !policy.enabled {
        return None;
    }
    let normalized_set_id = policy.set_id.trim().to_ascii_lowercase();
    let set = MANAGED_CATALOG
        .sets
        .iter()
        .find(|entry| entry.id.eq_ignore_ascii_case(normalized_set_id.as_str()))?;
    let nets = parse_cidr_list(&set.cidrs);
    if nets.is_empty() {
        return None;
    }
    Some(CompiledRule {
        source: MatchSource::ManagedSet,
        source_id: set.id.clone(),
        action: policy.action,
        redirect_url: sanitize_redirect_url(policy.redirect_url.as_deref()),
        custom_message: sanitize_custom_message(policy.custom_message.as_deref()),
        nets,
    })
}

fn compile_policy(cfg: &Config) -> CompiledPolicy {
    let emergency_allowlist = parse_cidr_list(&cfg.ip_range_emergency_allowlist);
    let custom_rules = cfg
        .ip_range_custom_rules
        .iter()
        .enumerate()
        .filter_map(|(index, rule)| compile_custom_rule(rule, index))
        .collect::<Vec<_>>();
    let managed_rules = cfg
        .ip_range_managed_policies
        .iter()
        .filter_map(compile_managed_rule)
        .collect::<Vec<_>>();
    CompiledPolicy {
        emergency_allowlist,
        custom_rules,
        managed_rules,
    }
}

fn compiled_policy_for(cfg: &Config) -> CompiledPolicy {
    let key = policy_cache_key(cfg);
    {
        let cache = COMPILED_POLICY_CACHE.lock().unwrap();
        if let Some(policy) = cache.get(&key) {
            return policy.clone();
        }
    }
    let policy = compile_policy(cfg);
    let mut cache = COMPILED_POLICY_CACHE.lock().unwrap();
    if cache.len() >= CACHE_MAX_ENTRIES {
        if let Some(oldest_key) = cache.keys().next().cloned() {
            cache.remove(&oldest_key);
        }
    }
    cache.insert(key, policy.clone());
    policy
}

fn match_rule(rule: &CompiledRule, ip: IpAddr) -> Option<MatchDetails> {
    for net in &rule.nets {
        if net.contains(&ip) {
            return Some(MatchDetails {
                source: rule.source.clone(),
                source_id: rule.source_id.clone(),
                action: rule.action,
                matched_cidr: net.to_string(),
                redirect_url: rule.redirect_url.clone(),
                custom_message: rule.custom_message.clone(),
            });
        }
    }
    None
}

fn evaluate_with_now(cfg: &Config, ip: &str, now_unix: u64) -> Evaluation {
    if cfg.ip_range_policy_mode == IpRangePolicyMode::Off {
        return Evaluation::NoMatch;
    }
    let Ok(ip_addr) = ip.parse::<IpAddr>() else {
        return Evaluation::NoMatch;
    };
    let compiled = compiled_policy_for(cfg);

    for net in &compiled.emergency_allowlist {
        if net.contains(&ip_addr) {
            return Evaluation::EmergencyAllowlisted {
                matched_cidr: net.to_string(),
            };
        }
    }

    for rule in &compiled.custom_rules {
        if let Some(matched) = match_rule(rule, ip_addr) {
            return Evaluation::Matched(matched);
        }
    }

    if cfg.ip_range_policy_mode == IpRangePolicyMode::Enforce
        && !cfg.ip_range_allow_stale_managed_enforce
        && managed_catalog_is_stale(cfg.ip_range_managed_max_staleness_hours, now_unix)
    {
        return Evaluation::NoMatch;
    }

    for rule in &compiled.managed_rules {
        if let Some(matched) = match_rule(rule, ip_addr) {
            return Evaluation::Matched(matched);
        }
    }

    Evaluation::NoMatch
}

pub(crate) fn evaluate(cfg: &Config, ip: &str) -> Evaluation {
    evaluate_with_now(cfg, ip, current_unix())
}

#[cfg(test)]
mod tests {
    use super::{
        evaluate, evaluate_with_now, has_managed_set, managed_set_metadata_with_staleness,
        Evaluation, MatchSource, MANAGED_CATALOG,
    };
    use crate::config::{defaults, IpRangeManagedPolicy, IpRangePolicyAction, IpRangePolicyMode, IpRangePolicyRule};

    #[test]
    fn managed_catalog_exposes_expected_sets() {
        assert!(has_managed_set("openai_gptbot"));
        assert!(has_managed_set("github_copilot"));
        let metadata = managed_set_metadata_with_staleness(168);
        assert!(metadata.iter().any(|entry| entry.id == "openai_chatgpt_user"));
    }

    #[test]
    fn emergency_allowlist_short_circuits_matches() {
        let mut cfg = defaults().clone();
        cfg.ip_range_policy_mode = IpRangePolicyMode::Enforce;
        cfg.ip_range_emergency_allowlist = vec!["203.0.113.0/24".to_string()];
        cfg.ip_range_custom_rules = vec![IpRangePolicyRule {
            id: "block_test".to_string(),
            enabled: true,
            cidrs: vec!["203.0.113.0/24".to_string()],
            action: IpRangePolicyAction::Forbidden403,
            redirect_url: None,
            custom_message: None,
        }];

        let result = evaluate(&cfg, "203.0.113.9");
        assert_eq!(
            result,
            Evaluation::EmergencyAllowlisted {
                matched_cidr: "203.0.113.0/24".to_string()
            }
        );
    }

    #[test]
    fn custom_rule_precedes_managed_rule() {
        let mut cfg = defaults().clone();
        cfg.ip_range_policy_mode = IpRangePolicyMode::Enforce;
        cfg.ip_range_custom_rules = vec![IpRangePolicyRule {
            id: "challenge_me".to_string(),
            enabled: true,
            cidrs: vec!["20.171.206.0/24".to_string()],
            action: IpRangePolicyAction::Maze,
            redirect_url: None,
            custom_message: None,
        }];
        cfg.ip_range_managed_policies = vec![IpRangeManagedPolicy {
            set_id: "openai_gptbot".to_string(),
            enabled: true,
            action: IpRangePolicyAction::Forbidden403,
            redirect_url: None,
            custom_message: None,
        }];

        let result = evaluate(&cfg, "20.171.206.10");
        let Evaluation::Matched(details) = result else {
            panic!("expected custom match");
        };
        assert_eq!(details.source, MatchSource::CustomRule);
        assert_eq!(details.source_id, "challenge_me");
        assert_eq!(details.action, IpRangePolicyAction::Maze);
    }

    #[test]
    fn managed_rule_matches_official_set() {
        let mut cfg = defaults().clone();
        cfg.ip_range_policy_mode = IpRangePolicyMode::Enforce;
        cfg.ip_range_managed_policies = vec![IpRangeManagedPolicy {
            set_id: "github_copilot".to_string(),
            enabled: true,
            action: IpRangePolicyAction::RateLimit,
            redirect_url: None,
            custom_message: None,
        }];

        let result = evaluate(&cfg, "20.85.130.105");
        let Evaluation::Matched(details) = result else {
            panic!("expected managed set match");
        };
        assert_eq!(details.source, MatchSource::ManagedSet);
        assert_eq!(details.source_id, "github_copilot");
        assert_eq!(details.action, IpRangePolicyAction::RateLimit);
    }

    #[test]
    fn managed_sets_are_skipped_when_catalog_is_stale_in_enforce_mode() {
        let mut cfg = defaults().clone();
        cfg.ip_range_policy_mode = IpRangePolicyMode::Enforce;
        cfg.ip_range_managed_max_staleness_hours = 24;
        cfg.ip_range_allow_stale_managed_enforce = false;
        cfg.ip_range_managed_policies = vec![IpRangeManagedPolicy {
            set_id: "github_copilot".to_string(),
            enabled: true,
            action: IpRangePolicyAction::RateLimit,
            redirect_url: None,
            custom_message: None,
        }];

        let stale_now = MANAGED_CATALOG.generated_at_unix + ((cfg.ip_range_managed_max_staleness_hours + 2) * 3600);
        assert_eq!(
            evaluate_with_now(&cfg, "20.85.130.105", stale_now),
            Evaluation::NoMatch
        );
    }

    #[test]
    fn managed_sets_can_be_allowed_when_catalog_is_stale() {
        let mut cfg = defaults().clone();
        cfg.ip_range_policy_mode = IpRangePolicyMode::Enforce;
        cfg.ip_range_managed_max_staleness_hours = 24;
        cfg.ip_range_allow_stale_managed_enforce = true;
        cfg.ip_range_managed_policies = vec![IpRangeManagedPolicy {
            set_id: "github_copilot".to_string(),
            enabled: true,
            action: IpRangePolicyAction::RateLimit,
            redirect_url: None,
            custom_message: None,
        }];

        let stale_now = MANAGED_CATALOG.generated_at_unix + ((cfg.ip_range_managed_max_staleness_hours + 2) * 3600);
        let result = evaluate_with_now(&cfg, "20.85.130.105", stale_now);
        let Evaluation::Matched(details) = result else {
            panic!("expected managed set match with stale override enabled");
        };
        assert_eq!(details.source, MatchSource::ManagedSet);
    }

    #[test]
    fn managed_set_metadata_marks_catalog_staleness() {
        let stale_hours = 1;
        let metadata = managed_set_metadata_with_staleness(stale_hours);
        assert!(!metadata.is_empty());
        assert!(metadata.iter().all(|entry| entry.managed_max_staleness_hours == stale_hours));
    }

    #[test]
    fn invalid_ip_or_mode_off_returns_no_match() {
        let mut cfg = defaults().clone();
        cfg.ip_range_policy_mode = IpRangePolicyMode::Off;
        assert_eq!(evaluate(&cfg, "203.0.113.4"), Evaluation::NoMatch);

        cfg.ip_range_policy_mode = IpRangePolicyMode::Enforce;
        assert_eq!(evaluate(&cfg, "not-an-ip"), Evaluation::NoMatch);
    }
}
