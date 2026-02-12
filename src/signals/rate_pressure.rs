use crate::challenge::KeyValueStore;
use crate::signals::ip_identity;
use std::time::{SystemTime, UNIX_EPOCH};

const RATE_MEDIUM_KEY: &str = "rate_pressure_medium";
const RATE_MEDIUM_LABEL: &str = "Rate pressure (>=50%)";
const RATE_HIGH_KEY: &str = "rate_pressure_high";
const RATE_HIGH_LABEL: &str = "Rate pressure (>=80%)";

pub fn current_rate_usage<S: KeyValueStore>(store: &S, site_id: &str, ip: &str) -> u32 {
    let key = current_window_key(site_id, ip, now_ts() / 60);
    store
        .get(&key)
        .ok()
        .flatten()
        .and_then(|v| String::from_utf8(v).ok())
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0)
}

pub fn bot_signals(
    rate_count: u32,
    rate_limit: u32,
    medium_weight: u8,
    high_weight: u8,
) -> [crate::signals::botness::BotSignal; 2] {
    if rate_limit == 0 {
        return unavailable_bot_signals();
    }

    let proximity = rate_proximity_score(rate_count, rate_limit);
    let medium_active = proximity >= 1;
    let high_active = proximity >= 2;

    [
        crate::signals::botness::BotSignal::scored(
            RATE_MEDIUM_KEY,
            RATE_MEDIUM_LABEL,
            medium_active,
            medium_weight,
        ),
        crate::signals::botness::BotSignal::scored(
            RATE_HIGH_KEY,
            RATE_HIGH_LABEL,
            high_active,
            high_weight,
        ),
    ]
}

pub fn disabled_bot_signals() -> [crate::signals::botness::BotSignal; 2] {
    [
        crate::signals::botness::BotSignal::disabled(RATE_MEDIUM_KEY, RATE_MEDIUM_LABEL),
        crate::signals::botness::BotSignal::disabled(RATE_HIGH_KEY, RATE_HIGH_LABEL),
    ]
}

fn unavailable_bot_signals() -> [crate::signals::botness::BotSignal; 2] {
    [
        crate::signals::botness::BotSignal::unavailable(RATE_MEDIUM_KEY, RATE_MEDIUM_LABEL),
        crate::signals::botness::BotSignal::unavailable(RATE_HIGH_KEY, RATE_HIGH_LABEL),
    ]
}

fn rate_proximity_score(rate_count: u32, rate_limit: u32) -> u8 {
    if rate_limit == 0 {
        return 0;
    }
    let ratio = rate_count as f32 / rate_limit as f32;
    if ratio >= 0.8 {
        2
    } else if ratio >= 0.5 {
        1
    } else {
        0
    }
}

fn current_window_key(site_id: &str, ip: &str, window: u64) -> String {
    let bucket = ip_identity::bucket_ip(ip);
    format!("rate:{}:{}:{}", site_id, bucket, window)
}

fn now_ts() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct MockStore {
        map: Mutex<HashMap<String, Vec<u8>>>,
    }

    impl MockStore {
        fn new() -> Self {
            MockStore {
                map: Mutex::new(HashMap::new()),
            }
        }
    }

    impl crate::challenge::KeyValueStore for MockStore {
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

    #[test]
    fn current_rate_usage_reads_current_window() {
        let store = MockStore::new();
        let ip = "1.2.3.4";
        let site = "default";
        let window = super::now_ts() / 60;
        let key = super::current_window_key(site, ip, window);
        store.set(&key, b"7").unwrap();
        let usage = current_rate_usage(&store, site, ip);
        assert_eq!(usage, 7);
    }

    #[test]
    fn rate_bot_signals_follow_pressure_bands() {
        let [medium, high] = bot_signals(40, 80, 2, 3);
        assert!(medium.active);
        assert_eq!(medium.contribution, 2);
        assert!(!high.active);
        assert_eq!(high.contribution, 0);

        let [medium, high] = bot_signals(70, 80, 2, 3);
        assert!(medium.active);
        assert_eq!(medium.contribution, 2);
        assert!(high.active);
        assert_eq!(high.contribution, 3);
    }

    #[test]
    fn rate_bot_signals_mark_unavailable_when_limit_zero() {
        let [medium, high] = bot_signals(10, 0, 2, 3);
        assert!(!medium.active);
        assert_eq!(
            medium.availability,
            crate::signals::botness::SignalAvailability::Unavailable
        );
        assert_eq!(medium.contribution, 0);

        assert!(!high.active);
        assert_eq!(
            high.availability,
            crate::signals::botness::SignalAvailability::Unavailable
        );
        assert_eq!(high.contribution, 0);
    }
}
