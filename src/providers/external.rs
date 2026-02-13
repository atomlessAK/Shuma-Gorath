use spin_sdk::http::{Request, Response};
use spin_sdk::key_value::Store;
use std::time::{SystemTime, UNIX_EPOCH};

use super::contracts::{
    BanStoreProvider, BanSyncResult, ChallengeEngineProvider, FingerprintSignalProvider,
    MazeTarpitProvider, RateLimitDecision, RateLimiterProvider,
};
use super::internal;

const EXTERNAL_RATE_WINDOW_TTL_SECONDS: u64 = 120;

pub(crate) struct ExternalRateLimiterProvider;
pub(crate) struct UnsupportedExternalBanStoreProvider;
pub(crate) struct UnsupportedExternalChallengeEngineProvider;
pub(crate) struct UnsupportedExternalMazeTarpitProvider;
pub(crate) struct ExternalFingerprintSignalProvider;

pub(crate) const RATE_LIMITER: ExternalRateLimiterProvider = ExternalRateLimiterProvider;
pub(crate) const UNSUPPORTED_BAN_STORE: UnsupportedExternalBanStoreProvider =
    UnsupportedExternalBanStoreProvider;
pub(crate) const UNSUPPORTED_CHALLENGE_ENGINE: UnsupportedExternalChallengeEngineProvider =
    UnsupportedExternalChallengeEngineProvider;
pub(crate) const UNSUPPORTED_MAZE_TARPIT: UnsupportedExternalMazeTarpitProvider =
    UnsupportedExternalMazeTarpitProvider;
pub(crate) const FINGERPRINT_SIGNAL: ExternalFingerprintSignalProvider =
    ExternalFingerprintSignalProvider;

trait DistributedRateCounter {
    fn current_usage(&self, key: &str) -> Result<u32, String>;
    fn increment_and_get(&self, key: &str, ttl_seconds: u64) -> Result<u32, String>;
}

struct RedisDistributedRateCounter {
    address: String,
}

impl RedisDistributedRateCounter {
    fn from_env() -> Option<Self> {
        crate::config::rate_limiter_redis_url().map(|address| Self { address })
    }

    fn open_connection(&self) -> Result<spin_sdk::redis::Connection, String> {
        spin_sdk::redis::Connection::open(&self.address)
            .map_err(|err| format!("redis connection failed ({:?})", err))
    }
}

impl DistributedRateCounter for RedisDistributedRateCounter {
    fn current_usage(&self, key: &str) -> Result<u32, String> {
        let conn = self.open_connection()?;
        let payload = conn
            .get(key)
            .map_err(|err| format!("redis GET failed ({:?})", err))?;
        let Some(bytes) = payload else {
            return Ok(0);
        };
        let raw =
            String::from_utf8(bytes).map_err(|_| "redis payload was not UTF-8".to_string())?;
        raw.trim()
            .parse::<u32>()
            .map_err(|_| "redis payload was not a valid u32 counter".to_string())
    }

    fn increment_and_get(&self, key: &str, ttl_seconds: u64) -> Result<u32, String> {
        let conn = self.open_connection()?;
        let next = conn
            .incr(key)
            .map_err(|err| format!("redis INCR failed ({:?})", err))?;

        if next == 1 {
            let ttl = i64::try_from(ttl_seconds).unwrap_or(i64::MAX);
            let args = [
                spin_sdk::redis::RedisParameter::Binary(key.as_bytes().to_vec()),
                spin_sdk::redis::RedisParameter::Int64(ttl),
            ];
            if let Err(err) = conn.execute("EXPIRE", &args) {
                eprintln!(
                    "[providers][rate] redis EXPIRE failed for key {} ({:?})",
                    key, err
                );
            }
        }

        if next < 0 {
            return Err("redis INCR returned a negative counter".to_string());
        }
        u32::try_from(next).map_err(|_| "redis INCR exceeded u32 counter range".to_string())
    }
}

fn now_ts() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn current_window_key(site_id: &str, ip: &str, window: u64) -> String {
    let bucket = crate::signals::ip_identity::bucket_ip(ip);
    format!("rate:{}:{}:{}", site_id, bucket, window)
}

fn current_window_rate_key(site_id: &str, ip: &str) -> String {
    current_window_key(site_id, ip, now_ts() / 60)
}

fn current_rate_usage_with_backend<B: DistributedRateCounter>(
    backend: Option<&B>,
    site_id: &str,
    ip: &str,
    fallback: impl FnOnce() -> u32,
) -> u32 {
    if let Some(distributed_backend) = backend {
        let key = current_window_rate_key(site_id, ip);
        match distributed_backend.current_usage(&key) {
            Ok(count) => return count,
            Err(err) => eprintln!(
                "[providers][rate] external distributed usage read failed for key {} ({}); falling back to internal",
                key, err
            ),
        }
    }

    fallback()
}

fn check_rate_limit_with_backend<B: DistributedRateCounter>(
    backend: Option<&B>,
    site_id: &str,
    ip: &str,
    limit: u32,
    fallback: impl FnOnce() -> RateLimitDecision,
) -> RateLimitDecision {
    if limit == 0 {
        return RateLimitDecision::Limited;
    }

    if let Some(distributed_backend) = backend {
        let key = current_window_rate_key(site_id, ip);
        match distributed_backend.increment_and_get(&key, EXTERNAL_RATE_WINDOW_TTL_SECONDS) {
            Ok(next) => {
                if next > limit {
                    RateLimitDecision::Limited
                } else {
                    RateLimitDecision::Allowed
                }
            }
            Err(err) => {
                eprintln!(
                    "[providers][rate] external distributed limiter failed for key {} ({}); falling back to internal",
                    key, err
                );
                fallback()
            }
        }
    } else {
        fallback()
    }
}

impl RateLimiterProvider for ExternalRateLimiterProvider {
    fn current_rate_usage(&self, store: &Store, site_id: &str, ip: &str) -> u32 {
        let distributed_backend = RedisDistributedRateCounter::from_env();
        current_rate_usage_with_backend(distributed_backend.as_ref(), site_id, ip, || {
            internal::RATE_LIMITER.current_rate_usage(store, site_id, ip)
        })
    }

    fn check_rate_limit(
        &self,
        store: &Store,
        site_id: &str,
        ip: &str,
        limit: u32,
    ) -> RateLimitDecision {
        let distributed_backend = RedisDistributedRateCounter::from_env();
        check_rate_limit_with_backend(distributed_backend.as_ref(), site_id, ip, limit, || {
            internal::RATE_LIMITER.check_rate_limit(store, site_id, ip, limit)
        })
    }
}

impl BanStoreProvider for UnsupportedExternalBanStoreProvider {
    fn is_banned(&self, store: &Store, site_id: &str, ip: &str) -> bool {
        internal::BAN_STORE.is_banned(store, site_id, ip)
    }

    fn ban_ip_with_fingerprint(
        &self,
        store: &Store,
        site_id: &str,
        ip: &str,
        reason: &str,
        duration_secs: u64,
        fingerprint: Option<crate::enforcement::ban::BanFingerprint>,
    ) {
        internal::BAN_STORE.ban_ip_with_fingerprint(
            store,
            site_id,
            ip,
            reason,
            duration_secs,
            fingerprint,
        );
    }

    fn unban_ip(&self, store: &Store, site_id: &str, ip: &str) {
        internal::BAN_STORE.unban_ip(store, site_id, ip);
    }

    fn sync_ban(&self, _site_id: &str, _ip: &str) -> BanSyncResult {
        BanSyncResult::Failed
    }

    fn sync_unban(&self, _site_id: &str, _ip: &str) -> BanSyncResult {
        BanSyncResult::Failed
    }
}

impl ChallengeEngineProvider for UnsupportedExternalChallengeEngineProvider {
    fn puzzle_path(&self) -> &'static str {
        internal::CHALLENGE_ENGINE.puzzle_path()
    }

    fn render_challenge(&self, req: &Request, transform_count: usize) -> Response {
        internal::CHALLENGE_ENGINE.render_challenge(req, transform_count)
    }

    fn serve_challenge_page(
        &self,
        req: &Request,
        test_mode: bool,
        transform_count: usize,
    ) -> Response {
        internal::CHALLENGE_ENGINE.serve_challenge_page(req, test_mode, transform_count)
    }

    fn handle_challenge_submit_with_outcome(
        &self,
        store: &Store,
        req: &Request,
    ) -> (Response, crate::challenge::ChallengeSubmitOutcome) {
        internal::CHALLENGE_ENGINE.handle_challenge_submit_with_outcome(store, req)
    }

    fn handle_pow_challenge(
        &self,
        ip: &str,
        enabled: bool,
        difficulty: u8,
        ttl_seconds: u64,
    ) -> Response {
        internal::CHALLENGE_ENGINE.handle_pow_challenge(ip, enabled, difficulty, ttl_seconds)
    }

    fn handle_pow_verify(&self, req: &Request, ip: &str, enabled: bool) -> Response {
        internal::CHALLENGE_ENGINE.handle_pow_verify(req, ip, enabled)
    }
}

impl MazeTarpitProvider for UnsupportedExternalMazeTarpitProvider {
    fn is_maze_path(&self, path: &str) -> bool {
        internal::MAZE_TARPIT.is_maze_path(path)
    }

    fn handle_maze_request(&self, path: &str) -> Response {
        internal::MAZE_TARPIT.handle_maze_request(path)
    }

    fn serve_maze_with_tracking(
        &self,
        store: &Store,
        cfg: &crate::config::Config,
        ip: &str,
        path: &str,
        event_reason: &str,
        event_outcome: &str,
    ) -> Response {
        internal::MAZE_TARPIT.serve_maze_with_tracking(
            store,
            cfg,
            ip,
            path,
            event_reason,
            event_outcome,
        )
    }
}

impl FingerprintSignalProvider for ExternalFingerprintSignalProvider {
    fn report_path(&self) -> &'static str {
        "/fingerprint-report"
    }

    fn source_availability(
        &self,
        cfg: &crate::config::Config,
    ) -> crate::signals::botness::SignalAvailability {
        if cfg.cdp_detection_enabled {
            crate::signals::botness::SignalAvailability::Unavailable
        } else {
            crate::signals::botness::SignalAvailability::Disabled
        }
    }

    fn handle_report(&self, _store: &Store, _req: &Request) -> Response {
        Response::new(
            501,
            "External fingerprint provider selected but not configured",
        )
    }

    fn detection_script(&self) -> &'static str {
        ""
    }

    fn report_script(&self, _report_endpoint: &str) -> String {
        String::new()
    }

    fn inject_detection(&self, html: &str, _report_endpoint: Option<&str>) -> String {
        html.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        check_rate_limit_with_backend, current_rate_usage_with_backend, DistributedRateCounter,
    };
    use crate::providers::contracts::RateLimitDecision;
    use std::cell::Cell;

    #[derive(Clone)]
    struct MockDistributedRateCounter {
        current_result: Result<u32, String>,
        increment_result: Result<u32, String>,
        current_calls: Cell<u32>,
        increment_calls: Cell<u32>,
    }

    impl MockDistributedRateCounter {
        fn with_results(
            current_result: Result<u32, String>,
            increment_result: Result<u32, String>,
        ) -> Self {
            Self {
                current_result,
                increment_result,
                current_calls: Cell::new(0),
                increment_calls: Cell::new(0),
            }
        }
    }

    impl DistributedRateCounter for MockDistributedRateCounter {
        fn current_usage(&self, _key: &str) -> Result<u32, String> {
            self.current_calls.set(self.current_calls.get() + 1);
            self.current_result.clone()
        }

        fn increment_and_get(&self, _key: &str, _ttl_seconds: u64) -> Result<u32, String> {
            self.increment_calls.set(self.increment_calls.get() + 1);
            self.increment_result.clone()
        }
    }

    #[test]
    fn distributed_rate_usage_prefers_backend_when_available() {
        let backend = MockDistributedRateCounter::with_results(Ok(7), Ok(0));
        let fallback_called = Cell::new(false);
        let usage = current_rate_usage_with_backend(Some(&backend), "default", "1.2.3.4", || {
            fallback_called.set(true);
            3
        });
        assert_eq!(usage, 7);
        assert!(!fallback_called.get());
        assert_eq!(backend.current_calls.get(), 1);
    }

    #[test]
    fn distributed_rate_usage_falls_back_when_backend_errors() {
        let backend =
            MockDistributedRateCounter::with_results(Err("backend unavailable".to_string()), Ok(0));
        let fallback_called = Cell::new(false);
        let usage = current_rate_usage_with_backend(Some(&backend), "default", "1.2.3.4", || {
            fallback_called.set(true);
            5
        });
        assert_eq!(usage, 5);
        assert!(fallback_called.get());
        assert_eq!(backend.current_calls.get(), 1);
    }

    #[test]
    fn distributed_rate_limit_prefers_backend_when_available() {
        let backend = MockDistributedRateCounter::with_results(Ok(0), Ok(3));
        let fallback_called = Cell::new(false);
        let decision =
            check_rate_limit_with_backend(Some(&backend), "default", "1.2.3.4", 3, || {
                fallback_called.set(true);
                RateLimitDecision::Limited
            });
        assert_eq!(decision, RateLimitDecision::Allowed);
        assert!(!fallback_called.get());
        assert_eq!(backend.increment_calls.get(), 1);
    }

    #[test]
    fn distributed_rate_limit_blocks_when_backend_counter_exceeds_limit() {
        let backend = MockDistributedRateCounter::with_results(Ok(0), Ok(4));
        let decision =
            check_rate_limit_with_backend(Some(&backend), "default", "1.2.3.4", 3, || {
                RateLimitDecision::Allowed
            });
        assert_eq!(decision, RateLimitDecision::Limited);
        assert_eq!(backend.increment_calls.get(), 1);
    }

    #[test]
    fn distributed_rate_limit_falls_back_on_backend_error() {
        let backend =
            MockDistributedRateCounter::with_results(Ok(0), Err("backend unavailable".to_string()));
        let fallback_called = Cell::new(false);
        let decision =
            check_rate_limit_with_backend(Some(&backend), "default", "1.2.3.4", 3, || {
                fallback_called.set(true);
                RateLimitDecision::Allowed
            });
        assert_eq!(decision, RateLimitDecision::Allowed);
        assert!(fallback_called.get());
        assert_eq!(backend.increment_calls.get(), 1);
    }

    #[test]
    fn distributed_rate_limit_zero_limit_blocks_without_backend_or_fallback() {
        let backend = MockDistributedRateCounter::with_results(Ok(0), Ok(1));
        let fallback_called = Cell::new(false);
        let decision =
            check_rate_limit_with_backend(Some(&backend), "default", "1.2.3.4", 0, || {
                fallback_called.set(true);
                RateLimitDecision::Allowed
            });
        assert_eq!(decision, RateLimitDecision::Limited);
        assert!(!fallback_called.get());
        assert_eq!(backend.increment_calls.get(), 0);
    }
}
