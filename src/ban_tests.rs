        #[test]
        fn test_ban_and_unban_unknown_ip() {
            let mut store = MockStore::default();
            let site_id = "testsite";
            let ip = "unknown";
            // Ban 'unknown' IP
            ban_ip(&store, site_id, ip, "test", 60);
            assert!(is_banned(&store, site_id, ip));
            // Unban 'unknown' IP (simulate admin unban)
            let key = format!("ban:{}:{}", site_id, ip);
            let _ = store.delete(&key);
            assert!(!is_banned(&store, site_id, ip));
        }
    #[test]
    fn test_ban_entry_serialization() {
        let entry = BanEntry {
            reason: "test".to_string(),
            expires: 1234567890,
        };
        let json = serde_json::to_string(&entry).unwrap();
        let de: BanEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(de.reason, "test");
        assert_eq!(de.expires, 1234567890);
    }
// src/ban_tests.rs
// Unit tests for ban logic

#[cfg(test)]
mod tests {
    use super::super::ban::*;
    use spin_sdk::key_value::testing::MockStore;

    #[test]
    fn test_ban_and_expiry() {
        let mut store = MockStore::default();
        let site_id = "testsite";
        let ip = "1.2.3.4";
        // Ban IP for 1 second
        ban_ip(&store, site_id, ip, "test", 1);
        assert!(is_banned(&store, site_id, ip));
        // Simulate expiry
        std::thread::sleep(std::time::Duration::from_secs(2));
        assert!(!is_banned(&store, site_id, ip));
    }
}
