// src/pow.rs
// Lightweight proof-of-work (PoW) challenge for JS verification

use base64::{engine::general_purpose, Engine as _};
use hmac::{Hmac, Mac};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use spin_sdk::http::{Request, Response};

#[derive(Debug, Serialize, Deserialize)]
struct PowPayload {
    seed_id: String,
    ip_bucket: String,
    issued_at: u64,
    expires_at: u64,
    difficulty: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PowChallenge {
    pub seed: String,
    pub difficulty: u8,
    pub expires_at: u64,
}

fn now_ts() -> u64 {
    crate::admin::now_ts()
}

fn get_pow_secret() -> String {
    std::env::var("SHUMA_POW_SECRET")
        .or_else(|_| std::env::var("SHUMA_JS_SECRET"))
        .or_else(|_| std::env::var("SHUMA_API_KEY"))
        .unwrap_or_else(|_| "changeme-pow-secret".to_string())
}

pub fn pow_enabled() -> bool {
    std::env::var("SHUMA_POW_ENABLED")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(true)
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

fn parse_seed_token(token: &str) -> Result<PowPayload, &'static str> {
    let mut parts = token.splitn(2, '.');
    let payload_b64 = parts.next().ok_or("missing payload")?;
    let sig_b64 = parts.next().ok_or("missing signature")?;
    let payload_bytes = general_purpose::STANDARD
        .decode(payload_b64.as_bytes())
        .map_err(|_| "invalid payload")?;
    let sig = general_purpose::STANDARD
        .decode(sig_b64.as_bytes())
        .map_err(|_| "invalid signature")?;
    let payload_json = String::from_utf8(payload_bytes).map_err(|_| "invalid payload")?;

    if !verify_signature(&payload_json, &sig) {
        return Err("signature mismatch");
    }

    serde_json::from_str::<PowPayload>(&payload_json).map_err(|_| "invalid payload")
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

pub fn issue_pow_challenge(ip: &str, difficulty: u8, ttl_seconds: u64) -> PowChallenge {
    let now = now_ts();
    let ttl = ttl_seconds;
    let seed_id = format!("{:016x}", rand::rng().random::<u64>());
    let payload = PowPayload {
        seed_id,
        ip_bucket: crate::ip::bucket_ip(ip),
        issued_at: now,
        expires_at: now + ttl,
        difficulty,
    };
    let seed = make_seed_token(&payload);
    PowChallenge {
        seed,
        difficulty,
        expires_at: payload.expires_at,
    }
}

pub fn handle_pow_challenge(ip: &str, difficulty: u8, ttl_seconds: u64) -> Response {
    if !pow_enabled() {
        return Response::new(404, "PoW disabled");
    }
    let challenge = issue_pow_challenge(ip, difficulty, ttl_seconds);
    let body = serde_json::to_string(&challenge).unwrap();
    Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .header("Cache-Control", "no-store")
        .body(body)
        .build()
}

pub fn handle_pow_verify(req: &Request, ip: &str) -> Response {
    if !pow_enabled() {
        return Response::new(404, "PoW disabled");
    }
    if *req.method() != spin_sdk::http::Method::Post {
        return Response::new(405, "Method Not Allowed");
    }

    let json = match crate::input_validation::parse_json_body(
        req.body(),
        crate::input_validation::MAX_POW_VERIFY_BYTES,
    ) {
        Ok(v) => v,
        Err(e) => return Response::new(400, e),
    };
    let seed = match json.get("seed").and_then(|v| v.as_str()) {
        Some(v) => v,
        None => return Response::new(400, "Missing seed"),
    };
    if !crate::input_validation::validate_seed_token(seed) {
        return Response::new(400, "Invalid seed");
    }
    let nonce = match json.get("nonce").and_then(|v| v.as_str()) {
        Some(v) => v,
        None => return Response::new(400, "Missing nonce"),
    };
    if !crate::input_validation::validate_nonce(nonce) {
        return Response::new(400, "Invalid nonce");
    }

    let payload = match parse_seed_token(seed) {
        Ok(p) => p,
        Err(_) => return Response::new(400, "Invalid seed"),
    };

    let now = now_ts();
    if now > payload.expires_at {
        return Response::new(400, "Seed expired");
    }

    let ip_bucket = crate::ip::bucket_ip(ip);
    if payload.ip_bucket != ip_bucket {
        return Response::new(400, "IP bucket mismatch");
    }

    if !verify_pow(seed, nonce, payload.difficulty) {
        return Response::new(400, "Invalid proof");
    }

    Response::builder()
        .status(200)
        .header("Set-Cookie", crate::js::js_verified_cookie(ip).as_str())
        .header("Cache-Control", "no-store")
        .body("OK")
        .build()
}
