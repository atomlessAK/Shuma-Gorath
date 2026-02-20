use spin_sdk::http::Request;

use super::render::normalize_return_to;
use super::token::{marker_cookie_value, parse_seed_token, SeedTokenError};
use super::types::{
    NotABotDecision, NotABotSubmitOutcome, NotABotSubmitResult, NotABotTelemetry,
};

const NOT_A_BOT_INTERACTION_MIN_MS: u32 = 250;
const NOT_A_BOT_INTERACTION_MAX_MS: u32 = 180_000;
const NOT_A_BOT_DOWN_UP_MIN_MS: u32 = 25;
const NOT_A_BOT_DOWN_UP_MAX_MS: u32 = 12_000;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct AttemptState {
    window_start: u64,
    count: u32,
}

pub(crate) fn serve_not_a_bot_page(
    req: &Request,
    test_mode: bool,
    cfg: &crate::config::Config,
) -> spin_sdk::http::Response {
    if !test_mode {
        return crate::challenge::challenge_response(404, "Not Found");
    }
    super::render::render_not_a_bot(req, cfg)
}

pub(crate) fn handle_not_a_bot_submit_with_outcome<S: crate::challenge::KeyValueStore>(
    store: &S,
    req: &Request,
    cfg: &crate::config::Config,
) -> NotABotSubmitResult {
    let ip = crate::extract_client_ip(req);
    let ua = req
        .header("user-agent")
        .and_then(|value| value.as_str())
        .unwrap_or("");
    let request_ip_bucket = crate::signals::ip_identity::bucket_ip(ip.as_str());
    let now = crate::admin::now_ts();

    if increment_and_check_attempt_limit(
        store,
        request_ip_bucket.as_str(),
        now,
        cfg.not_a_bot_attempt_window_seconds,
        cfg.not_a_bot_attempt_limit_per_window,
    ) {
        return failure_result(
            NotABotSubmitOutcome::AttemptLimitExceeded,
            NotABotDecision::MazeOrBlock,
            "/",
        );
    }

    if crate::request_validation::enforce_body_size(
        req.body(),
        crate::request_validation::MAX_CHALLENGE_FORM_BYTES,
    )
    .is_err()
    {
        return failure_result(
            NotABotSubmitOutcome::InvalidTelemetry,
            NotABotDecision::MazeOrBlock,
            "/",
        );
    }

    let form = match std::str::from_utf8(req.body()) {
        Ok(value) => value,
        Err(_) => {
            return failure_result(
                NotABotSubmitOutcome::InvalidTelemetry,
                NotABotDecision::MazeOrBlock,
                "/",
            )
        }
    };

    let seed_token = match get_form_field(form, "seed") {
        Some(value) if crate::request_validation::validate_seed_token(value.as_str()) => value,
        Some(_) => {
            return failure_result(
                NotABotSubmitOutcome::InvalidSeed,
                NotABotDecision::MazeOrBlock,
                "/",
            )
        }
        None => {
            return failure_result(
                NotABotSubmitOutcome::MissingSeed,
                NotABotDecision::MazeOrBlock,
                "/",
            )
        }
    };

    let seed = match parse_seed_token(seed_token.as_str()) {
        Ok(value) => value,
        Err(SeedTokenError::InvalidOperationEnvelope(
            crate::challenge::operation_envelope::EnvelopeValidationError::MissingOperationId,
        )) => {
            return failure_result(
                NotABotSubmitOutcome::SequenceViolation,
                NotABotDecision::MazeOrBlock,
                "/",
            )
        }
        Err(SeedTokenError::InvalidOperationEnvelope(_)) => {
            return failure_result(
                NotABotSubmitOutcome::SequenceViolation,
                NotABotDecision::MazeOrBlock,
                "/",
            )
        }
        Err(SeedTokenError::SignatureMismatch | SeedTokenError::InvalidPayloadJson) => {
            return failure_result(
                NotABotSubmitOutcome::InvalidSeed,
                NotABotDecision::MazeOrBlock,
                "/",
            )
        }
        Err(_) => {
            return failure_result(
                NotABotSubmitOutcome::InvalidSeed,
                NotABotDecision::MazeOrBlock,
                "/",
            )
        }
    };

    let return_to = normalize_return_to(seed.return_to.as_str());

    if now > seed.expires_at {
        return failure_result(
            NotABotSubmitOutcome::Expired,
            NotABotDecision::MazeOrBlock,
            return_to.as_str(),
        );
    }

    match crate::challenge::operation_envelope::validate_ordering_window(
        seed.flow_id.as_str(),
        seed.step_id.as_str(),
        seed.step_index,
        seed.issued_at,
        seed.expires_at,
        now,
        crate::challenge::operation_envelope::FLOW_NOT_A_BOT,
        crate::challenge::operation_envelope::STEP_NOT_A_BOT_SUBMIT,
        crate::challenge::operation_envelope::STEP_INDEX_NOT_A_BOT_SUBMIT,
        crate::challenge::operation_envelope::MAX_STEP_WINDOW_SECONDS_NOT_A_BOT,
    ) {
        Ok(_) => {}
        Err(_) => {
            return failure_result(
                NotABotSubmitOutcome::SequenceViolation,
                NotABotDecision::MazeOrBlock,
                return_to.as_str(),
            )
        }
    }

    if crate::challenge::operation_envelope::validate_request_binding(
        seed.ip_bucket.as_str(),
        seed.ua_bucket.as_str(),
        seed.path_class.as_str(),
        ip.as_str(),
        ua,
        crate::challenge::operation_envelope::PATH_CLASS_NOT_A_BOT_SUBMIT,
    )
    .is_err()
    {
        return failure_result(
            NotABotSubmitOutcome::BindingMismatch,
            NotABotDecision::MazeOrBlock,
            return_to.as_str(),
        );
    }

    let timing_bucket = format!("{}:{}", seed.ip_bucket, seed.ua_bucket);
    match crate::challenge::operation_envelope::validate_timing_primitives(
        store,
        seed.flow_id.as_str(),
        timing_bucket.as_str(),
        seed.issued_at,
        now,
        crate::challenge::operation_envelope::MIN_STEP_LATENCY_SECONDS_NOT_A_BOT,
        crate::challenge::operation_envelope::MAX_STEP_LATENCY_SECONDS_NOT_A_BOT,
        crate::challenge::operation_envelope::MAX_FLOW_AGE_SECONDS_NOT_A_BOT,
        crate::challenge::operation_envelope::TIMING_REGULARITY_WINDOW_NOT_A_BOT,
        crate::challenge::operation_envelope::TIMING_REGULARITY_SPREAD_SECONDS_NOT_A_BOT,
        crate::challenge::operation_envelope::TIMING_HISTORY_TTL_SECONDS_NOT_A_BOT,
    ) {
        Ok(_) => {}
        Err(_) => {
            return failure_result(
                NotABotSubmitOutcome::SequenceViolation,
                NotABotDecision::MazeOrBlock,
                return_to.as_str(),
            )
        }
    }

    match crate::challenge::operation_envelope::validate_operation_replay(
        store,
        seed.flow_id.as_str(),
        seed.operation_id.as_str(),
        now,
        seed.expires_at,
        crate::challenge::operation_envelope::MAX_OPERATION_REPLAY_TTL_SECONDS_NOT_A_BOT,
    ) {
        Ok(_) => {}
        Err(crate::challenge::operation_envelope::ReplayValidationError::ReplayDetected) => {
            return failure_result(
                NotABotSubmitOutcome::Replay,
                NotABotDecision::MazeOrBlock,
                return_to.as_str(),
            )
        }
        Err(crate::challenge::operation_envelope::ReplayValidationError::ExpiredOperation) => {
            return failure_result(
                NotABotSubmitOutcome::Expired,
                NotABotDecision::MazeOrBlock,
                return_to.as_str(),
            )
        }
    }

    let checked = parse_checked(get_form_field(form, "checked"));
    let telemetry = match get_form_field(form, "telemetry")
        .and_then(|value| serde_json::from_str::<NotABotTelemetry>(value.as_str()).ok())
    {
        Some(value) => value,
        None => {
            return failure_result(
                NotABotSubmitOutcome::InvalidTelemetry,
                NotABotDecision::MazeOrBlock,
                return_to.as_str(),
            )
        }
    };

    if !validate_telemetry_ranges(&telemetry) {
        return failure_result(
            NotABotSubmitOutcome::InvalidTelemetry,
            NotABotDecision::MazeOrBlock,
            return_to.as_str(),
        );
    }

    let Some(score) = compute_not_a_bot_score(checked, &telemetry) else {
        return failure_result(
            NotABotSubmitOutcome::MazeOrBlock,
            NotABotDecision::MazeOrBlock,
            return_to.as_str(),
        );
    };

    if score >= cfg.not_a_bot_score_pass_min {
        return NotABotSubmitResult {
            outcome: NotABotSubmitOutcome::Pass,
            decision: NotABotDecision::Pass,
            return_to,
            marker_cookie: Some(marker_cookie_value(
                seed.ip_bucket.as_str(),
                seed.ua_bucket.as_str(),
                cfg.not_a_bot_marker_ttl_seconds,
            )),
            solve_ms: Some(telemetry.interaction_elapsed_ms as u64),
        };
    }

    if score >= cfg.not_a_bot_score_escalate_min {
        return NotABotSubmitResult {
            outcome: NotABotSubmitOutcome::EscalatePuzzle,
            decision: NotABotDecision::EscalatePuzzle,
            return_to,
            marker_cookie: None,
            solve_ms: Some(telemetry.interaction_elapsed_ms as u64),
        };
    }

    failure_result(
        NotABotSubmitOutcome::MazeOrBlock,
        NotABotDecision::MazeOrBlock,
        return_to.as_str(),
    )
}

fn increment_and_check_attempt_limit<S: crate::challenge::KeyValueStore>(
    store: &S,
    ip_bucket: &str,
    now: u64,
    attempt_window_seconds: u64,
    attempt_limit_per_window: u32,
) -> bool {
    let key = format!("not_a_bot:attempt:{}", ip_bucket);
    let mut state = store
        .get(key.as_str())
        .ok()
        .flatten()
        .and_then(|raw| serde_json::from_slice::<AttemptState>(raw.as_slice()).ok())
        .unwrap_or(AttemptState {
            window_start: now,
            count: 0,
        });

    if now.saturating_sub(state.window_start) >= attempt_window_seconds {
        state.window_start = now;
        state.count = 0;
    }
    state.count = state.count.saturating_add(1);

    if let Ok(encoded) = serde_json::to_vec(&state) {
        if let Err(err) = store.set(key.as_str(), encoded.as_slice()) {
            eprintln!(
                "[not-a-bot] failed to persist attempt counter for {}: {:?}",
                key, err
            );
        }
    }

    state.count > attempt_limit_per_window
}

fn parse_checked(value: Option<String>) -> bool {
    match value {
        Some(raw) => matches!(
            raw.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        ),
        None => false,
    }
}

fn validate_telemetry_ranges(telemetry: &NotABotTelemetry) -> bool {
    if telemetry.pointer_move_count > 60_000 {
        return false;
    }
    if !(0.0..=100_000.0).contains(&telemetry.pointer_path_length) {
        return false;
    }
    if telemetry.pointer_direction_changes > 60_000 {
        return false;
    }
    if telemetry.down_up_ms > 600_000 {
        return false;
    }
    if telemetry.interaction_elapsed_ms > 600_000 {
        return false;
    }
    if telemetry.activation_count > 10 {
        return false;
    }
    if parse_activation_method(telemetry.activation_method.as_str()).is_none() {
        return false;
    }
    true
}

fn parse_activation_method(value: &str) -> Option<&'static str> {
    match value.trim().to_ascii_lowercase().as_str() {
        "pointer" => Some("pointer"),
        "touch" => Some("touch"),
        "keyboard" => Some("keyboard"),
        "unknown" | "" => Some("unknown"),
        _ => None,
    }
}

fn compute_not_a_bot_score(checked: bool, telemetry: &NotABotTelemetry) -> Option<u8> {
    if !checked || !telemetry.events_order_valid || !telemetry.activation_trusted {
        return None;
    }
    let activation_method = parse_activation_method(telemetry.activation_method.as_str())?;
    if telemetry.activation_count == 0 || telemetry.activation_count > 2 {
        return None;
    }
    if telemetry.interaction_elapsed_ms < NOT_A_BOT_INTERACTION_MIN_MS
        || telemetry.interaction_elapsed_ms > NOT_A_BOT_INTERACTION_MAX_MS
    {
        return None;
    }
    if telemetry.down_up_ms > 0
        && (telemetry.down_up_ms < NOT_A_BOT_DOWN_UP_MIN_MS
            || telemetry.down_up_ms > NOT_A_BOT_DOWN_UP_MAX_MS)
    {
        return None;
    }

    let mut score = 1u8;

    if telemetry.interaction_elapsed_ms >= 900 {
        score = score.saturating_add(2);
    } else if telemetry.interaction_elapsed_ms >= 500 {
        score = score.saturating_add(1);
    }

    if telemetry.down_up_ms >= 80 && telemetry.down_up_ms <= 5000 {
        score = score.saturating_add(1);
    }

    let plausible_pointer_motion = telemetry.pointer_move_count >= 2
        && telemetry.pointer_move_count <= 3000
        && telemetry.pointer_path_length >= 8.0
        && telemetry.pointer_path_length <= 80_000.0
        && telemetry.pointer_direction_changes >= 1
        && telemetry.pointer_direction_changes <= 3000;

    match activation_method {
        "pointer" => {
            if !telemetry.has_pointer {
                return None;
            }
            if plausible_pointer_motion {
                score = score.saturating_add(3);
            } else if telemetry.interaction_elapsed_ms >= 1200 {
                score = score.saturating_add(1);
            }
        }
        "touch" => {
            if !telemetry.touch_used {
                return None;
            }
            if plausible_pointer_motion || telemetry.interaction_elapsed_ms >= 800 {
                score = score.saturating_add(2);
            }
        }
        "keyboard" => {
            if !telemetry.keyboard_used {
                return None;
            }
            // Keep keyboard-only flows equivalent-strength without requiring pointer motion.
            score = score.saturating_add(if telemetry.control_focused { 3 } else { 2 });
        }
        "unknown" => {
            // Assistive and synthetic browser mediation paths can legitimately hide raw modality.
            // Keep this path pass-capable without granting a strong standalone score bump.
            if telemetry.control_focused && telemetry.interaction_elapsed_ms >= 900 {
                score = score.saturating_add(1);
            }
        }
        _ => return None,
    }

    if telemetry.keyboard_used || telemetry.touch_used || telemetry.has_pointer {
        score = score.saturating_add(1);
    }
    if telemetry.control_focused {
        score = score.saturating_add(1);
    }

    if telemetry.focus_changes <= 3 && telemetry.visibility_changes <= 1 {
        score = score.saturating_add(1);
    }

    Some(score.min(10))
}

fn failure_result(
    outcome: NotABotSubmitOutcome,
    decision: NotABotDecision,
    return_to: &str,
) -> NotABotSubmitResult {
    NotABotSubmitResult {
        outcome,
        decision,
        return_to: normalize_return_to(return_to),
        marker_cookie: None,
        solve_ms: None,
    }
}

fn get_form_field(form: &str, name: &str) -> Option<String> {
    for pair in form.split('&') {
        let mut parts = pair.splitn(2, '=');
        if let (Some(k), Some(v)) = (parts.next(), parts.next()) {
            if k == name {
                return Some(url_decode(v));
            }
        }
    }
    None
}

fn url_decode(s: &str) -> String {
    percent_encoding::percent_decode_str(s)
        .decode_utf8_lossy()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_rejects_unchecked_or_invalid_order() {
        let telemetry = NotABotTelemetry {
            events_order_valid: false,
            interaction_elapsed_ms: 1500,
            activation_method: "pointer".to_string(),
            activation_trusted: true,
            activation_count: 1,
            ..NotABotTelemetry::default()
        };
        assert!(compute_not_a_bot_score(false, &telemetry).is_none());
        assert!(compute_not_a_bot_score(true, &telemetry).is_none());
    }

    #[test]
    fn score_accepts_plausible_interaction() {
        let telemetry = NotABotTelemetry {
            has_pointer: true,
            pointer_move_count: 42,
            pointer_path_length: 560.0,
            pointer_direction_changes: 18,
            down_up_ms: 230,
            focus_changes: 1,
            visibility_changes: 0,
            interaction_elapsed_ms: 1800,
            keyboard_used: false,
            touch_used: false,
            events_order_valid: true,
            activation_method: "pointer".to_string(),
            activation_trusted: true,
            activation_count: 1,
            control_focused: true,
        };
        let score = compute_not_a_bot_score(true, &telemetry).unwrap();
        assert!(score >= 7);
    }

    #[test]
    fn score_accepts_keyboard_only_accessible_interaction() {
        let telemetry = NotABotTelemetry {
            has_pointer: false,
            pointer_move_count: 0,
            pointer_path_length: 0.0,
            pointer_direction_changes: 0,
            down_up_ms: 0,
            focus_changes: 1,
            visibility_changes: 0,
            interaction_elapsed_ms: 1700,
            keyboard_used: true,
            touch_used: false,
            events_order_valid: true,
            activation_method: "keyboard".to_string(),
            activation_trusted: true,
            activation_count: 1,
            control_focused: true,
        };
        let score = compute_not_a_bot_score(true, &telemetry).unwrap();
        assert!(
            score >= 7,
            "keyboard-only flow should remain equivalent-strength and pass-capable"
        );
    }

    #[test]
    fn score_fails_too_fast_or_too_slow_interactions() {
        let telemetry_fast = NotABotTelemetry {
            events_order_valid: true,
            interaction_elapsed_ms: 100,
            activation_method: "pointer".to_string(),
            activation_trusted: true,
            activation_count: 1,
            has_pointer: true,
            ..NotABotTelemetry::default()
        };
        let telemetry_slow = NotABotTelemetry {
            events_order_valid: true,
            interaction_elapsed_ms: 200_000,
            activation_method: "pointer".to_string(),
            activation_trusted: true,
            activation_count: 1,
            has_pointer: true,
            ..NotABotTelemetry::default()
        };
        assert!(compute_not_a_bot_score(true, &telemetry_fast).is_none());
        assert!(compute_not_a_bot_score(true, &telemetry_slow).is_none());
    }

    #[test]
    fn score_rejects_untrusted_activation() {
        let telemetry = NotABotTelemetry {
            has_pointer: true,
            pointer_move_count: 18,
            pointer_path_length: 240.0,
            pointer_direction_changes: 7,
            down_up_ms: 260,
            focus_changes: 1,
            visibility_changes: 0,
            interaction_elapsed_ms: 1500,
            events_order_valid: true,
            activation_method: "pointer".to_string(),
            activation_trusted: false,
            activation_count: 1,
            control_focused: true,
            ..NotABotTelemetry::default()
        };
        assert!(compute_not_a_bot_score(true, &telemetry).is_none());
    }

    #[test]
    fn score_allows_unknown_modality_when_other_human_signals_are_strong() {
        let telemetry = NotABotTelemetry {
            interaction_elapsed_ms: 1600,
            keyboard_used: true,
            events_order_valid: true,
            activation_method: "unknown".to_string(),
            activation_trusted: true,
            activation_count: 1,
            control_focused: true,
            ..NotABotTelemetry::default()
        };
        let score = compute_not_a_bot_score(true, &telemetry).unwrap();
        assert!(
            score >= 7,
            "unknown modality should remain pass-capable for accessibility mediation paths"
        );
    }
}
