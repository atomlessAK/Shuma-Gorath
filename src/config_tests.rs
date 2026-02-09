// src/config_tests.rs
// Unit tests for config defaults and parsing

#[cfg(test)]
mod tests {
    use crate::challenge::KeyValueStore;
    use std::collections::HashMap;
    use std::sync::Mutex;

    use once_cell::sync::Lazy;

    static ENV_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[derive(Default)]
    struct TestStore {
        map: Mutex<HashMap<String, Vec<u8>>>,
    }

    impl crate::challenge::KeyValueStore for TestStore {
        fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ()> {
            let m = self.map.lock().unwrap();
            Ok(m.get(key).cloned())
        }

        fn set(&self, key: &str, value: &[u8]) -> Result<(), ()> {
            let mut m = self.map.lock().unwrap();
            m.insert(key.to_string(), value.to_vec());
            Ok(())
        }

        fn delete(&self, key: &str) -> Result<(), ()> {
            let mut m = self.map.lock().unwrap();
            m.remove(key);
            Ok(())
        }
    }

    fn clear_env(keys: &[&str]) {
        for key in keys {
            std::env::remove_var(key);
        }
    }

    #[test]
    fn parse_challenge_threshold_defaults_to_3() {
        assert_eq!(crate::config::parse_challenge_threshold(None), 3);
    }

    #[test]
    fn parse_challenge_threshold_clamps_range() {
        assert_eq!(crate::config::parse_challenge_threshold(Some("0")), 1);
        assert_eq!(crate::config::parse_challenge_threshold(Some("99")), 10);
        assert_eq!(crate::config::parse_challenge_threshold(Some("5")), 5);
        assert_eq!(crate::config::parse_challenge_threshold(Some("junk")), 3);
    }

    #[test]
    fn parse_maze_threshold_clamps_range() {
        assert_eq!(crate::config::parse_maze_threshold(Some("0")), 1);
        assert_eq!(crate::config::parse_maze_threshold(Some("99")), 10);
        assert_eq!(crate::config::parse_maze_threshold(Some("6")), 6);
        assert_eq!(crate::config::parse_maze_threshold(Some("junk")), 6);
    }

    #[test]
    fn parse_botness_weight_clamps_range() {
        assert_eq!(crate::config::parse_botness_weight(Some("0"), 3), 0);
        assert_eq!(crate::config::parse_botness_weight(Some("11"), 3), 10);
        assert_eq!(crate::config::parse_botness_weight(Some("4"), 3), 4);
        assert_eq!(crate::config::parse_botness_weight(Some("junk"), 3), 3);
    }

    #[test]
    fn challenge_config_mutable_from_env_parses_values() {
        assert!(crate::config::challenge_config_mutable_from_env(Some("1")));
        assert!(crate::config::challenge_config_mutable_from_env(Some("true")));
        assert!(crate::config::challenge_config_mutable_from_env(Some("TRUE")));
        assert!(!crate::config::challenge_config_mutable_from_env(Some("0")));
        assert!(!crate::config::challenge_config_mutable_from_env(Some("false")));
        assert!(!crate::config::challenge_config_mutable_from_env(None));
    }

    #[test]
    fn parse_admin_page_config_defaults_to_disabled() {
        assert!(!crate::config::parse_admin_page_config_enabled(None));
        assert!(!crate::config::parse_admin_page_config_enabled(Some("junk")));
        assert!(crate::config::parse_admin_page_config_enabled(Some("true")));
        assert!(crate::config::parse_admin_page_config_enabled(Some("1")));
        assert!(!crate::config::parse_admin_page_config_enabled(Some("false")));
    }

    #[test]
    fn load_admin_page_config_disabled_ignores_kv_values() {
        let _lock = ENV_MUTEX.lock().unwrap();
        let keys = [
            "SHUMA_ADMIN_PAGE_CONFIG",
            "SHUMA_RATE_LIMIT",
            "SHUMA_MAZE_ENABLED",
            "SHUMA_TEST_MODE",
        ];
        clear_env(&keys);
        std::env::set_var("SHUMA_ADMIN_PAGE_CONFIG", "false");
        std::env::set_var("SHUMA_RATE_LIMIT", "321");
        std::env::set_var("SHUMA_MAZE_ENABLED", "0");
        std::env::set_var("SHUMA_TEST_MODE", "1");

        let store = TestStore::default();
        let mut kv_cfg = crate::config::Config::load(&store, "default");
        kv_cfg.rate_limit = 11;
        kv_cfg.maze_enabled = true;
        kv_cfg.test_mode = false;
        let key = "config:default".to_string();
        store.set(&key, &serde_json::to_vec(&kv_cfg).unwrap()).unwrap();

        let cfg = crate::config::Config::load(&store, "default");
        assert_eq!(cfg.rate_limit, 321);
        assert!(!cfg.maze_enabled);
        assert!(cfg.test_mode);

        clear_env(&keys);
    }

    #[test]
    fn load_admin_page_config_enabled_applies_env_overrides_over_kv() {
        let _lock = ENV_MUTEX.lock().unwrap();
        let keys = ["SHUMA_ADMIN_PAGE_CONFIG", "SHUMA_RATE_LIMIT", "SHUMA_HONEYPOTS"];
        clear_env(&keys);
        std::env::set_var("SHUMA_ADMIN_PAGE_CONFIG", "true");
        std::env::set_var("SHUMA_RATE_LIMIT", "222");
        std::env::set_var("SHUMA_HONEYPOTS", "[\"/trap-a\",\"/trap-b\"]");

        let store = TestStore::default();
        let mut kv_cfg = crate::config::Config::load(&store, "default");
        kv_cfg.rate_limit = 111;
        kv_cfg.honeypots = vec!["/kv-trap".to_string()];
        let key = "config:default".to_string();
        store.set(&key, &serde_json::to_vec(&kv_cfg).unwrap()).unwrap();

        let cfg = crate::config::Config::load(&store, "default");
        assert_eq!(cfg.rate_limit, 222);
        assert_eq!(cfg.honeypots, vec!["/trap-a".to_string(), "/trap-b".to_string()]);

        clear_env(&keys);
    }
}
