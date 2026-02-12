// src/signals/ip_identity.rs
// Helper utilities for bucketing/sanitizing IPs to reduce key cardinality

use std::hash::{Hash, Hasher};
use std::net::IpAddr;

/// Bucket an IP address to reduce cardinality for KV keys.
///
/// Strategy:
/// - IPv4: mask to /24 (zero last octet) => "a.b.c.0"
/// - IPv6: represent the first four segments (64-bit prefix) => "xxxx:xxxx:xxxx:xxxx::/64"
/// - Fallback: hash the original string into one of N buckets and return "h{n}".
pub fn bucket_ip(ip: &str) -> String {
    bucket_ip_with_buckets(ip, 1024)
}

/// Same as `bucket_ip` but allow specifying number of hash buckets for fallback.
pub fn bucket_ip_with_buckets(ip: &str, buckets: u64) -> String {
    if let Ok(addr) = ip.parse::<IpAddr>() {
        match addr {
            IpAddr::V4(v4) => {
                let o = v4.octets();
                return format!("{}.{}.{}.0", o[0], o[1], o[2]);
            }
            IpAddr::V6(v6) => {
                let segs = v6.segments();
                return format!(
                    "{:x}:{:x}:{:x}:{:x}::/64",
                    segs[0], segs[1], segs[2], segs[3]
                );
            }
        }
    }
    // Fallback: hash into N buckets using the default hasher
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    ip.hash(&mut hasher);
    let h = hasher.finish() % buckets;
    format!("h{}", h)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipv4_bucket() {
        assert_eq!(bucket_ip("1.2.3.4"), "1.2.3.0");
    }

    #[test]
    fn test_ipv6_bucket() {
        // just ensure it doesn't panic and contains ::/64
        let b = bucket_ip("2001:0db8:85a3:0000:0000:8a2e:0370:7334");
        assert!(b.contains("::/64"));
    }

    #[test]
    fn test_fallback_hash() {
        let b1 = bucket_ip_with_buckets("not-an-ip", 16);
        let b2 = bucket_ip_with_buckets("another", 16);
        assert!(b1.starts_with('h'));
        assert!(b2.starts_with('h'));
    }
}
