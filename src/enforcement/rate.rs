use crate::challenge::KeyValueStore;
use crate::signals::ip_identity;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn check_rate_limit<S: KeyValueStore>(store: &S, site_id: &str, ip: &str, limit: u32) -> bool {
    // Bucket the IP to limit distinct keys (reduces risk of KV cardinality explosion).
    let window_key = current_window_key(site_id, ip, now_ts() / 60);
    let count = store
        .get(&window_key)
        .ok()
        .flatten()
        .and_then(|v| String::from_utf8(v).ok())
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);
    if count >= limit {
        return false;
    }
    if let Err(e) = store.set(&window_key, (count + 1).to_string().as_bytes()) {
        eprintln!(
            "[rate] failed to persist counter for key {}: {:?}",
            window_key, e
        );
    }
    true
}

fn current_window_key(site_id: &str, ip: &str, window: u64) -> String {
    let bucket = ip_identity::bucket_ip(ip);
    format!("rate:{}:{}:{}", site_id, bucket, window)
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
    fn rate_limit_buckets_and_limits() {
        let store = MockStore::new();
        let ip = "1.2.3.4";
        let site = "default";
        // Allow up to 3
        assert!(check_rate_limit(&store, site, ip, 3));
        assert!(check_rate_limit(&store, site, ip, 3));
        assert!(check_rate_limit(&store, site, ip, 3));
        // 4th should be blocked
        assert!(!check_rate_limit(&store, site, ip, 3));
    }
}

fn now_ts() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
