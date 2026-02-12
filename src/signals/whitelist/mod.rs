/// Returns true if the given path matches any path whitelist entry (exact or prefix match, supports trailing * wildcard).
pub fn is_path_whitelisted(path: &str, path_whitelist: &[String]) -> bool {
    for entry in path_whitelist {
        let entry = entry.split('#').next().unwrap_or("").trim();
        if entry.is_empty() {
            continue;
        }
        if entry.ends_with('*') {
            let prefix = &entry[..entry.len() - 1];
            if path.starts_with(prefix) {
                return true;
            }
        } else if path == entry {
            return true;
        }
    }
    false
}
// src/whitelist.rs
// Whitelist logic for WASM Bot Defence
// Supports single IPs, CIDR ranges, and inline comments (e.g., "192.168.1.0/24 # office")

use ipnet::IpNet;
use std::net::IpAddr;

/// Returns true if the given IP is whitelisted by exact match or CIDR range.
pub fn is_whitelisted(ip: &str, whitelist: &[String]) -> bool {
    let ip_addr: IpAddr = match ip.parse() {
        Ok(addr) => addr,
        Err(_) => return false,
    };
    for entry in whitelist {
        // Remove inline comments and trim whitespace
        let entry = entry.split('#').next().unwrap_or("").trim();
        if entry.is_empty() {
            continue;
        }
        // Try exact match
        if entry == ip {
            return true;
        }
        // Try CIDR match
        if let Ok(net) = entry.parse::<IpNet>() {
            if net.contains(&ip_addr) {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod path_tests;

#[cfg(test)]
mod tests;
