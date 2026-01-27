// src/honeypot.rs
// Honeypot URL logic for WASM Bot Trap

pub fn is_honeypot(path: &str, honeypots: &[String]) -> bool {
    honeypots.iter().any(|h| h == path)
}
