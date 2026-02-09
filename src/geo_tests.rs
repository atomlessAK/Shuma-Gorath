#[cfg(test)]
mod tests {
    use spin_sdk::http::{Method, Request};
    use std::collections::HashMap;
    use std::sync::Mutex;

    #[derive(Default)]
    struct TestStore {
        map: Mutex<HashMap<String, Vec<u8>>>,
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

    fn build_request(headers: &[(&str, &str)]) -> Request {
        let mut builder = Request::builder();
        builder.method(Method::Get).uri("/health");
        for (name, value) in headers {
            builder.header(*name, *value);
        }
        builder.build()
    }

    #[test]
    fn extract_geo_country_ignores_untrusted_headers() {
        let req = build_request(&[("x-geo-country", "US")]);
        let country = crate::geo::extract_geo_country(&req, false);
        assert_eq!(country, None);
    }

    #[test]
    fn extract_geo_country_trims_and_normalizes() {
        let req = build_request(&[("x-geo-country", " us ")]);
        let country = crate::geo::extract_geo_country(&req, true);
        assert_eq!(country.as_deref(), Some("US"));
    }

    #[test]
    fn extract_geo_country_rejects_non_iso_code() {
        let req = build_request(&[("x-geo-country", "zz")]);
        let country = crate::geo::extract_geo_country(&req, true);
        assert_eq!(country, None);
    }

    #[test]
    fn geo_policy_uses_most_restrictive_match_precedence() {
        let store = TestStore::default();
        let mut cfg = crate::config::Config::load(&store, "default");
        cfg.geo_allow = vec!["US".to_string()];
        cfg.geo_challenge = vec!["US".to_string()];
        cfg.geo_maze = vec!["US".to_string()];
        cfg.geo_block = vec!["US".to_string()];

        let route = crate::geo::evaluate_geo_policy(Some("US"), &cfg);
        assert_eq!(route, crate::geo::GeoPolicyRoute::Block);
    }

    #[test]
    fn geo_policy_returns_none_when_no_match() {
        let store = TestStore::default();
        let cfg = crate::config::Config::load(&store, "default");
        let route = crate::geo::evaluate_geo_policy(Some("US"), &cfg);
        assert_eq!(route, crate::geo::GeoPolicyRoute::None);
    }
}
