use base64::{engine::general_purpose, Engine as _};
use hmac::{Hmac, Mac};
use sha2::Sha256;

use super::types::NotABotSeed;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SeedTokenError {
    MissingPayload,
    MissingSignature,
    InvalidPayloadEncoding,
    InvalidSignatureEncoding,
    InvalidPayloadUtf8,
    SignatureMismatch,
    InvalidPayloadJson,
    InvalidOperationEnvelope(crate::challenge::operation_envelope::EnvelopeValidationError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MarkerTokenError {
    MissingPayload,
    MissingSignature,
    InvalidPayloadEncoding,
    InvalidSignatureEncoding,
    InvalidPayloadUtf8,
    SignatureMismatch,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct NotABotMarker {
    token_version: u8,
    ip_bucket: String,
    ua_bucket: String,
    expires_at: u64,
}

fn get_challenge_secret() -> String {
    match std::env::var("SHUMA_CHALLENGE_SECRET") {
        Ok(secret) if !secret.trim().is_empty() => secret,
        _ => crate::config::env_string_required("SHUMA_JS_SECRET"),
    }
}

fn sign_payload(payload: &str) -> Vec<u8> {
    let secret = get_challenge_secret();
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(payload.as_bytes());
    mac.finalize().into_bytes().to_vec()
}

fn verify_signature(payload: &str, sig: &[u8]) -> bool {
    let secret = get_challenge_secret();
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(payload.as_bytes());
    mac.verify_slice(sig).is_ok()
}

fn encode_signed_payload(payload_json: &str) -> String {
    let sig = sign_payload(payload_json);
    let payload_b64 = general_purpose::STANDARD.encode(payload_json.as_bytes());
    let sig_b64 = general_purpose::STANDARD.encode(sig);
    format!("{}.{}", payload_b64, sig_b64)
}

fn decode_signed_payload(token: &str) -> Result<String, MarkerTokenError> {
    let mut parts = token.splitn(2, '.');
    let payload_b64 = parts.next().ok_or(MarkerTokenError::MissingPayload)?;
    let sig_b64 = parts.next().ok_or(MarkerTokenError::MissingSignature)?;
    let payload_bytes = general_purpose::STANDARD
        .decode(payload_b64.as_bytes())
        .map_err(|_| MarkerTokenError::InvalidPayloadEncoding)?;
    let sig = general_purpose::STANDARD
        .decode(sig_b64.as_bytes())
        .map_err(|_| MarkerTokenError::InvalidSignatureEncoding)?;
    let payload_json =
        String::from_utf8(payload_bytes).map_err(|_| MarkerTokenError::InvalidPayloadUtf8)?;
    if !verify_signature(&payload_json, &sig) {
        return Err(MarkerTokenError::SignatureMismatch);
    }
    Ok(payload_json)
}

pub(crate) fn make_seed_token(payload: &NotABotSeed) -> String {
    let payload_json = serde_json::to_string(payload).unwrap();
    encode_signed_payload(&payload_json)
}

pub(crate) fn parse_seed_token(token: &str) -> Result<NotABotSeed, SeedTokenError> {
    let mut parts = token.splitn(2, '.');
    let payload_b64 = parts.next().ok_or(SeedTokenError::MissingPayload)?;
    let sig_b64 = parts.next().ok_or(SeedTokenError::MissingSignature)?;
    let payload_bytes = general_purpose::STANDARD
        .decode(payload_b64.as_bytes())
        .map_err(|_| SeedTokenError::InvalidPayloadEncoding)?;
    let sig = general_purpose::STANDARD
        .decode(sig_b64.as_bytes())
        .map_err(|_| SeedTokenError::InvalidSignatureEncoding)?;
    let payload_json =
        String::from_utf8(payload_bytes).map_err(|_| SeedTokenError::InvalidPayloadUtf8)?;
    if !verify_signature(&payload_json, &sig) {
        return Err(SeedTokenError::SignatureMismatch);
    }
    let payload = serde_json::from_str::<NotABotSeed>(&payload_json)
        .map_err(|_| SeedTokenError::InvalidPayloadJson)?;
    crate::challenge::operation_envelope::validate_signed_operation_envelope(
        payload.operation_id.as_str(),
        payload.flow_id.as_str(),
        payload.step_id.as_str(),
        payload.issued_at,
        payload.expires_at,
        payload.token_version,
        crate::challenge::operation_envelope::FLOW_NOT_A_BOT,
        crate::challenge::operation_envelope::STEP_NOT_A_BOT_SUBMIT,
    )
    .map_err(SeedTokenError::InvalidOperationEnvelope)?;
    Ok(payload)
}

pub(crate) fn marker_cookie_value(ip_bucket: &str, ua_bucket: &str, ttl_seconds: u64) -> String {
    let expires_at = crate::admin::now_ts().saturating_add(ttl_seconds);
    let marker = NotABotMarker {
        token_version: crate::challenge::operation_envelope::TOKEN_VERSION_V1,
        ip_bucket: ip_bucket.to_string(),
        ua_bucket: ua_bucket.to_string(),
        expires_at,
    };
    let payload_json = serde_json::to_string(&marker).unwrap();
    let token = encode_signed_payload(&payload_json);
    format!(
        "shuma_not_a_bot={}; path=/; HttpOnly; SameSite=Strict; Max-Age={}",
        token, ttl_seconds
    )
}

pub(crate) fn has_valid_marker(req: &spin_sdk::http::Request, ip: &str, ua: &str) -> bool {
    let cookie_header = match req.header("cookie").and_then(|value| value.as_str()) {
        Some(value) => value,
        None => return false,
    };
    let token = match extract_cookie(cookie_header, "shuma_not_a_bot") {
        Some(value) if !value.trim().is_empty() => value,
        _ => return false,
    };
    let payload_json = match decode_signed_payload(token.as_str()) {
        Ok(value) => value,
        Err(_) => return false,
    };
    let marker = match serde_json::from_str::<NotABotMarker>(&payload_json) {
        Ok(value) => value,
        Err(_) => return false,
    };
    if marker.token_version != crate::challenge::operation_envelope::TOKEN_VERSION_V1 {
        return false;
    }
    let now = crate::admin::now_ts();
    if now > marker.expires_at {
        return false;
    }
    let expected_ip_bucket = crate::signals::ip_identity::bucket_ip(ip);
    if marker.ip_bucket != expected_ip_bucket {
        return false;
    }
    let expected_ua_bucket = crate::challenge::operation_envelope::user_agent_bucket(ua);
    marker.ua_bucket == expected_ua_bucket
}

fn extract_cookie(cookie_header: &str, key: &str) -> Option<String> {
    for part in cookie_header.split(';') {
        let trimmed = part.trim();
        let Some((cookie_key, cookie_value)) = trimmed.split_once('=') else {
            continue;
        };
        if cookie_key.trim() == key {
            return Some(cookie_value.trim().to_string());
        }
    }
    None
}

#[cfg(test)]
pub(crate) fn parse_marker_token(token: &str) -> Result<(String, String, u64), MarkerTokenError> {
    let payload_json = decode_signed_payload(token)?;
    let marker = serde_json::from_str::<NotABotMarker>(&payload_json)
        .map_err(|_| MarkerTokenError::InvalidPayloadEncoding)?;
    Ok((marker.ip_bucket, marker.ua_bucket, marker.expires_at))
}

#[cfg(test)]
mod tests {
    use super::*;
    use spin_sdk::http::Request;

    #[test]
    fn seed_token_roundtrip_validates_not_a_bot_envelope() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_JS_SECRET", "unit-test-secret");
        std::env::remove_var("SHUMA_CHALLENGE_SECRET");
        let seed = NotABotSeed {
            operation_id: "0123456789abcdef0123456789abcdef".to_string(),
            flow_id: crate::challenge::operation_envelope::FLOW_NOT_A_BOT.to_string(),
            step_id: crate::challenge::operation_envelope::STEP_NOT_A_BOT_SUBMIT.to_string(),
            step_index: crate::challenge::operation_envelope::STEP_INDEX_NOT_A_BOT_SUBMIT,
            issued_at: 100,
            expires_at: 200,
            token_version: crate::challenge::operation_envelope::TOKEN_VERSION_V1,
            ip_bucket: "198.51.100.0".to_string(),
            ua_bucket: "ua_bucket".to_string(),
            path_class: crate::challenge::operation_envelope::PATH_CLASS_NOT_A_BOT_SUBMIT
                .to_string(),
            return_to: "/".to_string(),
        };
        let token = make_seed_token(&seed);
        let parsed = parse_seed_token(token.as_str()).expect("seed should parse");
        assert_eq!(parsed.operation_id, seed.operation_id);
        assert_eq!(parsed.flow_id, seed.flow_id);
    }

    #[test]
    fn marker_cookie_roundtrip_matches_ip_and_ua_bucket() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_JS_SECRET", "unit-test-secret");
        std::env::remove_var("SHUMA_CHALLENGE_SECRET");
        let ip_bucket = "198.51.100.0";
        let ua = "Mozilla/5.0";
        let ua_bucket = crate::challenge::operation_envelope::user_agent_bucket(ua);
        let cookie = marker_cookie_value(ip_bucket, ua_bucket.as_str(), 600);
        let token = cookie
            .split(';')
            .next()
            .and_then(|pair| pair.split_once('='))
            .map(|(_, value)| value.to_string())
            .expect("cookie token");
        let (parsed_ip_bucket, parsed_ua_bucket, parsed_expiry) =
            parse_marker_token(token.as_str()).expect("marker should parse");
        assert_eq!(parsed_ip_bucket, ip_bucket);
        assert_eq!(parsed_ua_bucket, ua_bucket);
        assert!(parsed_expiry > crate::admin::now_ts());

        let req = Request::builder()
            .method(spin_sdk::http::Method::Get)
            .uri("/")
            .header("cookie", cookie)
            .header("user-agent", ua)
            .build();
        assert!(has_valid_marker(&req, "198.51.100.42", ua));
        assert!(!has_valid_marker(&req, "203.0.113.10", ua));
    }
}
