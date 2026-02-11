// src/whitelist_tests.rs
// Unit tests for whitelist logic (single IP, CIDR, comments)

#[cfg(test)]
mod tests {
    use super::super::whitelist::is_whitelisted;

    #[test]
    fn test_exact_ip_match() {
        let wl = vec!["1.2.3.4".to_string()];
        assert!(is_whitelisted("1.2.3.4", &wl));
        assert!(!is_whitelisted("1.2.3.5", &wl));
    }

    #[test]
    fn test_cidr_match() {
        let wl = vec!["192.168.1.0/24".to_string()];
        assert!(is_whitelisted("192.168.1.42", &wl));
        assert!(!is_whitelisted("192.168.2.1", &wl));
    }

    #[test]
    fn test_inline_comment_and_whitespace() {
        let wl = vec![
            "10.0.0.0/8 # corp network".to_string(),
            "  8.8.8.8   # google dns ".to_string(),
        ];
        assert!(is_whitelisted("10.1.2.3", &wl));
        assert!(is_whitelisted("8.8.8.8", &wl));
        assert!(!is_whitelisted("8.8.4.4", &wl));
    }

    #[test]
    fn test_empty_and_comment_only_lines() {
        let wl = vec![
            "# just a comment".to_string(),
            "   ".to_string(),
            "172.16.0.0/12".to_string(),
        ];
        assert!(is_whitelisted("172.16.5.5", &wl));
        assert!(!is_whitelisted("192.0.2.1", &wl));
    }
}
