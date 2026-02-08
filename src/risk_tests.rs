// src/risk_tests.rs
// Unit tests for challenge risk scoring

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct TestStore {
        map: Mutex<HashMap<String, Vec<u8>>>,
    }

    impl TestStore {
        fn new() -> Self {
            TestStore {
                map: Mutex::new(HashMap::new()),
            }
        }
    }

    impl crate::challenge::KeyValueStore for TestStore {
        fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ()> {
            let map = self.map.lock().unwrap();
            Ok(map.get(key).cloned())
        }

        fn set(&self, key: &str, value: &[u8]) -> Result<(), ()> {
            let mut map = self.map.lock().unwrap();
            map.insert(key.to_string(), value.to_vec());
            Ok(())
        }

        fn delete(&self, key: &str) -> Result<(), ()> {
            let mut map = self.map.lock().unwrap();
            map.remove(key);
            Ok(())
        }
    }

    #[test]
    fn risk_score_accounts_for_signals() {
        let score = crate::compute_risk_score(true, true, 40, 80);
        assert_eq!(score, 4);
    }

    #[test]
    fn risk_score_handles_rate_proximity() {
        let score = crate::compute_risk_score(true, false, 70, 80);
        assert_eq!(score, 3);
    }

    #[test]
    fn risk_score_zero_when_no_signals() {
        let score = crate::compute_risk_score(false, false, 0, 80);
        assert_eq!(score, 0);
    }

    #[test]
    fn botness_assessment_uses_configured_weights() {
        let store = TestStore::new();
        let mut cfg = crate::config::Config::load(&store, "default");
        cfg.botness_weights.js_required = 3;
        cfg.botness_weights.geo_risk = 4;
        cfg.botness_weights.rate_medium = 2;
        cfg.botness_weights.rate_high = 1;

        let assessment = crate::compute_botness_assessment(true, true, 70, 80, &cfg);

        assert_eq!(assessment.score, 10);
        assert_eq!(
            assessment
                .contributions
                .iter()
                .filter(|c| c.active)
                .map(|c| c.key)
                .collect::<Vec<_>>(),
            vec![
                "js_verification_required",
                "geo_risk",
                "rate_pressure_medium",
                "rate_pressure_high"
            ]
        );
    }

    #[test]
    fn botness_assessment_applies_rate_bands_correctly() {
        let store = TestStore::new();
        let cfg = crate::config::Config::load(&store, "default");

        let medium = crate::compute_botness_assessment(false, false, 40, 80, &cfg);
        let high = crate::compute_botness_assessment(false, false, 70, 80, &cfg);

        assert_eq!(medium.score, cfg.botness_weights.rate_medium);
        assert_eq!(high.score, cfg.botness_weights.rate_medium + cfg.botness_weights.rate_high);
    }
}
