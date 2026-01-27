// src/whitelist.rs
// Whitelist logic for WASM Bot Trap

pub fn is_whitelisted(ip: &str, whitelist: &[String]) -> bool {
    whitelist.iter().any(|w| w == ip)
}
