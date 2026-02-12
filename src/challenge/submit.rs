use spin_sdk::http::{Request, Response};

use super::puzzle::{build_puzzle, parse_submission};
use super::render::render_challenge;
use super::token::parse_seed_token;
use super::{challenge_response, KeyValueStore};

const CHALLENGE_FORBIDDEN_BODY: &str = "<html><body><h2 style='color:red;'>Forbidden. Please request a new challenge.</h2><a href='/challenge/puzzle'>Request new challenge.</a></body></html>";
const CHALLENGE_EXPIRED_BODY: &str = "<html><body><h2 style='color:red;'>Expired</h2><a href='/challenge/puzzle'>Request new challenge.</a></body></html>";
const CHALLENGE_INCORRECT_BODY: &str = "<html><body><h2 style='color:red;'>Incorrect.</h2><a href='/challenge/puzzle'>Request new challenge.</a></body></html>";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ChallengeSubmitOutcome {
    Solved,
    Incorrect,
    ExpiredReplay,
    Forbidden,
    InvalidOutput,
}

fn challenge_forbidden_response() -> Response {
    challenge_response(403, CHALLENGE_FORBIDDEN_BODY)
}

fn challenge_expired_response() -> Response {
    challenge_response(403, CHALLENGE_EXPIRED_BODY)
}

fn challenge_incorrect_response() -> Response {
    challenge_response(403, CHALLENGE_INCORRECT_BODY)
}

pub(crate) fn serve_challenge_page(
    req: &Request,
    test_mode: bool,
    transform_count: usize,
) -> Response {
    if !test_mode {
        return challenge_response(404, "Not Found");
    }
    render_challenge(req, transform_count)
}

pub(crate) fn handle_challenge_submit_with_outcome<S: KeyValueStore>(
    store: &S,
    req: &Request,
) -> (Response, ChallengeSubmitOutcome) {
    if crate::input_validation::enforce_body_size(
        req.body(),
        crate::input_validation::MAX_CHALLENGE_FORM_BYTES,
    )
    .is_err()
    {
        return (
            challenge_forbidden_response(),
            ChallengeSubmitOutcome::Forbidden,
        );
    }
    let form = match std::str::from_utf8(req.body()) {
        Ok(v) => v.to_string(),
        Err(_) => {
            return (
                challenge_forbidden_response(),
                ChallengeSubmitOutcome::Forbidden,
            )
        }
    };
    let seed_token = match get_form_field(&form, "seed") {
        Some(v) => v,
        None => {
            return (
                challenge_forbidden_response(),
                ChallengeSubmitOutcome::Forbidden,
            )
        }
    };
    if !crate::input_validation::validate_seed_token(seed_token.as_str()) {
        return (
            challenge_forbidden_response(),
            ChallengeSubmitOutcome::Forbidden,
        );
    }
    let output_raw = match get_form_field(&form, "output") {
        Some(v) => v,
        None => {
            return (
                challenge_response(400, "Invalid output"),
                ChallengeSubmitOutcome::InvalidOutput,
            )
        }
    };
    if output_raw.len() > 128 {
        return (
            challenge_response(400, "Invalid output"),
            ChallengeSubmitOutcome::InvalidOutput,
        );
    }
    let seed = match parse_seed_token(&seed_token) {
        Ok(s) => s,
        Err(_) => {
            return (
                challenge_forbidden_response(),
                ChallengeSubmitOutcome::Forbidden,
            )
        }
    };
    let now = crate::admin::now_ts();
    if now > seed.expires_at {
        return (
            challenge_expired_response(),
            ChallengeSubmitOutcome::ExpiredReplay,
        );
    }
    let ip = crate::extract_client_ip(req);
    let ip_bucket = crate::signals::ip::bucket_ip(&ip);
    if seed.ip_bucket != ip_bucket {
        return (
            challenge_forbidden_response(),
            ChallengeSubmitOutcome::Forbidden,
        );
    }
    let used_key = format!("challenge_used:{}", seed.seed_id);
    if let Ok(Some(val)) = store.get(&used_key) {
        if let Ok(stored) = String::from_utf8(val) {
            if let Ok(exp) = stored.parse::<u64>() {
                if now <= exp {
                    return (
                        challenge_expired_response(),
                        ChallengeSubmitOutcome::ExpiredReplay,
                    );
                }
            }
        }
        if let Err(e) = store.delete(&used_key) {
            eprintln!(
                "[challenge] failed to delete stale used marker {}: {:?}",
                used_key, e
            );
        }
    }
    if let Err(e) = store.set(&used_key, seed.expires_at.to_string().as_bytes()) {
        eprintln!(
            "[challenge] failed to persist used marker {}: {:?}",
            used_key, e
        );
    }
    let output = match parse_submission(&output_raw, seed.grid_size as usize) {
        Ok(v) => v,
        Err(_e) => {
            return (
                challenge_response(400, "Invalid output"),
                ChallengeSubmitOutcome::InvalidOutput,
            )
        }
    };
    let puzzle = build_puzzle(&seed);
    if output == puzzle.test_output {
        return (
            challenge_response(
                200,
                "<html><body><h2>Thank you! Challenge complete.</h2></body></html>",
            ),
            ChallengeSubmitOutcome::Solved,
        );
    }
    (
        challenge_incorrect_response(),
        ChallengeSubmitOutcome::Incorrect,
    )
}

#[cfg(test)]
pub fn handle_challenge_submit<S: KeyValueStore>(store: &S, req: &Request) -> Response {
    handle_challenge_submit_with_outcome(store, req).0
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
