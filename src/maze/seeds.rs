use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use super::state::MazeStateStore;

const OPERATOR_SOURCES_KEY: &str = "maze:seed:sources:v1";
const OPERATOR_CORPUS_KEY: &str = "maze:seed:corpus:v1";
const OPERATOR_REFRESH_RATE_PREFIX: &str = "maze:seed:refresh_rate";
const MAX_EXTRACTED_TERMS: usize = 512;
const MIN_TERM_LEN: usize = 3;
const MAX_TERM_LEN: usize = 32;

const INTERNAL_FALLBACK_TERMS: &[&str] = &[
    "portal",
    "workflow",
    "system",
    "asset",
    "report",
    "operations",
    "network",
    "service",
    "audit",
    "catalog",
    "inventory",
    "policy",
    "review",
    "batch",
    "runtime",
    "routing",
    "automation",
    "signals",
    "compliance",
    "tracking",
    "analytics",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OperatorSeedSource {
    pub id: String,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub allow_seed_use: bool,
    #[serde(default)]
    pub robots_allowed: bool,
    #[serde(default)]
    pub body_excerpt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredSeedCorpus {
    version: u64,
    refreshed_at: u64,
    provider: String,
    metadata_only: bool,
    source_count: usize,
    terms: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SeedCorpusSnapshot {
    pub version: u64,
    pub refreshed_at: u64,
    pub provider: String,
    pub metadata_only: bool,
    pub source_count: usize,
    pub term_count: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct MazeSeedCorpus {
    pub version: u64,
    pub provider: String,
    pub source_count: usize,
    pub metadata_only: bool,
    pub terms: Vec<String>,
}

impl MazeSeedCorpus {
    fn internal(now_secs: u64) -> Self {
        Self {
            version: now_secs,
            provider: "internal".to_string(),
            source_count: 0,
            metadata_only: true,
            terms: INTERNAL_FALLBACK_TERMS
                .iter()
                .map(|term| term.to_string())
                .collect(),
        }
    }
}

fn parse_corpus(raw: &[u8]) -> Option<StoredSeedCorpus> {
    serde_json::from_slice::<StoredSeedCorpus>(raw).ok()
}

fn read_refresh_count(store: &impl MazeStateStore, hour_bucket: u64) -> u32 {
    let key = format!("{}:{}", OPERATOR_REFRESH_RATE_PREFIX, hour_bucket);
    store
        .get(key.as_str())
        .ok()
        .flatten()
        .and_then(|raw| String::from_utf8(raw).ok())
        .and_then(|raw| raw.parse::<u32>().ok())
        .unwrap_or(0)
}

fn write_refresh_count(store: &impl MazeStateStore, hour_bucket: u64, value: u32) {
    let key = format!("{}:{}", OPERATOR_REFRESH_RATE_PREFIX, hour_bucket);
    if let Err(err) = store.set(key.as_str(), value.to_string().as_bytes()) {
        eprintln!(
            "[maze] failed to persist seed refresh rate key={} err={:?}",
            key, err
        );
    }
}

fn allowed_source_url(url: &str) -> bool {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return false;
    }
    let lower = trimmed.to_ascii_lowercase();
    if !lower.starts_with("https://") {
        return false;
    }
    if lower.contains("localhost")
        || lower.contains("127.0.0.1")
        || lower.contains("[::1]")
        || lower.contains("169.254.")
    {
        return false;
    }
    true
}

fn normalized_source(source: OperatorSeedSource) -> Result<OperatorSeedSource, String> {
    let id = source.id.trim();
    if id.is_empty() {
        return Err("seed source id must be non-empty".to_string());
    }
    if !id
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
    {
        return Err(format!("seed source id '{}' has invalid characters", source.id));
    }

    if let Some(url) = source.url.as_deref() {
        if !allowed_source_url(url) {
            return Err(format!(
                "seed source '{}' url must be https:// and must not target local/private endpoints",
                source.id
            ));
        }
    }

    Ok(OperatorSeedSource {
        id: id.to_string(),
        url: source.url.map(|value| value.trim().to_string()),
        title: source.title.map(|value| value.trim().to_string()),
        description: source.description.map(|value| value.trim().to_string()),
        keywords: source
            .keywords
            .into_iter()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .collect(),
        allow_seed_use: source.allow_seed_use,
        robots_allowed: source.robots_allowed,
        // Legal and bandwidth guardrail: body content is never used for seed extraction.
        body_excerpt: None,
    })
}

fn append_tokens(tokens: &mut BTreeSet<String>, raw: &str) {
    let mut current = String::new();
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() {
            current.push(ch.to_ascii_lowercase());
            if current.len() > MAX_TERM_LEN {
                current.clear();
            }
            continue;
        }
        if (MIN_TERM_LEN..=MAX_TERM_LEN).contains(&current.len()) {
            tokens.insert(current.clone());
        }
        current.clear();
    }
    if (MIN_TERM_LEN..=MAX_TERM_LEN).contains(&current.len()) {
        tokens.insert(current);
    }
}

fn extract_metadata_terms(source: &OperatorSeedSource) -> Vec<String> {
    let mut tokens = BTreeSet::new();
    if let Some(title) = source.title.as_deref() {
        append_tokens(&mut tokens, title);
    }
    if let Some(description) = source.description.as_deref() {
        append_tokens(&mut tokens, description);
    }
    for keyword in &source.keywords {
        append_tokens(&mut tokens, keyword);
    }
    tokens.into_iter().collect()
}

pub(crate) fn list_operator_sources(store: &impl MazeStateStore) -> Vec<OperatorSeedSource> {
    let Some(raw) = store.get(OPERATOR_SOURCES_KEY).ok().flatten() else {
        return Vec::new();
    };
    serde_json::from_slice::<Vec<OperatorSeedSource>>(raw.as_slice()).unwrap_or_default()
}

pub(crate) fn save_operator_sources(
    store: &impl MazeStateStore,
    cfg: &crate::config::Config,
    sources: Vec<OperatorSeedSource>,
) -> Result<(), String> {
    if sources.len() as u32 > cfg.maze_seed_refresh_max_sources {
        return Err(format!(
            "too many seed sources: max={} got={}",
            cfg.maze_seed_refresh_max_sources,
            sources.len()
        ));
    }

    let normalized = sources
        .into_iter()
        .map(normalized_source)
        .collect::<Result<Vec<_>, _>>()?;
    let payload =
        serde_json::to_vec(&normalized).map_err(|_| "failed to serialize seed sources".to_string())?;
    store
        .set(OPERATOR_SOURCES_KEY, payload.as_slice())
        .map_err(|_| "failed to persist seed sources".to_string())
}

pub(crate) fn cached_seed_snapshot(store: &impl MazeStateStore) -> Option<SeedCorpusSnapshot> {
    let raw = store.get(OPERATOR_CORPUS_KEY).ok().flatten()?;
    let parsed = parse_corpus(raw.as_slice())?;
    Some(SeedCorpusSnapshot {
        version: parsed.version,
        refreshed_at: parsed.refreshed_at,
        provider: parsed.provider,
        metadata_only: parsed.metadata_only,
        source_count: parsed.source_count,
        term_count: parsed.terms.len(),
    })
}

fn refresh_operator_corpus_impl(
    store: &impl MazeStateStore,
    cfg: &crate::config::Config,
    now_secs: u64,
    skip_rate_limit: bool,
) -> Result<MazeSeedCorpus, String> {
    let hour_bucket = now_secs / 3600;
    if !skip_rate_limit {
        let current = read_refresh_count(store, hour_bucket);
        if current >= cfg.maze_seed_refresh_rate_limit_per_hour {
            return Err(format!(
                "seed refresh rate limit exceeded for hour bucket {} (limit {})",
                hour_bucket, cfg.maze_seed_refresh_rate_limit_per_hour
            ));
        }
        write_refresh_count(store, hour_bucket, current.saturating_add(1));
    }

    let sources = list_operator_sources(store);
    if sources.is_empty() {
        return Err("no operator seed sources configured".to_string());
    }

    let mut terms = BTreeSet::new();
    let mut accepted_sources = 0usize;
    for source in sources
        .iter()
        .take(cfg.maze_seed_refresh_max_sources as usize)
    {
        if !source.allow_seed_use || !source.robots_allowed {
            continue;
        }
        if let Some(url) = source.url.as_deref() {
            if !allowed_source_url(url) {
                continue;
            }
        }
        accepted_sources = accepted_sources.saturating_add(1);
        for term in extract_metadata_terms(source) {
            terms.insert(term);
        }
        if terms.len() >= MAX_EXTRACTED_TERMS {
            break;
        }
    }

    if terms.is_empty() {
        return Err("no metadata terms extracted from operator seed sources".to_string());
    }
    let terms = terms.into_iter().take(MAX_EXTRACTED_TERMS).collect::<Vec<_>>();
    let stored = StoredSeedCorpus {
        version: now_secs,
        refreshed_at: now_secs,
        provider: cfg.maze_seed_provider.as_str().to_string(),
        metadata_only: true,
        source_count: accepted_sources,
        terms: terms.clone(),
    };
    let raw =
        serde_json::to_vec(&stored).map_err(|_| "failed to serialize seed corpus cache".to_string())?;
    store
        .set(OPERATOR_CORPUS_KEY, raw.as_slice())
        .map_err(|_| "failed to persist seed corpus cache".to_string())?;

    Ok(MazeSeedCorpus {
        version: stored.version,
        provider: stored.provider,
        source_count: stored.source_count,
        metadata_only: stored.metadata_only,
        terms,
    })
}

pub(crate) fn manual_refresh_operator_corpus(
    store: &impl MazeStateStore,
    cfg: &crate::config::Config,
    now_secs: u64,
) -> Result<MazeSeedCorpus, String> {
    refresh_operator_corpus_impl(store, cfg, now_secs, false)
}

pub(crate) fn load_seed_corpus(
    store: &impl MazeStateStore,
    cfg: &crate::config::Config,
    now_secs: u64,
) -> MazeSeedCorpus {
    if cfg.maze_seed_provider != crate::config::MazeSeedProvider::Operator {
        return MazeSeedCorpus::internal(now_secs);
    }

    let cached = store
        .get(OPERATOR_CORPUS_KEY)
        .ok()
        .flatten()
        .and_then(|raw| parse_corpus(raw.as_slice()));
    if let Some(cached) = &cached {
        let age = now_secs.saturating_sub(cached.refreshed_at);
        if age <= cfg.maze_seed_refresh_interval_seconds && !cached.terms.is_empty() {
            return MazeSeedCorpus {
                version: cached.version,
                provider: cached.provider.clone(),
                source_count: cached.source_count,
                metadata_only: cached.metadata_only,
                terms: cached.terms.clone(),
            };
        }
    }

    match refresh_operator_corpus_impl(store, cfg, now_secs, false) {
        Ok(corpus) => corpus,
        Err(err) => {
            eprintln!(
                "[maze] operator seed refresh failed, using cached/internal fallback err={}",
                err
            );
            if let Some(cached) = cached {
                if !cached.terms.is_empty() {
                    return MazeSeedCorpus {
                        version: cached.version,
                        provider: cached.provider,
                        source_count: cached.source_count,
                        metadata_only: cached.metadata_only,
                        terms: cached.terms,
                    };
                }
            }
            MazeSeedCorpus::internal(now_secs)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{load_seed_corpus, manual_refresh_operator_corpus, save_operator_sources, OperatorSeedSource};
    use crate::maze::state::MazeStateStore;
    use std::collections::HashMap;
    use std::sync::Mutex;

    #[derive(Default)]
    struct MemStore {
        data: Mutex<HashMap<String, Vec<u8>>>,
    }

    impl MazeStateStore for MemStore {
        fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ()> {
            Ok(self.data.lock().unwrap().get(key).cloned())
        }

        fn set(&self, key: &str, value: &[u8]) -> Result<(), ()> {
            self.data
                .lock()
                .unwrap()
                .insert(key.to_string(), value.to_vec());
            Ok(())
        }

        fn delete(&self, key: &str) -> Result<(), ()> {
            self.data.lock().unwrap().remove(key);
            Ok(())
        }
    }

    fn operator_cfg() -> crate::config::Config {
        let mut cfg = crate::config::defaults().clone();
        cfg.maze_seed_provider = crate::config::MazeSeedProvider::Operator;
        cfg.maze_seed_refresh_max_sources = 10;
        cfg.maze_seed_refresh_rate_limit_per_hour = 1;
        cfg.maze_seed_refresh_interval_seconds = 60;
        cfg
    }

    #[test]
    fn metadata_extraction_ignores_body_excerpt() {
        let store = MemStore::default();
        let cfg = operator_cfg();
        save_operator_sources(
            &store,
            &cfg,
            vec![OperatorSeedSource {
                id: "news".to_string(),
                url: Some("https://example.com/feed".to_string()),
                title: Some("Zero Trust Routing Update".to_string()),
                description: Some("Policy and signal quality improvements".to_string()),
                keywords: vec!["maze".to_string(), "bot".to_string()],
                allow_seed_use: true,
                robots_allowed: true,
                body_excerpt: Some("private body content should not leak".to_string()),
            }],
        )
        .expect("seed sources should save");
        let refreshed =
            manual_refresh_operator_corpus(&store, &cfg, 1_735_000_000).expect("refresh should pass");
        assert!(refreshed.terms.iter().any(|term| term == "routing"));
        assert!(!refreshed.terms.iter().any(|term| term == "private"));
    }

    #[test]
    fn refresh_is_rate_limited_per_hour() {
        let store = MemStore::default();
        let cfg = operator_cfg();
        save_operator_sources(
            &store,
            &cfg,
            vec![OperatorSeedSource {
                id: "news".to_string(),
                url: Some("https://example.com/feed".to_string()),
                title: Some("Signal window update".to_string()),
                description: Some("queue and budget protections".to_string()),
                keywords: vec!["proof".to_string()],
                allow_seed_use: true,
                robots_allowed: true,
                body_excerpt: None,
            }],
        )
        .expect("seed sources should save");

        assert!(manual_refresh_operator_corpus(&store, &cfg, 1_735_000_000).is_ok());
        assert!(manual_refresh_operator_corpus(&store, &cfg, 1_735_000_100).is_err());
    }

    #[test]
    fn operator_provider_falls_back_to_internal_when_empty() {
        let store = MemStore::default();
        let cfg = operator_cfg();
        let corpus = load_seed_corpus(&store, &cfg, 1_735_000_000);
        assert_eq!(corpus.provider, "internal");
        assert!(!corpus.terms.is_empty());
    }
}
