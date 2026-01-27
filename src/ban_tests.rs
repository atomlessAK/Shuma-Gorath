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
