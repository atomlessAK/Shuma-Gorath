use serde::{Deserialize, Serialize};
use spin_sdk::http::{Request, Response};
use std::time::{SystemTime, UNIX_EPOCH};

use super::content::{
    capitalize, generate_link_text, generate_paragraph, generate_title, DEPARTMENTS, NOUNS,
};
use super::renders::{
    generate_polymorphic_maze_page, AdvancedMazeLink, AdvancedMazeRenderOptions,
};
use super::rng::{generate_path_segment, SeededRng};
use super::state::MazeStateStore;
use super::token::{self, MazeTokenError, MazeTraversalToken};
use super::types::MazeConfig;

const BUDGET_GLOBAL_ACTIVE_KEY: &str = "maze:budget:active:global";
const BUDGET_BUCKET_ACTIVE_PREFIX: &str = "maze:budget:active:bucket";
const TOKEN_REPLAY_PREFIX: &str = "maze:token:seen";
const CHECKPOINT_PREFIX: &str = "maze:checkpoint";
const RISK_PREFIX: &str = "maze:risk";
const MAX_RISK_SCORE: u8 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MazeFallbackReason {
    TokenInvalid,
    TokenExpired,
    TokenReplay,
    TokenBindingMismatch,
    TokenDepthExceeded,
    BudgetExceeded,
    CheckpointMissing,
    MicroPowFailed,
}

impl MazeFallbackReason {
    pub(crate) fn detection_label(self) -> &'static str {
        match self {
            MazeFallbackReason::TokenInvalid => "maze_token_invalid",
            MazeFallbackReason::TokenExpired => "maze_token_expired",
            MazeFallbackReason::TokenReplay => "maze_token_replay",
            MazeFallbackReason::TokenBindingMismatch => "maze_token_binding_mismatch",
            MazeFallbackReason::TokenDepthExceeded => "maze_depth_exceeded",
            MazeFallbackReason::BudgetExceeded => "maze_budget_exceeded",
            MazeFallbackReason::CheckpointMissing => "maze_checkpoint_missing",
            MazeFallbackReason::MicroPowFailed => "maze_micro_pow_failed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MazeRolloutPhase {
    Instrument,
    Advisory,
    Enforce,
}

impl MazeRolloutPhase {
    fn from_config(cfg: &crate::config::Config) -> Self {
        match cfg.maze_rollout_phase.as_str() {
            "instrument" => MazeRolloutPhase::Instrument,
            "advisory" => MazeRolloutPhase::Advisory,
            _ => MazeRolloutPhase::Enforce,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CheckpointState {
    last_ts_ms: u64,
    last_depth: u16,
    expires_at: u64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct MazeRenderResult {
    pub html: String,
    pub depth: u16,
    pub flow_id: String,
    pub variant_id: String,
    pub seed_provider: String,
    pub seed_version: u64,
    pub seed_metadata_only: bool,
    pub seed_source_count: usize,
    pub response_cap_exceeded: bool,
    pub bytes: usize,
    pub render_ms: u64,
    pub token_validated: bool,
}

#[derive(Debug, Clone)]
pub(crate) enum MazeServeDecision {
    Serve(MazeRenderResult),
    Fallback(MazeFallbackReason),
}

struct BudgetLease<'a, S: MazeStateStore> {
    store: &'a S,
    global_key: String,
    bucket_key: String,
    active: bool,
}

impl<'a, S: MazeStateStore> BudgetLease<'a, S> {
    fn release(&mut self) {
        if !self.active {
            return;
        }
        decrement_counter(self.store, self.global_key.as_str());
        decrement_counter(self.store, self.bucket_key.as_str());
        self.active = false;
    }
}

impl<S: MazeStateStore> Drop for BudgetLease<'_, S> {
    fn drop(&mut self) {
        self.release();
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn read_counter(store: &(impl MazeStateStore + ?Sized), key: &str) -> u32 {
    store
        .get(key)
        .ok()
        .flatten()
        .and_then(|raw| String::from_utf8(raw).ok())
        .and_then(|raw| raw.parse::<u32>().ok())
        .unwrap_or(0)
}

fn write_counter(store: &(impl MazeStateStore + ?Sized), key: &str, value: u32) {
    if let Err(err) = store.set(key, value.to_string().as_bytes()) {
        eprintln!("[maze] failed to persist counter key={} err={:?}", key, err);
    }
}

fn increment_counter(store: &(impl MazeStateStore + ?Sized), key: &str) -> u32 {
    let next = read_counter(store, key).saturating_add(1);
    write_counter(store, key, next);
    next
}

fn decrement_counter(store: &(impl MazeStateStore + ?Sized), key: &str) -> u32 {
    let current = read_counter(store, key);
    let next = current.saturating_sub(1);
    write_counter(store, key, next);
    next
}

fn risk_key(ip_bucket: &str) -> String {
    format!("{}:{}", RISK_PREFIX, ip_bucket)
}

pub(crate) fn current_behavior_score(store: &(impl MazeStateStore + ?Sized), ip: &str) -> u8 {
    let key = risk_key(crate::signals::ip_identity::bucket_ip(ip).as_str());
    read_counter(store, key.as_str()).min(MAX_RISK_SCORE as u32) as u8
}

pub(crate) fn increment_behavior_score(
    store: &(impl MazeStateStore + ?Sized),
    ip: &str,
    amount: u8,
) {
    if amount == 0 {
        return;
    }
    let ip_bucket = crate::signals::ip_identity::bucket_ip(ip);
    let key = risk_key(ip_bucket.as_str());
    let next = read_counter(store, key.as_str())
        .saturating_add(amount as u32)
        .min(MAX_RISK_SCORE as u32);
    write_counter(store, key.as_str(), next);
}

fn budget_bucket_key(ip_bucket: &str) -> String {
    format!("{}:{}", BUDGET_BUCKET_ACTIVE_PREFIX, ip_bucket)
}

fn try_acquire_budget<'a, S: MazeStateStore>(
    store: &'a S,
    cfg: &crate::config::Config,
    ip_bucket: &str,
) -> Option<BudgetLease<'a, S>> {
    let global = read_counter(store, BUDGET_GLOBAL_ACTIVE_KEY);
    let bucket_key = budget_bucket_key(ip_bucket);
    let bucket = read_counter(store, bucket_key.as_str());
    if global >= cfg.maze_max_concurrent_global || bucket >= cfg.maze_max_concurrent_per_ip_bucket {
        return None;
    }

    increment_counter(store, BUDGET_GLOBAL_ACTIVE_KEY);
    increment_counter(store, bucket_key.as_str());
    Some(BudgetLease {
        store,
        global_key: BUDGET_GLOBAL_ACTIVE_KEY.to_string(),
        bucket_key,
        active: true,
    })
}

fn token_replay_key(flow_id: &str, operation_id: &str) -> String {
    format!("{}:{}:{}", TOKEN_REPLAY_PREFIX, flow_id, operation_id)
}

fn checkpoint_key(flow_id: &str, ip_bucket: &str) -> String {
    format!("{}:{}:{}", CHECKPOINT_PREFIX, flow_id, ip_bucket)
}

fn replay_seen(
    store: &(impl MazeStateStore + ?Sized),
    flow_id: &str,
    operation_id: &str,
    now: u64,
) -> bool {
    let key = token_replay_key(flow_id, operation_id);
    let seen_until = store
        .get(key.as_str())
        .ok()
        .flatten()
        .and_then(|raw| String::from_utf8(raw).ok())
        .and_then(|raw| raw.parse::<u64>().ok());
    matches!(seen_until, Some(until) if now <= until)
}

fn mark_replay_seen(
    store: &impl MazeStateStore,
    token: &MazeTraversalToken,
    replay_ttl: u64,
    now: u64,
) {
    let key = token_replay_key(token.flow_id.as_str(), token.operation_id.as_str());
    let seen_until = std::cmp::min(token.expires_at, now.saturating_add(replay_ttl));
    if let Err(err) = store.set(key.as_str(), seen_until.to_string().as_bytes()) {
        eprintln!(
            "[maze] failed to persist replay marker key={} err={:?}",
            key, err
        );
    }
}

fn load_checkpoint_state(
    store: &impl MazeStateStore,
    flow_id: &str,
    ip_bucket: &str,
    now: u64,
) -> Option<CheckpointState> {
    let key = checkpoint_key(flow_id, ip_bucket);
    let raw = store.get(key.as_str()).ok().flatten()?;
    let state = serde_json::from_slice::<CheckpointState>(raw.as_slice()).ok()?;
    if now > state.expires_at {
        return None;
    }
    Some(state)
}

fn checkpoint_is_required(cfg: &crate::config::Config, depth: u16) -> bool {
    cfg.maze_client_expansion_enabled && depth as u64 > cfg.maze_checkpoint_every_nodes
}

fn checkpoint_missing(
    store: &impl MazeStateStore,
    cfg: &crate::config::Config,
    token: &MazeTraversalToken,
    ip_bucket: &str,
    now_ms: u64,
) -> bool {
    if !checkpoint_is_required(cfg, token.depth) {
        return false;
    }
    let Some(state) = load_checkpoint_state(store, token.flow_id.as_str(), ip_bucket, now_ms / 1000) else {
        return true;
    };
    let depth_delta = token.depth.saturating_sub(state.last_depth) as u64;
    let elapsed_ms = now_ms.saturating_sub(state.last_ts_ms);
    depth_delta > cfg.maze_step_ahead_max || elapsed_ms > cfg.maze_checkpoint_every_ms
}

fn should_enforce_violation(phase: MazeRolloutPhase, reason: MazeFallbackReason) -> bool {
    match phase {
        MazeRolloutPhase::Enforce => true,
        MazeRolloutPhase::Advisory => matches!(reason, MazeFallbackReason::BudgetExceeded),
        MazeRolloutPhase::Instrument => false,
    }
}

fn pow_difficulty_for_depth(cfg: &crate::config::Config, depth: u16) -> Option<u8> {
    if !cfg.maze_micro_pow_enabled || depth < cfg.maze_micro_pow_depth_start {
        return None;
    }
    let extra = depth.saturating_sub(cfg.maze_micro_pow_depth_start) as u8;
    Some(cfg.maze_micro_pow_base_difficulty.saturating_add(extra).min(24))
}

fn map_token_error(err: MazeTokenError) -> MazeFallbackReason {
    match err {
        MazeTokenError::Expired => MazeFallbackReason::TokenExpired,
        MazeTokenError::Missing | MazeTokenError::Malformed | MazeTokenError::InvalidVersion => {
            MazeFallbackReason::TokenInvalid
        }
        MazeTokenError::SignatureMismatch => MazeFallbackReason::TokenBindingMismatch,
    }
}

fn parse_existing_token(
    store: &impl MazeStateStore,
    cfg: &crate::config::Config,
    query: &str,
    ip_bucket: &str,
    ua_bucket: &str,
    path_prefix: &str,
    now_secs: u64,
    now_ms_value: u64,
) -> Result<Option<(String, MazeTraversalToken)>, MazeFallbackReason> {
    let Some(raw_token) = crate::request_validation::query_param(query, "mt") else {
        return Ok(None);
    };
    let secret = token::secret_from_env();
    let parsed = token::verify(raw_token.as_str(), secret.as_str(), Some(now_secs))
        .map_err(map_token_error)?;
    if parsed.path_prefix != path_prefix || parsed.ip_bucket != ip_bucket || parsed.ua_bucket != ua_bucket {
        return Err(MazeFallbackReason::TokenBindingMismatch);
    }
    if parsed.depth > cfg.maze_token_max_depth {
        return Err(MazeFallbackReason::TokenDepthExceeded);
    }
    if replay_seen(store, parsed.flow_id.as_str(), parsed.operation_id.as_str(), now_secs) {
        return Err(MazeFallbackReason::TokenReplay);
    }
    if checkpoint_missing(store, cfg, &parsed, ip_bucket, now_ms_value) {
        if parsed.depth > cfg.maze_no_js_fallback_max_depth {
            return Err(MazeFallbackReason::CheckpointMissing);
        }
    }
    if let Some(required_pow) = pow_difficulty_for_depth(cfg, parsed.depth) {
        let nonce = crate::request_validation::query_param(query, "mpn").unwrap_or_default();
        if !token::verify_micro_pow(raw_token.as_str(), nonce.as_str(), required_pow) {
            return Err(MazeFallbackReason::MicroPowFailed);
        }
    }

    mark_replay_seen(store, &parsed, cfg.maze_replay_ttl_seconds, now_secs);
    Ok(Some((raw_token, parsed)))
}

pub(crate) fn handle_checkpoint(
    store: &impl MazeStateStore,
    cfg: &crate::config::Config,
    req: &Request,
    ip: &str,
    user_agent: &str,
) -> Response {
    if *req.method() != spin_sdk::http::Method::Post {
        return Response::new(405, "Method Not Allowed");
    }

    let payload = match crate::request_validation::parse_json_body(
        req.body(),
        crate::request_validation::MAX_POW_VERIFY_BYTES,
    ) {
        Ok(payload) => payload,
        Err(_) => return Response::new(400, "Invalid checkpoint payload"),
    };
    let raw_token = payload
        .get("token")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    if raw_token.is_empty() {
        return Response::new(400, "Missing token");
    }

    let secret = token::secret_from_env();
    let now_secs = now_ms() / 1000;
    let parsed = match token::verify(raw_token.as_str(), secret.as_str(), Some(now_secs)) {
        Ok(parsed) => parsed,
        Err(_) => return Response::new(400, "Invalid checkpoint token"),
    };
    let ip_bucket = crate::signals::ip_identity::bucket_ip(ip);
    let ua_bucket = token::ua_bucket(user_agent);
    if parsed.ip_bucket != ip_bucket || parsed.ua_bucket != ua_bucket {
        return Response::new(403, "Checkpoint binding mismatch");
    }

    let now_ms_value = now_ms();
    let state = CheckpointState {
        last_ts_ms: now_ms_value,
        last_depth: parsed.depth,
        expires_at: now_secs.saturating_add(cfg.maze_replay_ttl_seconds),
    };
    let key = checkpoint_key(parsed.flow_id.as_str(), ip_bucket.as_str());
    if let Ok(raw) = serde_json::to_vec(&state) {
        if let Err(err) = store.set(key.as_str(), raw.as_slice()) {
            eprintln!("[maze] failed to persist checkpoint key={} err={:?}", key, err);
        }
    }
    Response::new(204, "")
}

fn flow_entropy_nonce(existing: Option<&MazeTraversalToken>, path_prefix: &str, now_secs: u64, ip_bucket: &str, ua_bucket: &str) -> String {
    existing
        .map(|token| token.entropy_nonce.clone())
        .unwrap_or_else(|| token::flow_id_from(ip_bucket, ua_bucket, path_prefix, now_secs))
}

fn make_breadcrumb(rng: &mut SeededRng) -> String {
    let dept = rng.pick(DEPARTMENTS);
    let noun = capitalize(rng.pick(NOUNS));
    format!("Portal > {} > {} Index", dept, noun)
}

pub(crate) fn serve(
    store: &impl MazeStateStore,
    cfg: &crate::config::Config,
    req: &Request,
    ip: &str,
    user_agent: &str,
    path: &str,
) -> MazeServeDecision {
    let now_ms_value = now_ms();
    let now_secs = now_ms_value / 1000;
    let phase = MazeRolloutPhase::from_config(cfg);
    let query = req.query();
    let ip_bucket = crate::signals::ip_identity::bucket_ip(ip);
    let ua_bucket = token::ua_bucket(user_agent);
    let path_prefix = if path.starts_with("/trap/") {
        "/trap/"
    } else {
        "/maze/"
    };

    let token_ctx = match parse_existing_token(
        store,
        cfg,
        query,
        ip_bucket.as_str(),
        ua_bucket.as_str(),
        path_prefix,
        now_secs,
        now_ms_value,
    ) {
        Ok(ctx) => ctx,
        Err(reason) => {
            increment_behavior_score(store, ip, 2);
            if should_enforce_violation(phase, reason) {
                return MazeServeDecision::Fallback(reason);
            }
            None
        }
    };

    let mut budget_lease = match try_acquire_budget(store, cfg, ip_bucket.as_str()) {
        Some(lease) => lease,
        None => {
            increment_behavior_score(store, ip, 1);
            if should_enforce_violation(phase, MazeFallbackReason::BudgetExceeded) {
                return MazeServeDecision::Fallback(MazeFallbackReason::BudgetExceeded);
            }
            return MazeServeDecision::Serve(MazeRenderResult {
                html: super::generate_maze_page(path, &MazeConfig::default()),
                depth: 0,
                flow_id: token::flow_id_from(ip_bucket.as_str(), ua_bucket.as_str(), path, now_secs),
                variant_id: "legacy-budget-open".to_string(),
                seed_provider: "internal".to_string(),
                seed_version: now_secs,
                seed_metadata_only: true,
                seed_source_count: 0,
                response_cap_exceeded: false,
                bytes: 0,
                render_ms: 0,
                token_validated: false,
            });
        }
    };

    let entropy_nonce = flow_entropy_nonce(
        token_ctx.as_ref().map(|(_, token)| token),
        path_prefix,
        now_secs,
        ip_bucket.as_str(),
        ua_bucket.as_str(),
    );
    let minute_bucket = now_secs / cfg.maze_entropy_window_seconds.max(1);
    let secret = token::secret_from_env();
    let seed = token::entropy_seed(
        secret.as_str(),
        "default",
        ip_bucket.as_str(),
        ua_bucket.as_str(),
        path,
        minute_bucket,
        entropy_nonce.as_str(),
    );
    let mut rng = SeededRng::new(seed);
    let seed_corpus = super::seeds::load_seed_corpus(store, cfg, now_secs);
    let variant_layout = (seed & 0xff) as u8 % 3;
    let variant_palette = ((seed >> 8) & 0xff) as u8 % 3;
    let variant_id = format!(
        "maze-v{}{}-{}-s{}",
        variant_layout, variant_palette, minute_bucket, seed_corpus.version
    );
    let seed_term = if seed_corpus.terms.is_empty() {
        None
    } else {
        let idx = (seed as usize) % seed_corpus.terms.len();
        Some(seed_corpus.terms[idx].clone())
    };

    let render_cfg = MazeConfig::default();
    let paragraph_count = rng
        .range(render_cfg.min_paragraphs, render_cfg.max_paragraphs)
        .min(cfg.maze_max_paragraphs as usize);
    let mut paragraphs = Vec::with_capacity(paragraph_count);
    for index in 0..paragraph_count {
        let mut paragraph = generate_paragraph(&mut rng);
        if index == 0 {
            if let Some(term) = seed_term.as_deref() {
                paragraph.push_str(format!(" Reference focus: {}.", term).as_str());
            }
        }
        paragraphs.push(paragraph);
    }

    let link_count = rng
        .range(render_cfg.min_links, render_cfg.max_links)
        .min(cfg.maze_max_links as usize);
    let mut all_links = Vec::with_capacity(link_count);
    let current_token = token_ctx.as_ref().map(|(_, token)| token);
    for _ in 0..link_count {
        let segment_len = if cfg.maze_path_entropy_segment_len < 8 {
            8
        } else {
            cfg.maze_path_entropy_segment_len as usize
        };
        let next_path = format!("{}{}", path_prefix, generate_path_segment(&mut rng, segment_len));
        let next_token = token::issue_child_token(
            current_token,
            path_prefix,
            ip_bucket.as_str(),
            ua_bucket.as_str(),
            cfg.maze_token_ttl_seconds,
            cfg.maze_token_max_depth,
            cfg.maze_token_branch_budget,
            entropy_nonce.as_str(),
            variant_layout as u16 * 10 + variant_palette as u16,
            now_secs,
        );
        let raw_child = token::sign(&next_token, secret.as_str());
        let href = format!("{}?mt={}", next_path, raw_child);
        let pow = pow_difficulty_for_depth(cfg, next_token.depth);
        let topical_suffix = if seed_corpus.terms.is_empty() {
            None
        } else {
            let idx = (rng.next() as usize) % seed_corpus.terms.len();
            Some(seed_corpus.terms[idx].clone())
        };
        let link_text = if let Some(term) = topical_suffix.as_deref() {
            format!("{} {}", generate_link_text(&mut rng), capitalize(term))
        } else {
            generate_link_text(&mut rng)
        };
        let link_description = if let Some(term) = topical_suffix.as_deref() {
            format!("{} Context stream: {}.", generate_paragraph(&mut rng), term)
        } else {
            generate_paragraph(&mut rng)
        };
        all_links.push(AdvancedMazeLink {
            href,
            text: link_text,
            description: link_description,
            pow_difficulty: pow,
        });
    }

    let visible_links = cfg
        .maze_server_visible_links
        .min(all_links.len() as u32)
        .max(1) as usize;
    let hidden_links = all_links
        .iter()
        .skip(visible_links)
        .map(|link| {
            serde_json::json!({
                "href": link.href,
                "text": link.text,
                "pow_difficulty": link.pow_difficulty
            })
        })
        .collect::<Vec<_>>();

    let checkpoint_token = token_ctx
        .as_ref()
        .map(|(raw, _)| raw.clone())
        .unwrap_or_default();
    let flow_id = token_ctx
        .as_ref()
        .map(|(_, token)| token.flow_id.clone())
        .unwrap_or_else(|| token::flow_id_from(ip_bucket.as_str(), ua_bucket.as_str(), path_prefix, now_secs));
    let current_depth = token_ctx
        .as_ref()
        .map(|(_, token)| token.depth)
        .unwrap_or(0);

    let bootstrap_json = serde_json::json!({
        "flow_id": flow_id,
        "depth": current_depth,
        "checkpoint_token": checkpoint_token,
        "hidden_links": hidden_links
    })
    .to_string();

    let title = generate_title(&mut rng);
    let render_options = AdvancedMazeRenderOptions {
        title,
        breadcrumb: make_breadcrumb(&mut rng),
        paragraphs,
        links: all_links,
        server_visible_links: visible_links,
        bootstrap_json,
        variant_layout,
        variant_palette,
        variant_id: variant_id.clone(),
        rollout_phase: cfg.maze_rollout_phase.as_str().to_string(),
    };
    let started_at = now_ms();
    let html = generate_polymorphic_maze_page(&render_options);
    let elapsed_ms = now_ms().saturating_sub(started_at);
    let bytes = html.as_bytes().len();
    budget_lease.release();

    let response_cap_exceeded =
        bytes > cfg.maze_max_response_bytes as usize || elapsed_ms > cfg.maze_max_response_duration_ms;
    if response_cap_exceeded {
        increment_behavior_score(store, ip, 1);
        if should_enforce_violation(phase, MazeFallbackReason::BudgetExceeded) {
            return MazeServeDecision::Fallback(MazeFallbackReason::BudgetExceeded);
        }
    }

    increment_behavior_score(store, ip, 1);
    MazeServeDecision::Serve(MazeRenderResult {
        html,
        depth: current_depth,
        flow_id,
        variant_id,
        seed_provider: seed_corpus.provider,
        seed_version: seed_corpus.version,
        seed_metadata_only: seed_corpus.metadata_only,
        seed_source_count: seed_corpus.source_count,
        response_cap_exceeded,
        bytes,
        render_ms: elapsed_ms,
        token_validated: token_ctx.is_some(),
    })
}

#[cfg(test)]
mod tests {
    use super::{checkpoint_key, replay_seen, token_replay_key, MazeFallbackReason, MazeServeDecision};
    use crate::maze::state::MazeStateStore;
    use spin_sdk::http::{Method, Request};
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

    #[test]
    fn helper_keys_are_stable() {
        assert_eq!(token_replay_key("f", "o"), "maze:token:seen:f:o");
        assert_eq!(checkpoint_key("f", "ip"), "maze:checkpoint:f:ip");
    }

    #[test]
    fn replay_state_detects_seen() {
        let store = MemStore::default();
        let key = token_replay_key("flow", "op");
        MazeStateStore::set(&store, key.as_str(), b"9999999999").expect("set replay key");
        assert!(replay_seen(&store, "flow", "op", 1));
    }

    #[test]
    fn invalid_token_maps_to_fallback() {
        let store = MemStore::default();
        let cfg = crate::config::defaults().clone();
        let req = Request::builder()
            .method(Method::Get)
            .uri("/maze/abc?mt=bad-token")
            .body(Vec::<u8>::new())
            .build();
        let decision = super::serve(&store, &cfg, &req, "198.51.100.9", "TestUA/1.0", "/maze/abc");
        match decision {
            MazeServeDecision::Fallback(reason) => {
                assert!(
                    matches!(
                        reason,
                        MazeFallbackReason::TokenInvalid | MazeFallbackReason::TokenBindingMismatch
                    ),
                    "unexpected fallback reason: {:?}",
                    reason
                );
            }
            MazeServeDecision::Serve(_) => panic!("expected fallback decision"),
        }
    }
}
