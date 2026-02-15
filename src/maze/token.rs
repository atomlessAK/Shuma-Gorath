use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

const TOKEN_VERSION_V1: u8 = 1;
const TOKEN_PREFIX: &str = "mzt1";
const MAX_TOKEN_BYTES: usize = 4096;
const OPERATION_ID_HEX_BYTES: usize = 12;
const EXPANSION_SEED_SIG_HEX_BYTES: usize = 16;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct MazeTraversalToken {
    pub version: u8,
    pub operation_id: String,
    pub flow_id: String,
    pub path_prefix: String,
    pub path_digest: String,
    pub ip_bucket: String,
    pub ua_bucket: String,
    pub issued_at: u64,
    pub expires_at: u64,
    pub depth: u16,
    pub branch_budget: u8,
    pub prev_digest: String,
    pub entropy_nonce: String,
    pub variant_id: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MazeTokenError {
    Missing,
    Malformed,
    SignatureMismatch,
    InvalidVersion,
    Expired,
}

fn now_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn hex_lower(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let hi = byte >> 4;
        let lo = byte & 0x0f;
        out.push(match hi {
            0..=9 => (b'0' + hi) as char,
            _ => (b'a' + (hi - 10)) as char,
        });
        out.push(match lo {
            0..=9 => (b'0' + lo) as char,
            _ => (b'a' + (lo - 10)) as char,
        });
    }
    out
}

fn hmac_sign(secret: &str, payload: &[u8]) -> Vec<u8> {
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC accepts any secret length");
    mac.update(payload);
    mac.finalize().into_bytes().to_vec()
}

fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (lhs, rhs) in a.bytes().zip(b.bytes()) {
        diff |= lhs ^ rhs;
    }
    diff == 0
}

pub(crate) fn secret_from_env() -> String {
    std::env::var("SHUMA_MAZE_SECRET")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .or_else(|| {
            std::env::var("SHUMA_CHALLENGE_SECRET")
                .ok()
                .filter(|v| !v.trim().is_empty())
        })
        .or_else(|| {
            std::env::var("SHUMA_JS_SECRET")
                .ok()
                .filter(|v| !v.trim().is_empty())
        })
        .unwrap_or_else(|| "maze-default-secret".to_string())
}

pub(crate) fn verify(
    raw_token: &str,
    secret: &str,
    now_override: Option<u64>,
) -> Result<MazeTraversalToken, MazeTokenError> {
    if raw_token.trim().is_empty() {
        return Err(MazeTokenError::Missing);
    }
    if raw_token.len() > MAX_TOKEN_BYTES {
        return Err(MazeTokenError::Malformed);
    }

    let mut parts = raw_token.split('.');
    let prefix = parts.next().ok_or(MazeTokenError::Malformed)?;
    let encoded_payload = parts.next().ok_or(MazeTokenError::Malformed)?;
    let encoded_sig = parts.next().ok_or(MazeTokenError::Malformed)?;
    if parts.next().is_some() || prefix != TOKEN_PREFIX {
        return Err(MazeTokenError::Malformed);
    }

    let payload = URL_SAFE_NO_PAD
        .decode(encoded_payload)
        .map_err(|_| MazeTokenError::Malformed)?;
    let expected_sig = hmac_sign(secret, &payload);
    let supplied_sig = URL_SAFE_NO_PAD
        .decode(encoded_sig)
        .map_err(|_| MazeTokenError::Malformed)?;
    if expected_sig != supplied_sig {
        return Err(MazeTokenError::SignatureMismatch);
    }

    let token: MazeTraversalToken =
        serde_json::from_slice(payload.as_slice()).map_err(|_| MazeTokenError::Malformed)?;
    if token.version != TOKEN_VERSION_V1 {
        return Err(MazeTokenError::InvalidVersion);
    }
    let now = now_override.unwrap_or_else(now_seconds);
    if now > token.expires_at {
        return Err(MazeTokenError::Expired);
    }
    Ok(token)
}

pub(crate) fn sign(token: &MazeTraversalToken, secret: &str) -> String {
    let payload = serde_json::to_vec(token).expect("maze token should serialize");
    let payload_b64 = URL_SAFE_NO_PAD.encode(payload.as_slice());
    let signature_b64 = URL_SAFE_NO_PAD.encode(hmac_sign(secret, payload.as_slice()));
    format!("{TOKEN_PREFIX}.{payload_b64}.{signature_b64}")
}

fn operation_id(target_path: &str, flow_id: &str, depth: u16, now: u64) -> String {
    let mut hasher = Sha256::new();
    hasher.update(target_path.as_bytes());
    hasher.update(flow_id.as_bytes());
    hasher.update(depth.to_le_bytes());
    hasher.update(now.to_le_bytes());
    let digest = hasher.finalize();
    hex_lower(&digest[..OPERATION_ID_HEX_BYTES])
}

pub(crate) fn digest(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    let digest = hasher.finalize();
    hex_lower(&digest[..12])
}

pub(crate) fn ua_bucket(user_agent: &str) -> String {
    let normalized = if user_agent.trim().is_empty() {
        "unknown"
    } else {
        user_agent.trim()
    };
    digest(normalized)
}

pub(crate) fn flow_id_from(ip_bucket: &str, ua_bucket: &str, path: &str, now: u64) -> String {
    let mut hasher = Sha256::new();
    hasher.update(ip_bucket.as_bytes());
    hasher.update(ua_bucket.as_bytes());
    hasher.update(path.as_bytes());
    hasher.update(now.to_le_bytes());
    hex_lower(&hasher.finalize()[..12])
}

pub(crate) fn issue_child_token(
    parent: Option<&MazeTraversalToken>,
    target_path: &str,
    path_prefix: &str,
    ip_bucket: &str,
    ua_bucket: &str,
    ttl_seconds: u64,
    max_depth: u16,
    branch_budget: u8,
    entropy_nonce: &str,
    variant_id: u16,
    now: u64,
) -> MazeTraversalToken {
    let parent_depth = parent.map(|token| token.depth).unwrap_or(0);
    let depth = (parent_depth + 1).min(max_depth);
    let flow_id = parent
        .map(|token| token.flow_id.clone())
        .unwrap_or_else(|| flow_id_from(ip_bucket, ua_bucket, path_prefix, now));
    let path_digest = digest(target_path);
    let parent_digest = parent
        .map(|token| digest(format!("{}:{}", token.flow_id, token.operation_id).as_str()))
        .unwrap_or_else(|| digest(path_prefix));

    MazeTraversalToken {
        version: TOKEN_VERSION_V1,
        operation_id: operation_id(target_path, flow_id.as_str(), depth, now),
        flow_id,
        path_prefix: path_prefix.to_string(),
        path_digest,
        ip_bucket: ip_bucket.to_string(),
        ua_bucket: ua_bucket.to_string(),
        issued_at: now,
        expires_at: now.saturating_add(ttl_seconds),
        depth,
        branch_budget,
        prev_digest: parent_digest,
        entropy_nonce: entropy_nonce.to_string(),
        variant_id,
    }
}

pub(crate) fn verify_micro_pow(raw_token: &str, nonce: &str, difficulty: u8) -> bool {
    if difficulty == 0 || nonce.trim().is_empty() {
        return true;
    }
    let mut hasher = Sha256::new();
    hasher.update(raw_token.as_bytes());
    hasher.update(b":");
    hasher.update(nonce.as_bytes());
    let digest = hasher.finalize();

    let mut bits_remaining = difficulty;
    for byte in digest {
        if bits_remaining == 0 {
            return true;
        }
        if bits_remaining >= 8 {
            if byte != 0 {
                return false;
            }
            bits_remaining -= 8;
        } else {
            let mask = 0xff << (8 - bits_remaining);
            return (byte & mask) == 0;
        }
    }
    true
}

pub(crate) fn entropy_seed(
    secret: &str,
    site_id: &str,
    ip_bucket: &str,
    ua_bucket: &str,
    path: &str,
    minute_bucket: u64,
    chain_nonce: &str,
) -> u64 {
    let payload = format!("{site_id}|{ip_bucket}|{ua_bucket}|{path}|{minute_bucket}|{chain_nonce}");
    let digest = hmac_sign(secret, payload.as_bytes());
    let mut seed_bytes = [0u8; 8];
    seed_bytes.copy_from_slice(&digest[..8]);
    u64::from_le_bytes(seed_bytes)
}

fn expansion_seed_payload(
    flow_id: &str,
    path_prefix: &str,
    entropy_nonce: &str,
    depth: u16,
    seed: u64,
    hidden_count: usize,
    segment_len: usize,
) -> String {
    format!("{flow_id}|{path_prefix}|{entropy_nonce}|{depth}|{seed}|{hidden_count}|{segment_len}")
}

pub(crate) fn sign_expansion_seed(
    secret: &str,
    flow_id: &str,
    path_prefix: &str,
    entropy_nonce: &str,
    depth: u16,
    seed: u64,
    hidden_count: usize,
    segment_len: usize,
) -> String {
    let payload = expansion_seed_payload(
        flow_id,
        path_prefix,
        entropy_nonce,
        depth,
        seed,
        hidden_count,
        segment_len,
    );
    let digest = hmac_sign(secret, payload.as_bytes());
    hex_lower(&digest[..EXPANSION_SEED_SIG_HEX_BYTES])
}

pub(crate) fn verify_expansion_seed_signature(
    signature: &str,
    secret: &str,
    flow_id: &str,
    path_prefix: &str,
    entropy_nonce: &str,
    depth: u16,
    seed: u64,
    hidden_count: usize,
    segment_len: usize,
) -> bool {
    if signature.trim().is_empty() {
        return false;
    }
    let expected = sign_expansion_seed(
        secret,
        flow_id,
        path_prefix,
        entropy_nonce,
        depth,
        seed,
        hidden_count,
        segment_len,
    );
    constant_time_eq(expected.as_str(), signature)
}

#[cfg(test)]
mod tests {
    use super::{
        digest, entropy_seed, issue_child_token, secret_from_env, sign, sign_expansion_seed,
        verify, verify_expansion_seed_signature, verify_micro_pow, MazeTokenError,
    };

    #[test]
    fn token_round_trip_succeeds() {
        let secret = "maze-test-secret";
        let token = issue_child_token(
            None,
            "/maze/a",
            "/maze/",
            "ipb",
            "uab",
            120,
            8,
            3,
            "nonce",
            2,
            1_735_000_000,
        );
        let raw = sign(&token, secret);
        let parsed = verify(&raw, secret, Some(1_735_000_010)).expect("token should verify");
        assert_eq!(parsed.flow_id, token.flow_id);
        assert_eq!(parsed.depth, token.depth);
        assert_eq!(parsed.path_digest, digest("/maze/a"));
    }

    #[test]
    fn token_rejects_signature_mismatch() {
        let secret = "maze-test-secret";
        let token = issue_child_token(
            None,
            "/maze/a",
            "/maze/",
            "ipb",
            "uab",
            120,
            8,
            3,
            "nonce",
            2,
            1_735_000_000,
        );
        let mut raw = sign(&token, secret);
        raw.push('x');
        let err = verify(&raw, secret, Some(1_735_000_010)).unwrap_err();
        assert_eq!(err, MazeTokenError::SignatureMismatch);
    }

    #[test]
    fn token_rejects_expired() {
        let secret = "maze-test-secret";
        let token = issue_child_token(
            None,
            "/maze/a",
            "/maze/",
            "ipb",
            "uab",
            1,
            8,
            3,
            "nonce",
            2,
            1_735_000_000,
        );
        let raw = sign(&token, secret);
        let err = verify(&raw, secret, Some(1_735_000_100)).unwrap_err();
        assert_eq!(err, MazeTokenError::Expired);
    }

    #[test]
    fn entropy_seed_changes_with_window() {
        let secret = "maze-test-secret";
        let first = entropy_seed(secret, "default", "ipb", "uab", "/maze/a", 100, "n1");
        let second = entropy_seed(secret, "default", "ipb", "uab", "/maze/a", 101, "n1");
        assert_ne!(first, second);
    }

    #[test]
    fn micro_pow_accepts_valid_nonce() {
        let token = "sample-token";
        let difficulty = 12u8;
        let mut nonce: u64 = 0;
        loop {
            let probe = nonce.to_string();
            if verify_micro_pow(token, probe.as_str(), difficulty) {
                break;
            }
            nonce += 1;
            assert!(nonce < 2_000_000, "expected nonce should be found quickly");
        }
    }

    #[test]
    fn digest_has_stable_width() {
        assert_eq!(digest("abc").len(), 24);
    }

    #[test]
    fn secret_fallback_never_empty() {
        let secret = secret_from_env();
        assert!(!secret.trim().is_empty());
    }

    #[test]
    fn expansion_seed_signatures_verify_and_reject_tampering() {
        let secret = "maze-test-secret";
        let signature = sign_expansion_seed(secret, "flow-a", "/maze/", "nonce-1", 2, 1234, 6, 16);
        assert!(verify_expansion_seed_signature(
            signature.as_str(),
            secret,
            "flow-a",
            "/maze/",
            "nonce-1",
            2,
            1234,
            6,
            16
        ));
        assert!(!verify_expansion_seed_signature(
            signature.as_str(),
            secret,
            "flow-a",
            "/maze/",
            "nonce-1",
            2,
            9999,
            6,
            16
        ));
    }

    #[test]
    fn sibling_tokens_are_operation_unique_per_link_edge() {
        let now = 1_735_000_000;
        let parent = issue_child_token(
            None,
            "/maze/root",
            "/maze/",
            "ipb",
            "uab",
            120,
            8,
            3,
            "nonce",
            2,
            now,
        );
        let first = issue_child_token(
            Some(&parent),
            "/maze/first-edge",
            "/maze/",
            "ipb",
            "uab",
            120,
            8,
            3,
            "nonce",
            2,
            now,
        );
        let second = issue_child_token(
            Some(&parent),
            "/maze/second-edge",
            "/maze/",
            "ipb",
            "uab",
            120,
            8,
            3,
            "nonce",
            2,
            now,
        );
        assert_ne!(first.operation_id, second.operation_id);
        assert_eq!(first.prev_digest, second.prev_digest);
    }
}
