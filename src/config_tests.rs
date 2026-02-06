// src/config_tests.rs
// Unit tests for config defaults and parsing

#[cfg(test)]
mod tests {
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
    fn challenge_config_mutable_from_env_parses_values() {
        assert!(crate::config::challenge_config_mutable_from_env(Some("1")));
        assert!(crate::config::challenge_config_mutable_from_env(Some("true")));
        assert!(crate::config::challenge_config_mutable_from_env(Some("TRUE")));
        assert!(!crate::config::challenge_config_mutable_from_env(Some("0")));
        assert!(!crate::config::challenge_config_mutable_from_env(Some("false")));
        assert!(!crate::config::challenge_config_mutable_from_env(None));
    }
}
