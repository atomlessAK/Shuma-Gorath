// src/ban_tests.rs
// Unit tests for ban logic

#[cfg(test)]
mod tests {
    use super::super::ban::*;
    use std::collections::HashMap;

    use std::cell::RefCell;
    #[derive(Default)]
    struct TestStore {
        map: RefCell<HashMap<String, Vec<u8>>>,
    }
    impl super::super::challenge::KeyValueStore for TestStore {
        fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ()> {
            Ok(self.map.borrow().get(key).cloned())
        }
        fn set(&self, key: &str, value: &[u8]) -> Result<(), ()> {
            self.map.borrow_mut().insert(key.to_string(), value.to_vec());
            Ok(())
        }
        fn delete(&self, key: &str) -> Result<(), ()> {
            self.map.borrow_mut().remove(key);
            Ok(())
        }
    }

    #[test]
    fn test_ban_and_expiry() {
        let store = TestStore::default();
        let site_id = "testsite";
        let ip = "1.2.3.4";
        // Ban IP for 1 second
        ban_ip(&store, site_id, ip, "test", 1);
        assert!(is_banned(&store, site_id, ip));
    }

    #[test]
    fn test_ban_and_unban_unknown_ip() {
        let store = TestStore::default();
        let site_id = "testsite";
        let ip = "unknown";
        // Ban 'unknown' IP
        ban_ip(&store, site_id, ip, "test", 60);
        assert!(is_banned(&store, site_id, ip));
        // Unban using the unban_ip function
        unban_ip(&store, site_id, ip);
        assert!(!is_banned(&store, site_id, ip));
    }

    #[test]
    fn test_unban_ip_function() {
        let store = TestStore::default();
        let site_id = "testsite";
        let ip = "192.168.1.100";
        
        // Ban an IP
        ban_ip(&store, site_id, ip, "test_reason", 3600);
        assert!(is_banned(&store, site_id, ip), "IP should be banned after ban_ip");
        
        // Unban using unban_ip function
        unban_ip(&store, site_id, ip);
        assert!(!is_banned(&store, site_id, ip), "IP should not be banned after unban_ip");
    }

    #[test]
    fn test_unban_ip_nonexistent() {
        let store = TestStore::default();
        let site_id = "testsite";
        let ip = "10.0.0.1";
        
        // Unban a non-existent IP should not panic
        unban_ip(&store, site_id, ip);
        assert!(!is_banned(&store, site_id, ip), "Non-existent IP should not be banned");
    }

    #[test]
    fn test_ban_entry_serialization() {
        let entry = BanEntry {
            reason: "test".to_string(),
            expires: 1234567890,
            banned_at: 1234560000,
            fingerprint: Some(BanFingerprint {
                score: Some(6),
                signals: vec!["rate_limit_exceeded".to_string()],
                summary: Some("rate_limit=80".to_string()),
            }),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let de: BanEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(de.reason, "test");
        assert_eq!(de.expires, 1234567890);
        assert_eq!(de.banned_at, 1234560000);
        assert!(de.fingerprint.is_some());
    }

    #[test]
    fn test_legacy_ban_entry_deserializes_with_defaults() {
        let legacy = r#"{"reason":"legacy","expires":42}"#;
        let de: BanEntry = serde_json::from_str(legacy).unwrap();
        assert_eq!(de.reason, "legacy");
        assert_eq!(de.expires, 42);
        assert!(de.fingerprint.is_none());
        assert!(de.banned_at > 0);
    }
    // use super::super::ban::*;
    // Removed MockStore; all tests use TestStore implementing KeyValueStore

    // Removed duplicate test using MockStore; TestStore is used for all tests
}
