pub(crate) const TOKEN_VERSION_V1: u8 = 1;

pub(crate) const FLOW_CHALLENGE_PUZZLE: &str = "challenge_puzzle";
pub(crate) const STEP_CHALLENGE_PUZZLE_SUBMIT: &str = "puzzle_submit";
pub(crate) const PATH_CLASS_CHALLENGE_PUZZLE_SUBMIT: &str = "challenge_puzzle_submit";
pub(crate) const STEP_INDEX_CHALLENGE_PUZZLE_SUBMIT: u8 = 2;
pub(crate) const MAX_STEP_WINDOW_SECONDS_CHALLENGE_PUZZLE: u64 = 300;
pub(crate) const MIN_STEP_LATENCY_SECONDS_CHALLENGE_PUZZLE: u64 = 1;
pub(crate) const MAX_STEP_LATENCY_SECONDS_CHALLENGE_PUZZLE: u64 = 900;
pub(crate) const MAX_FLOW_AGE_SECONDS_CHALLENGE_PUZZLE: u64 = 900;
pub(crate) const TIMING_REGULARITY_WINDOW_CHALLENGE_PUZZLE: usize = 4;
pub(crate) const TIMING_REGULARITY_SPREAD_SECONDS_CHALLENGE_PUZZLE: u64 = 1;
pub(crate) const TIMING_HISTORY_TTL_SECONDS_CHALLENGE_PUZZLE: u64 = 1800;
pub(crate) const MAX_OPERATION_REPLAY_TTL_SECONDS_CHALLENGE_PUZZLE: u64 = 900;

pub(crate) const FLOW_JS_VERIFICATION: &str = "js_verification";
pub(crate) const STEP_JS_POW_VERIFY: &str = "pow_verify";
pub(crate) const PATH_CLASS_JS_POW_VERIFY: &str = "pow_verify";
pub(crate) const STEP_INDEX_JS_POW_VERIFY: u8 = 2;
pub(crate) const MAX_STEP_WINDOW_SECONDS_JS_POW_VERIFY: u64 = 300;
pub(crate) const MIN_STEP_LATENCY_SECONDS_JS_POW_VERIFY: u64 = 1;
pub(crate) const MAX_STEP_LATENCY_SECONDS_JS_POW_VERIFY: u64 = 600;
pub(crate) const MAX_FLOW_AGE_SECONDS_JS_POW_VERIFY: u64 = 600;
pub(crate) const TIMING_REGULARITY_WINDOW_JS_POW_VERIFY: usize = 4;
pub(crate) const TIMING_REGULARITY_SPREAD_SECONDS_JS_POW_VERIFY: u64 = 1;
pub(crate) const TIMING_HISTORY_TTL_SECONDS_JS_POW_VERIFY: u64 = 1200;
pub(crate) const MAX_OPERATION_REPLAY_TTL_SECONDS_JS_POW_VERIFY: u64 = 600;

const MAX_OPERATION_ID_LEN: usize = 64;
const UA_BUCKET_HEX_LEN: usize = 16;
const CADENCE_KEY_PREFIX: &str = "seq:cadence";
const OP_REPLAY_KEY_PREFIX: &str = "seq:op_seen";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EnvelopeValidationError {
    MissingOperationId,
    InvalidOperationId,
    InvalidFlowId,
    InvalidStepId,
    InvalidTokenVersion,
    InvalidIssuedWindow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BindingValidationError {
    IpBucketMismatch,
    UaBucketMismatch,
    PathClassMismatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OrderingValidationError {
    OrderViolation,
    WindowExceeded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TimingValidationError {
    TooFast,
    TooRegular,
    TooSlow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReplayValidationError {
    ReplayDetected,
    ExpiredOperation,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct CadenceHistoryState {
    expires_at: u64,
    latencies: Vec<u64>,
}

pub(crate) fn validate_signed_operation_envelope(
    operation_id: &str,
    flow_id: &str,
    step_id: &str,
    issued_at: u64,
    expires_at: u64,
    token_version: u8,
    expected_flow_id: &str,
    expected_step_id: &str,
) -> Result<(), EnvelopeValidationError> {
    if operation_id.trim().is_empty() {
        return Err(EnvelopeValidationError::MissingOperationId);
    }
    if !is_valid_operation_id(operation_id) {
        return Err(EnvelopeValidationError::InvalidOperationId);
    }
    if flow_id != expected_flow_id {
        return Err(EnvelopeValidationError::InvalidFlowId);
    }
    if step_id != expected_step_id {
        return Err(EnvelopeValidationError::InvalidStepId);
    }
    if token_version != TOKEN_VERSION_V1 {
        return Err(EnvelopeValidationError::InvalidTokenVersion);
    }
    if issued_at > expires_at {
        return Err(EnvelopeValidationError::InvalidIssuedWindow);
    }
    Ok(())
}

pub(crate) fn user_agent_bucket(user_agent: &str) -> String {
    use sha2::{Digest, Sha256};

    let normalized = if user_agent.trim().is_empty() {
        "unknown"
    } else {
        user_agent.trim()
    };
    let digest = Sha256::digest(normalized.as_bytes());
    let mut out = String::with_capacity(UA_BUCKET_HEX_LEN);
    for byte in digest {
        if out.len() >= UA_BUCKET_HEX_LEN {
            break;
        }
        out.push(hex_nibble((byte >> 4) & 0x0f));
        if out.len() >= UA_BUCKET_HEX_LEN {
            break;
        }
        out.push(hex_nibble(byte & 0x0f));
    }
    out
}

pub(crate) fn validate_request_binding(
    expected_ip_bucket: &str,
    expected_ua_bucket: &str,
    expected_path_class: &str,
    request_ip: &str,
    request_user_agent: &str,
    request_path_class: &str,
) -> Result<(), BindingValidationError> {
    let ip_bucket = crate::signals::ip_identity::bucket_ip(request_ip);
    if expected_ip_bucket != ip_bucket {
        return Err(BindingValidationError::IpBucketMismatch);
    }

    let ua_bucket = user_agent_bucket(request_user_agent);
    if expected_ua_bucket != ua_bucket {
        return Err(BindingValidationError::UaBucketMismatch);
    }

    if expected_path_class != request_path_class {
        return Err(BindingValidationError::PathClassMismatch);
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn validate_ordering_window(
    flow_id: &str,
    step_id: &str,
    step_index: u8,
    issued_at: u64,
    expires_at: u64,
    now: u64,
    expected_flow_id: &str,
    expected_step_id: &str,
    expected_step_index: u8,
    max_step_window_seconds: u64,
) -> Result<(), OrderingValidationError> {
    if flow_id != expected_flow_id
        || step_id != expected_step_id
        || step_index != expected_step_index
    {
        return Err(OrderingValidationError::OrderViolation);
    }
    let step_window_end = std::cmp::min(
        expires_at,
        issued_at.saturating_add(max_step_window_seconds),
    );
    if now > step_window_end {
        return Err(OrderingValidationError::WindowExceeded);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn validate_timing_primitives<S: crate::challenge::KeyValueStore + ?Sized>(
    store: &S,
    flow_id: &str,
    timing_bucket: &str,
    issued_at: u64,
    now: u64,
    min_step_latency_seconds: u64,
    max_step_latency_seconds: u64,
    max_flow_age_seconds: u64,
    cadence_window_size: usize,
    cadence_spread_threshold_seconds: u64,
    cadence_history_ttl_seconds: u64,
) -> Result<(), TimingValidationError> {
    if now < issued_at {
        return Err(TimingValidationError::TooFast);
    }
    let latency_seconds = now.saturating_sub(issued_at);
    if latency_seconds < min_step_latency_seconds {
        return Err(TimingValidationError::TooFast);
    }
    if latency_seconds > max_step_latency_seconds || latency_seconds > max_flow_age_seconds {
        return Err(TimingValidationError::TooSlow);
    }

    if cadence_window_size < 2
        || cadence_history_ttl_seconds == 0
        || timing_bucket.trim().is_empty()
    {
        return Ok(());
    }

    let key = cadence_state_key(flow_id, timing_bucket);
    let mut latencies = load_cadence_latencies(store, key.as_str(), now);
    latencies.push(latency_seconds);
    while latencies.len() > cadence_window_size {
        latencies.remove(0);
    }

    let is_too_regular = if latencies.len() >= cadence_window_size {
        let min_latency = latencies.iter().copied().min().unwrap_or(latency_seconds);
        let max_latency = latencies.iter().copied().max().unwrap_or(latency_seconds);
        max_latency.saturating_sub(min_latency) <= cadence_spread_threshold_seconds
    } else {
        false
    };

    let state = CadenceHistoryState {
        expires_at: now.saturating_add(cadence_history_ttl_seconds),
        latencies,
    };
    if let Ok(raw) = serde_json::to_vec(&state) {
        if store.set(key.as_str(), raw.as_slice()).is_err() {
            eprintln!("[sequence] failed to persist cadence state for key {}", key);
        }
    }

    if is_too_regular {
        return Err(TimingValidationError::TooRegular);
    }

    Ok(())
}

pub(crate) fn validate_operation_replay<S: crate::challenge::KeyValueStore + ?Sized>(
    store: &S,
    flow_id: &str,
    operation_id: &str,
    now: u64,
    expires_at: u64,
    max_replay_ttl_seconds: u64,
) -> Result<(), ReplayValidationError> {
    if now > expires_at {
        return Err(ReplayValidationError::ExpiredOperation);
    }

    let replay_key = operation_replay_key(flow_id, operation_id);
    if let Ok(Some(raw)) = store.get(replay_key.as_str()) {
        if let Ok(stored) = String::from_utf8(raw) {
            if let Ok(seen_until) = stored.parse::<u64>() {
                if now <= seen_until {
                    return Err(ReplayValidationError::ReplayDetected);
                }
            }
        }
        if store.delete(replay_key.as_str()).is_err() {
            eprintln!(
                "[sequence] failed to delete stale replay marker {}",
                replay_key
            );
        }
    }

    let track_until = std::cmp::min(expires_at, now.saturating_add(max_replay_ttl_seconds));
    if track_until <= now {
        return Err(ReplayValidationError::ExpiredOperation);
    }
    if store
        .set(replay_key.as_str(), track_until.to_string().as_bytes())
        .is_err()
    {
        eprintln!("[sequence] failed to persist replay marker {}", replay_key);
    }
    Ok(())
}

fn cadence_state_key(flow_id: &str, timing_bucket: &str) -> String {
    format!("{}:{}:{}", CADENCE_KEY_PREFIX, flow_id, timing_bucket)
}

fn operation_replay_key(flow_id: &str, operation_id: &str) -> String {
    format!("{}:{}:{}", OP_REPLAY_KEY_PREFIX, flow_id, operation_id)
}

fn load_cadence_latencies<S: crate::challenge::KeyValueStore + ?Sized>(
    store: &S,
    key: &str,
    now: u64,
) -> Vec<u64> {
    let Ok(Some(raw)) = store.get(key) else {
        return Vec::new();
    };
    let Ok(state) = serde_json::from_slice::<CadenceHistoryState>(raw.as_slice()) else {
        return Vec::new();
    };
    if now > state.expires_at {
        return Vec::new();
    }
    state.latencies
}

fn is_valid_operation_id(operation_id: &str) -> bool {
    if operation_id.len() > MAX_OPERATION_ID_LEN {
        return false;
    }
    operation_id
        .chars()
        .all(|ch| ch.is_ascii_hexdigit() || ch == '_' || ch == '-' || ch == ':')
}

fn hex_nibble(nibble: u8) -> char {
    match nibble {
        0..=9 => (b'0' + nibble) as char,
        _ => (b'a' + (nibble - 10)) as char,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        user_agent_bucket, validate_operation_replay, validate_ordering_window,
        validate_request_binding, validate_signed_operation_envelope, validate_timing_primitives,
        BindingValidationError, EnvelopeValidationError, OrderingValidationError,
        ReplayValidationError, TimingValidationError, FLOW_CHALLENGE_PUZZLE,
        PATH_CLASS_CHALLENGE_PUZZLE_SUBMIT, STEP_CHALLENGE_PUZZLE_SUBMIT,
        STEP_INDEX_CHALLENGE_PUZZLE_SUBMIT, TOKEN_VERSION_V1,
    };
    use std::cell::RefCell;
    use std::collections::HashMap;

    #[derive(Default)]
    struct TestStore {
        map: RefCell<HashMap<String, Vec<u8>>>,
    }

    impl crate::challenge::KeyValueStore for TestStore {
        fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ()> {
            Ok(self.map.borrow().get(key).cloned())
        }

        fn set(&self, key: &str, value: &[u8]) -> Result<(), ()> {
            self.map
                .borrow_mut()
                .insert(key.to_string(), value.to_vec());
            Ok(())
        }

        fn delete(&self, key: &str) -> Result<(), ()> {
            self.map.borrow_mut().remove(key);
            Ok(())
        }
    }

    #[test]
    fn signed_operation_envelope_accepts_valid_values() {
        let result = validate_signed_operation_envelope(
            "a1b2c3d4e5f60718",
            FLOW_CHALLENGE_PUZZLE,
            STEP_CHALLENGE_PUZZLE_SUBMIT,
            100,
            200,
            TOKEN_VERSION_V1,
            FLOW_CHALLENGE_PUZZLE,
            STEP_CHALLENGE_PUZZLE_SUBMIT,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn signed_operation_envelope_rejects_invalid_operation_id() {
        let err = validate_signed_operation_envelope(
            "bad op id",
            FLOW_CHALLENGE_PUZZLE,
            STEP_CHALLENGE_PUZZLE_SUBMIT,
            100,
            200,
            TOKEN_VERSION_V1,
            FLOW_CHALLENGE_PUZZLE,
            STEP_CHALLENGE_PUZZLE_SUBMIT,
        )
        .unwrap_err();
        assert_eq!(err, EnvelopeValidationError::InvalidOperationId);
    }

    #[test]
    fn signed_operation_envelope_rejects_mismatched_flow_or_step() {
        let flow_err = validate_signed_operation_envelope(
            "a1b2",
            "wrong_flow",
            STEP_CHALLENGE_PUZZLE_SUBMIT,
            100,
            200,
            TOKEN_VERSION_V1,
            FLOW_CHALLENGE_PUZZLE,
            STEP_CHALLENGE_PUZZLE_SUBMIT,
        )
        .unwrap_err();
        assert_eq!(flow_err, EnvelopeValidationError::InvalidFlowId);

        let step_err = validate_signed_operation_envelope(
            "a1b2",
            FLOW_CHALLENGE_PUZZLE,
            "wrong_step",
            100,
            200,
            TOKEN_VERSION_V1,
            FLOW_CHALLENGE_PUZZLE,
            STEP_CHALLENGE_PUZZLE_SUBMIT,
        )
        .unwrap_err();
        assert_eq!(step_err, EnvelopeValidationError::InvalidStepId);
    }

    #[test]
    fn signed_operation_envelope_rejects_invalid_version_or_window() {
        let version_err = validate_signed_operation_envelope(
            "a1b2",
            FLOW_CHALLENGE_PUZZLE,
            STEP_CHALLENGE_PUZZLE_SUBMIT,
            100,
            200,
            2,
            FLOW_CHALLENGE_PUZZLE,
            STEP_CHALLENGE_PUZZLE_SUBMIT,
        )
        .unwrap_err();
        assert_eq!(version_err, EnvelopeValidationError::InvalidTokenVersion);

        let window_err = validate_signed_operation_envelope(
            "a1b2",
            FLOW_CHALLENGE_PUZZLE,
            STEP_CHALLENGE_PUZZLE_SUBMIT,
            201,
            200,
            TOKEN_VERSION_V1,
            FLOW_CHALLENGE_PUZZLE,
            STEP_CHALLENGE_PUZZLE_SUBMIT,
        )
        .unwrap_err();
        assert_eq!(window_err, EnvelopeValidationError::InvalidIssuedWindow);
    }

    #[test]
    fn user_agent_bucket_is_stable_and_non_empty() {
        let a = user_agent_bucket("Mozilla/5.0");
        let b = user_agent_bucket("Mozilla/5.0");
        let c = user_agent_bucket("curl/8.0");
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(a.len(), 16);
    }

    #[test]
    fn request_binding_rejects_ip_ua_and_path_mismatches() {
        let expected_ip_bucket = crate::signals::ip_identity::bucket_ip("1.2.3.4");
        let expected_ua_bucket = user_agent_bucket("Mozilla/5.0");

        let ip_err = validate_request_binding(
            expected_ip_bucket.as_str(),
            expected_ua_bucket.as_str(),
            PATH_CLASS_CHALLENGE_PUZZLE_SUBMIT,
            "5.6.7.8",
            "Mozilla/5.0",
            PATH_CLASS_CHALLENGE_PUZZLE_SUBMIT,
        )
        .unwrap_err();
        assert_eq!(ip_err, BindingValidationError::IpBucketMismatch);

        let ua_err = validate_request_binding(
            expected_ip_bucket.as_str(),
            expected_ua_bucket.as_str(),
            PATH_CLASS_CHALLENGE_PUZZLE_SUBMIT,
            "1.2.3.4",
            "curl/8.0",
            PATH_CLASS_CHALLENGE_PUZZLE_SUBMIT,
        )
        .unwrap_err();
        assert_eq!(ua_err, BindingValidationError::UaBucketMismatch);

        let path_err = validate_request_binding(
            expected_ip_bucket.as_str(),
            expected_ua_bucket.as_str(),
            PATH_CLASS_CHALLENGE_PUZZLE_SUBMIT,
            "1.2.3.4",
            "Mozilla/5.0",
            "wrong_path_class",
        )
        .unwrap_err();
        assert_eq!(path_err, BindingValidationError::PathClassMismatch);
    }

    #[test]
    fn ordering_window_rejects_order_and_window_violations() {
        let order_err = validate_ordering_window(
            FLOW_CHALLENGE_PUZZLE,
            STEP_CHALLENGE_PUZZLE_SUBMIT,
            1,
            100,
            400,
            150,
            FLOW_CHALLENGE_PUZZLE,
            STEP_CHALLENGE_PUZZLE_SUBMIT,
            STEP_INDEX_CHALLENGE_PUZZLE_SUBMIT,
            300,
        )
        .unwrap_err();
        assert_eq!(order_err, OrderingValidationError::OrderViolation);

        let window_err = validate_ordering_window(
            FLOW_CHALLENGE_PUZZLE,
            STEP_CHALLENGE_PUZZLE_SUBMIT,
            STEP_INDEX_CHALLENGE_PUZZLE_SUBMIT,
            100,
            600,
            450,
            FLOW_CHALLENGE_PUZZLE,
            STEP_CHALLENGE_PUZZLE_SUBMIT,
            STEP_INDEX_CHALLENGE_PUZZLE_SUBMIT,
            300,
        )
        .unwrap_err();
        assert_eq!(window_err, OrderingValidationError::WindowExceeded);
    }

    #[test]
    fn ordering_window_accepts_valid_step_progression() {
        let result = validate_ordering_window(
            FLOW_CHALLENGE_PUZZLE,
            STEP_CHALLENGE_PUZZLE_SUBMIT,
            STEP_INDEX_CHALLENGE_PUZZLE_SUBMIT,
            100,
            400,
            150,
            FLOW_CHALLENGE_PUZZLE,
            STEP_CHALLENGE_PUZZLE_SUBMIT,
            STEP_INDEX_CHALLENGE_PUZZLE_SUBMIT,
            300,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn replay_primitive_rejects_duplicate_and_expired_operations() {
        let store = TestStore::default();
        let first =
            validate_operation_replay(&store, FLOW_CHALLENGE_PUZZLE, "abc123", 100, 300, 300);
        assert!(first.is_ok());

        let replay_err =
            validate_operation_replay(&store, FLOW_CHALLENGE_PUZZLE, "abc123", 101, 300, 300)
                .unwrap_err();
        assert_eq!(replay_err, ReplayValidationError::ReplayDetected);

        let expired_err =
            validate_operation_replay(&store, FLOW_CHALLENGE_PUZZLE, "xyz999", 400, 300, 300)
                .unwrap_err();
        assert_eq!(expired_err, ReplayValidationError::ExpiredOperation);
    }

    #[test]
    fn timing_primitives_reject_fast_regular_and_slow_sequences() {
        let store = TestStore::default();

        let fast_err = validate_timing_primitives(
            &store,
            FLOW_CHALLENGE_PUZZLE,
            "ip_bucket",
            100,
            100,
            1,
            900,
            900,
            4,
            1,
            1800,
        )
        .unwrap_err();
        assert_eq!(fast_err, TimingValidationError::TooFast);

        let slow_err = validate_timing_primitives(
            &store,
            FLOW_CHALLENGE_PUZZLE,
            "ip_bucket",
            100,
            1200,
            1,
            900,
            900,
            4,
            1,
            1800,
        )
        .unwrap_err();
        assert_eq!(slow_err, TimingValidationError::TooSlow);

        for _ in 0..3 {
            let ok = validate_timing_primitives(
                &store,
                FLOW_CHALLENGE_PUZZLE,
                "regular_bucket",
                100,
                110,
                1,
                900,
                900,
                4,
                1,
                1800,
            );
            assert!(ok.is_ok());
        }
        let regular_err = validate_timing_primitives(
            &store,
            FLOW_CHALLENGE_PUZZLE,
            "regular_bucket",
            100,
            110,
            1,
            900,
            900,
            4,
            1,
            1800,
        )
        .unwrap_err();
        assert_eq!(regular_err, TimingValidationError::TooRegular);
    }

    #[test]
    fn timing_primitives_allow_human_like_variability() {
        let store = TestStore::default();
        let attempts = [110_u64, 118_u64, 130_u64, 149_u64];
        for now in attempts {
            let result = validate_timing_primitives(
                &store,
                FLOW_CHALLENGE_PUZZLE,
                "human_bucket",
                100,
                now,
                1,
                900,
                900,
                4,
                1,
                1800,
            );
            assert!(result.is_ok());
        }
    }
}
