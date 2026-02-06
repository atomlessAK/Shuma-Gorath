// src/challenge.rs
// Interactive math challenge for banned users

use spin_sdk::http::{Request, Response};
use spin_sdk::key_value::Store;

pub trait KeyValueStore {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ()>;
    fn set(&self, key: &str, value: &[u8]) -> Result<(), ()>;
    fn delete(&self, key: &str) -> Result<(), ()>;
}

impl KeyValueStore for Store {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ()> {
        Store::get(self, key).map_err(|_| ())
    }
    fn set(&self, key: &str, value: &[u8]) -> Result<(), ()> {
        Store::set(self, key, value).map_err(|_| ())
    }
    fn delete(&self, key: &str) -> Result<(), ()> {
        Store::delete(self, key).map_err(|_| ())
    }
}

use base64::{engine::general_purpose, Engine as _};
use hmac::{Hmac, Mac};
use percent_encoding;
use rand::prelude::*;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

const CHALLENGE_PREFIX: &str = "challenge:";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ChallengeSeed {
    pub seed_id: String,
    pub issued_at: u64,
    pub expires_at: u64,
    pub ip_bucket: String,
    pub grid_size: u8,
    pub active_cells: u8,
    pub transforms: Vec<Transform>,
    pub training_count: u8,
    pub seed: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Transform {
    RotateCw90,
    RotateCcw90,
    MirrorHorizontal,
    MirrorVertical,
    ShiftUp,
    ShiftDown,
    ShiftLeft,
    ShiftRight,
    DropTop,
    DropBottom,
    DropLeft,
    DropRight,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ChallengePuzzle {
    pub training_pairs: Vec<(Vec<u8>, Vec<u8>)>,
    pub test_input: Vec<u8>,
    pub test_output: Vec<u8>,
    pub grid_size: usize,
}

fn get_challenge_secret() -> String {
    std::env::var("CHALLENGE_SECRET")
        .or_else(|_| std::env::var("API_KEY"))
        .unwrap_or_else(|_| "changeme-challenge-secret".to_string())
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

#[allow(dead_code)]
fn make_seed_token(payload: &ChallengeSeed) -> String {
    let payload_json = serde_json::to_string(payload).unwrap();
    let sig = sign_payload(&payload_json);
    let payload_b64 = general_purpose::STANDARD.encode(payload_json.as_bytes());
    let sig_b64 = general_purpose::STANDARD.encode(sig);
    format!("{}.{}", payload_b64, sig_b64)
}

#[allow(dead_code)]
fn parse_seed_token(token: &str) -> Result<ChallengeSeed, &'static str> {
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

    serde_json::from_str::<ChallengeSeed>(&payload_json).map_err(|_| "invalid payload")
}

pub(crate) fn build_puzzle(seed: &ChallengeSeed) -> ChallengePuzzle {
    let size = seed.grid_size as usize;
    let active = seed.active_cells as usize;
    let mut rng = StdRng::seed_from_u64(seed.seed);
    let mut training_pairs = Vec::new();
    for _ in 0..seed.training_count {
        let input = generate_grid(&mut rng, size, active);
        let output = apply_transforms(&input, size, &seed.transforms);
        training_pairs.push((input, output));
    }
    let test_input = generate_grid(&mut rng, size, active);
    let test_output = apply_transforms(&test_input, size, &seed.transforms);
    ChallengePuzzle {
        training_pairs,
        test_input,
        test_output,
        grid_size: size,
    }
}

fn generate_grid(rng: &mut StdRng, size: usize, active: usize) -> Vec<u8> {
    let mut grid = vec![0u8; size * size];
    let mut indices: Vec<usize> = (0..grid.len()).collect();
    indices.shuffle(rng);
    for idx in indices.into_iter().take(active) {
        grid[idx] = 1;
    }
    grid
}

fn apply_transforms(grid: &[u8], size: usize, transforms: &[Transform]) -> Vec<u8> {
    let mut current = grid.to_vec();
    for t in transforms {
        current = apply_transform(&current, size, *t);
    }
    current
}

pub(crate) fn apply_transform(grid: &[u8], size: usize, transform: Transform) -> Vec<u8> {
    let mut out = vec![0u8; size * size];
    match transform {
        Transform::RotateCw90 => {
            for r in 0..size {
                for c in 0..size {
                    let src = idx(r, c, size);
                    let dst = idx(c, size - 1 - r, size);
                    out[dst] = grid[src];
                }
            }
        }
        Transform::RotateCcw90 => {
            for r in 0..size {
                for c in 0..size {
                    let src = idx(r, c, size);
                    let dst = idx(size - 1 - c, r, size);
                    out[dst] = grid[src];
                }
            }
        }
        Transform::MirrorHorizontal => {
            for r in 0..size {
                for c in 0..size {
                    let src = idx(r, c, size);
                    let dst = idx(r, size - 1 - c, size);
                    out[dst] = grid[src];
                }
            }
        }
        Transform::MirrorVertical => {
            for r in 0..size {
                for c in 0..size {
                    let src = idx(r, c, size);
                    let dst = idx(size - 1 - r, c, size);
                    out[dst] = grid[src];
                }
            }
        }
        Transform::ShiftUp | Transform::DropTop => {
            for r in 1..size {
                for c in 0..size {
                    let src = idx(r, c, size);
                    let dst = idx(r - 1, c, size);
                    out[dst] = grid[src];
                }
            }
        }
        Transform::ShiftDown | Transform::DropBottom => {
            for r in 0..size - 1 {
                for c in 0..size {
                    let src = idx(r, c, size);
                    let dst = idx(r + 1, c, size);
                    out[dst] = grid[src];
                }
            }
        }
        Transform::ShiftLeft | Transform::DropLeft => {
            for r in 0..size {
                for c in 1..size {
                    let src = idx(r, c, size);
                    let dst = idx(r, c - 1, size);
                    out[dst] = grid[src];
                }
            }
        }
        Transform::ShiftRight | Transform::DropRight => {
            for r in 0..size {
                for c in 0..size - 1 {
                    let src = idx(r, c, size);
                    let dst = idx(r, c + 1, size);
                    out[dst] = grid[src];
                }
            }
        }
    }
    out
}

pub(crate) fn parse_submission(input: &str, size: usize) -> Result<Vec<u8>, &'static str> {
    let trimmed = input.trim();
    let expected = size * size;
    if trimmed.is_empty() {
        return Err("invalid length");
    }
    let is_bitstring = trimmed.chars().all(|c| c == '0' || c == '1');
    if is_bitstring {
        if trimmed.len() != expected {
            return Err("invalid length");
        }
        let out = trimmed.chars().map(|c| if c == '1' { 1 } else { 0 }).collect();
        return Ok(out);
    }
    let mut out = vec![0u8; expected];
    for part in trimmed.split(',') {
        let p = part.trim();
        if p.is_empty() {
            continue;
        }
        let idx = p.parse::<usize>().map_err(|_| "invalid index")?;
        if idx >= expected {
            return Err("invalid index");
        }
        out[idx] = 1;
    }
    Ok(out)
}

fn idx(row: usize, col: usize, size: usize) -> usize {
    row * size + col
}

/// Generates a random math challenge (add, sub, mul) and stores the answer and question type in KV for the IP.
/// NOTE: This function is currently unused - banned users now see block page directly.
/// Kept for potential future use if challenge-on-ban is re-enabled.
#[allow(dead_code)]
pub fn serve_challenge<S: KeyValueStore>(store: &S, ip: &str) -> Response {
    let mut rng = rand::rng();
    let a: u32 = rng.random_range(10..=99);
    let b: u32 = rng.random_range(10..=99);
    let question_types = ["add", "sub", "mul"];
    let qtype = *question_types.choose(&mut rng).unwrap_or(&"add");
    let (question, answer) = match qtype {
        "add" => (format!("{a} + {b}"), a + b),
        "sub" => {
            let (x, y) = if a > b { (a, b) } else { (b, a) };
            (format!("{x} - {y}"), x - y)
        },
        "mul" => {
            let a = rng.random_range(2..=12);
            let b = rng.random_range(2..=12);
            (format!("{a} × {b}"), a * b)
        },
        _ => (format!("{a} + {b}"), a + b),
    };
    let key = format!("{}{}", CHALLENGE_PREFIX, ip);
    let value = format!("{}:{}", answer, qtype);
    let _ = store.set(&key, value.as_bytes());
    let html = format!(r#"
        <html><head><style>
        body {{ font-family: sans-serif; background: #f9f9f9; margin: 2em; }}
        .challenge-container {{ background: #fff; padding: 2em; border-radius: 8px; box-shadow: 0 2px 8px #ccc; max-width: 400px; margin: auto; }}
        label {{ font-size: 1.2em; }}
        input[type=number] {{ font-size: 1.2em; width: 80px; }}
        button {{ font-size: 1em; padding: 0.5em 1em; }}
        </style></head><body>
        <div class="challenge-container">
        <h2>Are you human?</h2>
        <form method='POST' action='/challenge'>
            <label>Solve: {question} = </label>
            <input name='answer' type='number' required autofocus />
            <input type='hidden' name='ip' value='{ip}' />
            <button type='submit'>Submit</button>
        </form>
        <p style="color: #888; font-size: 0.9em;">Prove you are not a bot to regain access.</p>
        </div>
        </body></html>
    "#);
    Response::new(200, html)
}

/// Validates the challenge answer. If correct, unbans the IP and returns a success page.
pub fn handle_challenge_submit<S: KeyValueStore>(store: &S, req: &Request) -> Response {
    let form = String::from_utf8_lossy(req.body()).to_string();
    let answer = get_form_field(&form, "answer");
    let ip = get_form_field(&form, "ip");
    if let (Some(answer), Some(ip)) = (answer, ip) {
        let key = format!("{}{}", CHALLENGE_PREFIX, ip);
        if let Ok(Some(val)) = store.get(&key) {
            if let Ok(stored) = String::from_utf8(val) {
                let mut parts = stored.splitn(2, ':');
                if let (Some(expected), _) = (parts.next(), parts.next()) {
                    // Compare as integers for robustness
                    if let (Ok(expected_num), Ok(answer_num)) = (expected.trim().parse::<i64>(), answer.trim().parse::<i64>()) {
                        if answer_num == expected_num {
                            // Unban the IP
                            let ban_key = format!("ban:default:{}", ip);
                            let _ = store.delete(&ban_key);
                            let _ = store.delete(&key);
                            return Response::new(200, "<html><body><h2>Thank you! You are unbanned. Please reload the page.</h2></body></html>");
                        }
                    }
                }
            }
        }
    }
    let html = "<html><body><h2 style='color:red;'>Incorrect answer. Please try again.</h2><a href='/challenge'>Back to challenge</a></body></html>";
    Response::new(403, html)
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
    percent_encoding::percent_decode_str(s).decode_utf8_lossy().to_string()
}
