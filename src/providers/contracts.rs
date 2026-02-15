use spin_sdk::http::{Request, Response};
use spin_sdk::key_value::Store;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RateLimitDecision {
    Allowed,
    Limited,
}

impl RateLimitDecision {
    pub fn as_str(self) -> &'static str {
        match self {
            RateLimitDecision::Allowed => "allowed",
            RateLimitDecision::Limited => "limited",
        }
    }
}

pub(crate) trait RateLimiterProvider {
    fn current_rate_usage(&self, store: &Store, site_id: &str, ip: &str) -> u32;
    fn check_rate_limit(
        &self,
        store: &Store,
        site_id: &str,
        ip: &str,
        limit: u32,
    ) -> RateLimitDecision;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BanSyncResult {
    Synced,
    Deferred,
    Failed,
}

impl BanSyncResult {
    pub fn as_str(self) -> &'static str {
        match self {
            BanSyncResult::Synced => "synced",
            BanSyncResult::Deferred => "deferred",
            BanSyncResult::Failed => "failed",
        }
    }
}

pub(crate) trait BanStoreProvider {
    fn is_banned(&self, store: &Store, site_id: &str, ip: &str) -> bool;
    fn list_active_bans(
        &self,
        store: &Store,
        site_id: &str,
    ) -> Vec<(String, crate::enforcement::ban::BanEntry)>;
    fn ban_ip_with_fingerprint(
        &self,
        store: &Store,
        site_id: &str,
        ip: &str,
        reason: &str,
        duration_secs: u64,
        fingerprint: Option<crate::enforcement::ban::BanFingerprint>,
    );
    fn unban_ip(&self, store: &Store, site_id: &str, ip: &str);

    fn sync_ban(&self, _site_id: &str, _ip: &str) -> BanSyncResult {
        BanSyncResult::Deferred
    }

    fn sync_unban(&self, _site_id: &str, _ip: &str) -> BanSyncResult {
        BanSyncResult::Deferred
    }
}

pub(crate) trait ChallengeEngineProvider {
    fn puzzle_path(&self) -> &'static str;
    fn render_challenge(&self, req: &Request, transform_count: usize) -> Response;
    fn serve_challenge_page(
        &self,
        req: &Request,
        test_mode: bool,
        transform_count: usize,
    ) -> Response;
    fn handle_challenge_submit_with_outcome(
        &self,
        store: &Store,
        req: &Request,
    ) -> (Response, crate::challenge::ChallengeSubmitOutcome);
    fn handle_pow_challenge(
        &self,
        ip: &str,
        user_agent: &str,
        enabled: bool,
        difficulty: u8,
        ttl_seconds: u64,
    ) -> Response;
    fn handle_pow_verify(&self, req: &Request, ip: &str, enabled: bool) -> Response;
}

pub(crate) trait MazeTarpitProvider {
    fn is_maze_path(&self, path: &str) -> bool;
    fn handle_maze_request(&self, path: &str) -> Response;
    fn serve_maze_with_tracking(
        &self,
        req: &Request,
        store: &Store,
        cfg: &crate::config::Config,
        ip: &str,
        user_agent: &str,
        path: &str,
        event_reason: &str,
        event_outcome: &str,
    ) -> Response;

    fn maybe_handle_tarpit(
        &self,
        _req: &Request,
        _store: &Store,
        _cfg: &crate::config::Config,
        _site_id: &str,
        _ip: &str,
    ) -> Option<Response> {
        None
    }
}

pub(crate) trait FingerprintSignalProvider {
    fn report_path(&self) -> &'static str;
    fn source_availability(
        &self,
        cfg: &crate::config::Config,
    ) -> crate::signals::botness::SignalAvailability;
    fn handle_report(&self, store: &Store, req: &Request) -> Response;
    fn detection_script(&self) -> &'static str;
    fn report_script(&self, report_endpoint: &str) -> String;
    fn inject_detection(&self, html: &str, report_endpoint: Option<&str>) -> String;
}

#[cfg(test)]
mod tests {
    use super::{BanStoreProvider, BanSyncResult, RateLimitDecision};
    use spin_sdk::key_value::Store;

    struct StubBanStoreProvider;

    impl BanStoreProvider for StubBanStoreProvider {
        fn is_banned(&self, _store: &Store, _site_id: &str, _ip: &str) -> bool {
            false
        }

        fn list_active_bans(
            &self,
            _store: &Store,
            _site_id: &str,
        ) -> Vec<(String, crate::enforcement::ban::BanEntry)> {
            Vec::new()
        }

        fn ban_ip_with_fingerprint(
            &self,
            _store: &Store,
            _site_id: &str,
            _ip: &str,
            _reason: &str,
            _duration_secs: u64,
            _fingerprint: Option<crate::enforcement::ban::BanFingerprint>,
        ) {
        }

        fn unban_ip(&self, _store: &Store, _site_id: &str, _ip: &str) {}
    }

    #[test]
    fn rate_limit_decision_has_stable_labels() {
        assert_eq!(RateLimitDecision::Allowed.as_str(), "allowed");
        assert_eq!(RateLimitDecision::Limited.as_str(), "limited");
    }

    #[test]
    fn ban_sync_result_has_stable_labels() {
        assert_eq!(BanSyncResult::Synced.as_str(), "synced");
        assert_eq!(BanSyncResult::Deferred.as_str(), "deferred");
        assert_eq!(BanSyncResult::Failed.as_str(), "failed");
    }

    #[test]
    fn ban_store_sync_defaults_to_deferred() {
        let provider = StubBanStoreProvider;
        assert_eq!(
            provider.sync_ban("default", "1.2.3.4"),
            BanSyncResult::Deferred
        );
        assert_eq!(
            provider.sync_unban("default", "1.2.3.4"),
            BanSyncResult::Deferred
        );
    }
}
