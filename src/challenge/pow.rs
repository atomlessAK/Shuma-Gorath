// src/challenge/pow.rs
// Lightweight proof-of-work (PoW) challenge for JS verification

use base64::{engine::general_purpose, Engine as _};
use hmac::{Hmac, Mac};
use once_cell::sync::Lazy;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use spin_sdk::http::{Request, Response};
use spin_sdk::key_value::Store;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Serialize, Deserialize)]
struct PowPayload {
    seed_id: String,
    operation_id: String,
    flow_id: String,
    step_id: String,
    step_index: u8,
    ip_bucket: String,
    ua_bucket: String,
    path_class: String,
    issued_at: u64,
    expires_at: u64,
    token_version: u8,
    difficulty: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PowChallenge {
    pub seed: String,
    pub difficulty: u8,
    pub expires_at: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PowSeedTokenError {
    MissingPayload,
    MissingSignature,
    InvalidPayloadEncoding,
    InvalidSignatureEncoding,
    InvalidPayloadUtf8,
    SignatureMismatch,
    InvalidPayloadJson,
    InvalidOperationEnvelope(crate::challenge::operation_envelope::EnvelopeValidationError),
}

fn now_ts() -> u64 {
    crate::admin::now_ts()
}

#[cfg(not(test))]
fn try_open_default_store() -> Option<Store> {
    std::panic::catch_unwind(|| Store::open_default().ok())
        .ok()
        .flatten()
}

#[cfg(test)]
fn try_open_default_store() -> Option<Store> {
    None
}

fn get_pow_secret() -> String {
    match std::env::var("SHUMA_POW_SECRET") {
        Ok(secret) if !secret.trim().is_empty() => secret,
        _ => std::env::var("SHUMA_JS_SECRET")
            .ok()
            .filter(|secret| !secret.trim().is_empty())
            .unwrap_or_else(|| "pow-default-secret".to_string()),
    }
}

fn sign_payload(payload: &str) -> Vec<u8> {
    let secret = get_pow_secret();
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(payload.as_bytes());
    mac.finalize().into_bytes().to_vec()
}

fn verify_signature(payload: &str, sig: &[u8]) -> bool {
    let secret = get_pow_secret();
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(payload.as_bytes());
    mac.verify_slice(sig).is_ok()
}

fn make_seed_token(payload: &PowPayload) -> String {
    let payload_json = serde_json::to_string(payload).unwrap();
    let sig = sign_payload(&payload_json);
    let payload_b64 = general_purpose::STANDARD.encode(payload_json.as_bytes());
    let sig_b64 = general_purpose::STANDARD.encode(sig);
    format!("{}.{}", payload_b64, sig_b64)
}

fn parse_seed_token(token: &str) -> Result<PowPayload, PowSeedTokenError> {
    let mut parts = token.splitn(2, '.');
    let payload_b64 = parts.next().ok_or(PowSeedTokenError::MissingPayload)?;
    let sig_b64 = parts.next().ok_or(PowSeedTokenError::MissingSignature)?;
    let payload_bytes = general_purpose::STANDARD
        .decode(payload_b64.as_bytes())
        .map_err(|_| PowSeedTokenError::InvalidPayloadEncoding)?;
    let sig = general_purpose::STANDARD
        .decode(sig_b64.as_bytes())
        .map_err(|_| PowSeedTokenError::InvalidSignatureEncoding)?;
    let payload_json =
        String::from_utf8(payload_bytes).map_err(|_| PowSeedTokenError::InvalidPayloadUtf8)?;

    if !verify_signature(&payload_json, &sig) {
        return Err(PowSeedTokenError::SignatureMismatch);
    }

    let payload = serde_json::from_str::<PowPayload>(&payload_json)
        .map_err(|_| PowSeedTokenError::InvalidPayloadJson)?;
    crate::challenge::operation_envelope::validate_signed_operation_envelope(
        payload.operation_id.as_str(),
        payload.flow_id.as_str(),
        payload.step_id.as_str(),
        payload.issued_at,
        payload.expires_at,
        payload.token_version,
        crate::challenge::operation_envelope::FLOW_JS_VERIFICATION,
        crate::challenge::operation_envelope::STEP_JS_POW_VERIFY,
    )
    .map_err(PowSeedTokenError::InvalidOperationEnvelope)?;
    Ok(payload)
}

fn has_leading_zero_bits(hash: &[u8], bits: u8) -> bool {
    let mut remaining = bits as i32;
    for b in hash {
        if remaining <= 0 {
            return true;
        }
        if remaining >= 8 {
            if *b != 0 {
                return false;
            }
            remaining -= 8;
        } else {
            let mask: u8 = 0xFF << (8 - remaining as u8);
            return (b & mask) == 0;
        }
    }
    true
}

fn verify_pow(seed_token: &str, nonce: &str, difficulty: u8) -> bool {
    let mut hasher = Sha256::new();
    hasher.update(seed_token.as_bytes());
    hasher.update(b":");
    hasher.update(nonce.as_bytes());
    let hash = hasher.finalize();
    has_leading_zero_bits(&hash, difficulty)
}

pub fn issue_pow_challenge(
    ip: &str,
    user_agent: &str,
    difficulty: u8,
    ttl_seconds: u64,
) -> PowChallenge {
    let now = now_ts();
    let ttl = ttl_seconds;
    let seed_id = format!("{:016x}", rand::rng().random::<u64>());
    let mut rng = rand::rng();
    let payload = PowPayload {
        seed_id,
        operation_id: format!("{:016x}{:016x}", rng.random::<u64>(), rng.random::<u64>()),
        flow_id: crate::challenge::operation_envelope::FLOW_JS_VERIFICATION.to_string(),
        step_id: crate::challenge::operation_envelope::STEP_JS_POW_VERIFY.to_string(),
        step_index: crate::challenge::operation_envelope::STEP_INDEX_JS_POW_VERIFY,
        ip_bucket: crate::signals::ip_identity::bucket_ip(ip),
        ua_bucket: crate::challenge::operation_envelope::user_agent_bucket(user_agent),
        path_class: crate::challenge::operation_envelope::PATH_CLASS_JS_POW_VERIFY.to_string(),
        issued_at: now,
        expires_at: now + ttl,
        token_version: crate::challenge::operation_envelope::TOKEN_VERSION_V1,
        difficulty,
    };
    let seed = make_seed_token(&payload);
    PowChallenge {
        seed,
        difficulty,
        expires_at: payload.expires_at,
    }
}

pub fn handle_pow_challenge(
    ip: &str,
    user_agent: &str,
    pow_enabled: bool,
    difficulty: u8,
    ttl_seconds: u64,
) -> Response {
    if !pow_enabled {
        return Response::new(404, "PoW disabled");
    }
    let challenge = issue_pow_challenge(ip, user_agent, difficulty, ttl_seconds);
    let body = serde_json::to_string(&challenge).unwrap();
    Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .header("Cache-Control", "no-store")
        .body(body)
        .build()
}

pub fn handle_pow_verify(req: &Request, ip: &str, pow_enabled: bool) -> Response {
    if !pow_enabled {
        return Response::new(404, "PoW disabled");
    }
    if *req.method() != spin_sdk::http::Method::Post {
        return Response::new(405, "Method Not Allowed");
    }

    let json = match crate::request_validation::parse_json_body(
        req.body(),
        crate::request_validation::MAX_POW_VERIFY_BYTES,
    ) {
        Ok(v) => v,
        Err(e) => {
            record_pow_failure("sequence_violation", ip);
            return Response::new(400, e);
        }
    };
    let seed = match json.get("seed").and_then(|v| v.as_str()) {
        Some(v) => v,
        None => {
            record_pow_failure("missing_seed_nonce", ip);
            return Response::new(400, "Missing seed");
        }
    };
    if !crate::request_validation::validate_seed_token(seed) {
        record_pow_failure("sequence_violation", ip);
        return Response::new(400, "Invalid seed");
    }
    let nonce = match json.get("nonce").and_then(|v| v.as_str()) {
        Some(v) => v,
        None => {
            record_pow_failure("missing_seed_nonce", ip);
            return Response::new(400, "Missing nonce");
        }
    };
    if !crate::request_validation::validate_nonce(nonce) {
        record_pow_failure("missing_seed_nonce", ip);
        return Response::new(400, "Invalid nonce");
    }

    let payload = match parse_seed_token(seed) {
        Ok(p) => p,
        Err(PowSeedTokenError::InvalidOperationEnvelope(
            crate::challenge::operation_envelope::EnvelopeValidationError::MissingOperationId,
        )) => {
            record_sequence_policy_violation(
                crate::runtime::policy_taxonomy::PolicyTransition::SeqOpMissing,
                "missing_operation_id",
            );
            record_pow_failure("sequence_violation", ip);
            return Response::new(400, "Invalid seed");
        }
        Err(PowSeedTokenError::InvalidOperationEnvelope(_)) => {
            record_sequence_policy_violation(
                crate::runtime::policy_taxonomy::PolicyTransition::SeqOpInvalid,
                "invalid_operation_envelope",
            );
            record_pow_failure("sequence_violation", ip);
            return Response::new(400, "Invalid seed");
        }
        Err(_) => {
            record_sequence_policy_violation(
                crate::runtime::policy_taxonomy::PolicyTransition::SeqOpInvalid,
                "invalid_seed_token",
            );
            record_pow_failure("sequence_violation", ip);
            return Response::new(400, "Invalid seed");
        }
    };

    let now = now_ts();
    if now > payload.expires_at {
        record_sequence_policy_violation(
            crate::runtime::policy_taxonomy::PolicyTransition::SeqOpExpired,
            "seed_expired",
        );
        record_pow_failure("expired_replay", ip);
        return Response::new(400, "Seed expired");
    }
    match crate::challenge::operation_envelope::validate_ordering_window(
        payload.flow_id.as_str(),
        payload.step_id.as_str(),
        payload.step_index,
        payload.issued_at,
        payload.expires_at,
        now,
        crate::challenge::operation_envelope::FLOW_JS_VERIFICATION,
        crate::challenge::operation_envelope::STEP_JS_POW_VERIFY,
        crate::challenge::operation_envelope::STEP_INDEX_JS_POW_VERIFY,
        crate::challenge::operation_envelope::MAX_STEP_WINDOW_SECONDS_JS_POW_VERIFY,
    ) {
        Ok(_) => {}
        Err(crate::challenge::operation_envelope::OrderingValidationError::OrderViolation) => {
            record_sequence_policy_violation(
                crate::runtime::policy_taxonomy::PolicyTransition::SeqOrderViolation,
                "invalid_step_order",
            );
            record_pow_failure("sequence_violation", ip);
            return Response::new(400, "Invalid step order");
        }
        Err(crate::challenge::operation_envelope::OrderingValidationError::WindowExceeded) => {
            record_sequence_policy_violation(
                crate::runtime::policy_taxonomy::PolicyTransition::SeqWindowExceeded,
                "sequence_window_exceeded",
            );
            record_pow_failure("expired_replay", ip);
            return Response::new(400, "Seed expired");
        }
    }

    let ua = req
        .header("user-agent")
        .and_then(|value| value.as_str())
        .unwrap_or("");
    if crate::challenge::operation_envelope::validate_request_binding(
        payload.ip_bucket.as_str(),
        payload.ua_bucket.as_str(),
        payload.path_class.as_str(),
        ip,
        ua,
        crate::challenge::operation_envelope::PATH_CLASS_JS_POW_VERIFY,
    )
    .is_err()
    {
        record_sequence_policy_violation(
            crate::runtime::policy_taxonomy::PolicyTransition::SeqBindingMismatch,
            "binding_mismatch",
        );
        record_pow_failure("binding_timing_mismatch", ip);
        return Response::new(400, "Binding mismatch");
    }

    let state_store = try_open_default_store();
    let fallback_store = FallbackInMemoryStore;
    let store_ref: &dyn crate::challenge::KeyValueStore = if let Some(store) = state_store.as_ref()
    {
        store
    } else {
        eprintln!("[pow] kv store unavailable; using in-memory replay/cadence fallback");
        &fallback_store
    };
    let timing_bucket = format!("{}:{}", payload.ip_bucket, payload.ua_bucket);
    match crate::challenge::operation_envelope::validate_timing_primitives(
        store_ref,
        payload.flow_id.as_str(),
        timing_bucket.as_str(),
        payload.issued_at,
        now,
        crate::challenge::operation_envelope::MIN_STEP_LATENCY_SECONDS_JS_POW_VERIFY,
        crate::challenge::operation_envelope::MAX_STEP_LATENCY_SECONDS_JS_POW_VERIFY,
        crate::challenge::operation_envelope::MAX_FLOW_AGE_SECONDS_JS_POW_VERIFY,
        crate::challenge::operation_envelope::TIMING_REGULARITY_WINDOW_JS_POW_VERIFY,
        crate::challenge::operation_envelope::TIMING_REGULARITY_SPREAD_SECONDS_JS_POW_VERIFY,
        crate::challenge::operation_envelope::TIMING_HISTORY_TTL_SECONDS_JS_POW_VERIFY,
    ) {
        Ok(_) => {}
        Err(crate::challenge::operation_envelope::TimingValidationError::TooFast) => {
            record_sequence_policy_violation(
                crate::runtime::policy_taxonomy::PolicyTransition::SeqTimingTooFast,
                "timing_too_fast",
            );
            record_pow_failure("binding_timing_mismatch", ip);
            return Response::new(400, "Proof submitted too quickly");
        }
        Err(crate::challenge::operation_envelope::TimingValidationError::TooRegular) => {
            record_sequence_policy_violation(
                crate::runtime::policy_taxonomy::PolicyTransition::SeqTimingTooRegular,
                "timing_too_regular",
            );
            record_pow_failure("binding_timing_mismatch", ip);
            return Response::new(400, "Suspicious request cadence");
        }
        Err(crate::challenge::operation_envelope::TimingValidationError::TooSlow) => {
            record_sequence_policy_violation(
                crate::runtime::policy_taxonomy::PolicyTransition::SeqTimingTooSlow,
                "timing_too_slow",
            );
            record_pow_failure("binding_timing_mismatch", ip);
            return Response::new(400, "Seed expired");
        }
    }
    match crate::challenge::operation_envelope::validate_operation_replay(
        store_ref,
        payload.flow_id.as_str(),
        payload.operation_id.as_str(),
        now,
        payload.expires_at,
        crate::challenge::operation_envelope::MAX_OPERATION_REPLAY_TTL_SECONDS_JS_POW_VERIFY,
    ) {
        Ok(_) => {}
        Err(crate::challenge::operation_envelope::ReplayValidationError::ReplayDetected) => {
            record_sequence_policy_violation(
                crate::runtime::policy_taxonomy::PolicyTransition::SeqOpReplay,
                "operation_replay_detected",
            );
            record_pow_failure("expired_replay", ip);
            return Response::new(400, "Seed already used");
        }
        Err(crate::challenge::operation_envelope::ReplayValidationError::ExpiredOperation) => {
            record_sequence_policy_violation(
                crate::runtime::policy_taxonomy::PolicyTransition::SeqOpExpired,
                "operation_expired",
            );
            record_pow_failure("expired_replay", ip);
            return Response::new(400, "Seed expired");
        }
    }

    if !verify_pow(seed, nonce, payload.difficulty) {
        record_pow_failure("invalid_proof", ip);
        return Response::new(400, "Invalid proof");
    }

    Response::builder()
        .status(200)
        .header(
            "Set-Cookie",
            crate::signals::js_verification::js_verified_cookie(ip).as_str(),
        )
        .header("Cache-Control", "no-store")
        .body("OK")
        .build()
}

static FALLBACK_POW_STATE_STORE: Lazy<Mutex<HashMap<String, Vec<u8>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

struct FallbackInMemoryStore;

impl crate::challenge::KeyValueStore for FallbackInMemoryStore {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ()> {
        let map = FALLBACK_POW_STATE_STORE.lock().map_err(|_| ())?;
        Ok(map.get(key).cloned())
    }

    fn set(&self, key: &str, value: &[u8]) -> Result<(), ()> {
        let mut map = FALLBACK_POW_STATE_STORE.lock().map_err(|_| ())?;
        map.insert(key.to_string(), value.to_vec());
        Ok(())
    }

    fn delete(&self, key: &str) -> Result<(), ()> {
        let mut map = FALLBACK_POW_STATE_STORE.lock().map_err(|_| ())?;
        map.remove(key);
        Ok(())
    }
}

fn record_sequence_policy_violation(
    transition: crate::runtime::policy_taxonomy::PolicyTransition,
    outcome_context: &str,
) {
    if let Some(store) = try_open_default_store() {
        let policy_match = crate::runtime::policy_taxonomy::resolve_policy_match(transition);
        crate::observability::metrics::record_policy_match(&store, &policy_match);
        crate::admin::log_event(
            &store,
            &crate::admin::EventLogEntry {
                ts: crate::admin::now_ts(),
                event: crate::admin::EventType::Challenge,
                ip: None,
                reason: Some("pow_verify_sequence_violation".to_string()),
                outcome: Some(policy_match.annotate_outcome(outcome_context)),
                admin: None,
            },
        );
    }
}

fn record_pow_failure(reason: &str, ip: &str) {
    if let Some(store) = try_open_default_store() {
        crate::observability::monitoring::record_pow_failure(&store, ip, reason);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        handle_pow_verify, issue_pow_challenge, make_seed_token, parse_seed_token, verify_pow,
        PowPayload, FALLBACK_POW_STATE_STORE,
    };
    use spin_sdk::http::{Method, Request};
    use std::sync::MutexGuard;

    fn setup_pow_test_env() -> MutexGuard<'static, ()> {
        let lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_POW_SECRET", "pow-test-secret");
        std::env::set_var("SHUMA_JS_SECRET", "js-test-secret");
        if let Ok(mut map) = FALLBACK_POW_STATE_STORE.lock() {
            map.clear();
        }
        lock
    }

    fn make_pow_verify_request(seed: &str, nonce: &str, user_agent: &str) -> Request {
        let payload = serde_json::json!({
            "seed": seed,
            "nonce": nonce
        });
        let mut builder = Request::builder();
        builder
            .method(Method::Post)
            .uri("/pow/verify")
            .header("content-type", "application/json")
            .header("user-agent", user_agent)
            .body(serde_json::to_vec(&payload).expect("pow verify payload should serialize"));
        builder.build()
    }

    fn find_valid_nonce(seed: &str, difficulty: u8) -> String {
        for attempt in 0..200_000u64 {
            let nonce = format!("{:x}", attempt);
            if verify_pow(seed, nonce.as_str(), difficulty) {
                return nonce;
            }
        }
        panic!("failed to find nonce for test");
    }

    #[test]
    fn issued_pow_seed_contains_valid_operation_envelope() {
        let _lock = setup_pow_test_env();
        let challenge = issue_pow_challenge("1.2.3.4", "Mozilla/5.0", 12, 60);
        let payload = parse_seed_token(&challenge.seed).expect("issued seed should parse");
        assert_eq!(
            payload.flow_id,
            crate::challenge::operation_envelope::FLOW_JS_VERIFICATION
        );
        assert_eq!(
            payload.step_id,
            crate::challenge::operation_envelope::STEP_JS_POW_VERIFY
        );
        assert_eq!(
            payload.step_index,
            crate::challenge::operation_envelope::STEP_INDEX_JS_POW_VERIFY
        );
        assert_eq!(
            payload.token_version,
            crate::challenge::operation_envelope::TOKEN_VERSION_V1
        );
        assert!(!payload.operation_id.is_empty());
    }

    #[test]
    fn pow_seed_rejects_invalid_operation_envelope() {
        let _lock = setup_pow_test_env();
        let now = crate::admin::now_ts();
        let payload = PowPayload {
            seed_id: "seed-1".to_string(),
            operation_id: "a1b2c3d4".to_string(),
            flow_id: "wrong_flow".to_string(),
            step_id: crate::challenge::operation_envelope::STEP_JS_POW_VERIFY.to_string(),
            step_index: crate::challenge::operation_envelope::STEP_INDEX_JS_POW_VERIFY,
            ip_bucket: crate::signals::ip_identity::bucket_ip("1.2.3.4"),
            ua_bucket: crate::challenge::operation_envelope::user_agent_bucket("Mozilla/5.0"),
            path_class: crate::challenge::operation_envelope::PATH_CLASS_JS_POW_VERIFY.to_string(),
            issued_at: now,
            expires_at: now + 60,
            token_version: crate::challenge::operation_envelope::TOKEN_VERSION_V1,
            difficulty: 12,
        };
        let seed = make_seed_token(&payload);
        assert!(parse_seed_token(seed.as_str()).is_err());
    }

    #[test]
    fn pow_verify_rejects_operation_replay() {
        let _lock = setup_pow_test_env();
        let challenge = issue_pow_challenge("198.51.100.10", "ReplayUA/1.0", 12, 120);
        let mut payload = parse_seed_token(&challenge.seed).expect("seed should parse");
        let now = crate::admin::now_ts();
        payload.issued_at = now.saturating_sub(2);
        payload.expires_at = now + 120;
        let seed = make_seed_token(&payload);
        let nonce = find_valid_nonce(seed.as_str(), payload.difficulty);
        let req = make_pow_verify_request(seed.as_str(), nonce.as_str(), "ReplayUA/1.0");

        let first = handle_pow_verify(&req, "198.51.100.10", true);
        assert_eq!(*first.status(), 200u16);

        let second = handle_pow_verify(&req, "198.51.100.10", true);
        assert_eq!(*second.status(), 400u16);
        assert_eq!(String::from_utf8_lossy(second.body()), "Seed already used");
    }

    #[test]
    fn pow_verify_rejects_too_fast_submission() {
        let _lock = setup_pow_test_env();
        let challenge = issue_pow_challenge("198.51.100.11", "FastUA/1.0", 12, 120);
        let mut payload = parse_seed_token(&challenge.seed).expect("seed should parse");
        let now = crate::admin::now_ts();
        // Keep issued_at in the near future so the timing check is deterministic
        // even when test execution crosses a second boundary.
        payload.issued_at = now.saturating_add(2);
        payload.expires_at = now + 120;
        let seed = make_seed_token(&payload);
        let nonce = find_valid_nonce(seed.as_str(), payload.difficulty);
        let req = make_pow_verify_request(seed.as_str(), nonce.as_str(), "FastUA/1.0");

        let resp = handle_pow_verify(&req, "198.51.100.11", true);
        assert_eq!(*resp.status(), 400u16);
        assert_eq!(
            String::from_utf8_lossy(resp.body()),
            "Proof submitted too quickly"
        );
    }

    #[test]
    fn pow_verify_rejects_too_regular_cadence() {
        let _lock = setup_pow_test_env();
        let ip = "198.51.100.12";
        let ua = "RegularUA/1.0";

        for _ in 0..3 {
            let challenge = issue_pow_challenge(ip, ua, 12, 120);
            let mut payload = parse_seed_token(&challenge.seed).expect("seed should parse");
            let now = crate::admin::now_ts();
            payload.issued_at = now.saturating_sub(2);
            payload.expires_at = now + 120;
            let seed = make_seed_token(&payload);
            let nonce = find_valid_nonce(seed.as_str(), payload.difficulty);
            let req = make_pow_verify_request(seed.as_str(), nonce.as_str(), ua);
            let resp = handle_pow_verify(&req, ip, true);
            assert_eq!(*resp.status(), 200u16);
        }

        let challenge = issue_pow_challenge(ip, ua, 12, 120);
        let mut payload = parse_seed_token(&challenge.seed).expect("seed should parse");
        let now = crate::admin::now_ts();
        payload.issued_at = now.saturating_sub(2);
        payload.expires_at = now + 120;
        let seed = make_seed_token(&payload);
        let nonce = find_valid_nonce(seed.as_str(), payload.difficulty);
        let req = make_pow_verify_request(seed.as_str(), nonce.as_str(), ua);
        let resp = handle_pow_verify(&req, ip, true);
        assert_eq!(*resp.status(), 400u16);
        assert_eq!(
            String::from_utf8_lossy(resp.body()),
            "Suspicious request cadence"
        );
    }
}
