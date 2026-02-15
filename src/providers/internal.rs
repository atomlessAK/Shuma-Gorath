use spin_sdk::http::{Request, Response};
use spin_sdk::key_value::Store;

use super::contracts::{
    BanStoreProvider, ChallengeEngineProvider, FingerprintSignalProvider, MazeTarpitProvider,
    RateLimitDecision, RateLimiterProvider,
};

pub(crate) struct InternalRateLimiterProvider;
pub(crate) struct InternalBanStoreProvider;
pub(crate) struct InternalChallengeEngineProvider;
pub(crate) struct InternalMazeTarpitProvider;
pub(crate) struct InternalFingerprintSignalProvider;

pub(crate) const RATE_LIMITER: InternalRateLimiterProvider = InternalRateLimiterProvider;
pub(crate) const BAN_STORE: InternalBanStoreProvider = InternalBanStoreProvider;
pub(crate) const CHALLENGE_ENGINE: InternalChallengeEngineProvider =
    InternalChallengeEngineProvider;
pub(crate) const MAZE_TARPIT: InternalMazeTarpitProvider = InternalMazeTarpitProvider;
pub(crate) const FINGERPRINT_SIGNAL: InternalFingerprintSignalProvider =
    InternalFingerprintSignalProvider;

impl RateLimiterProvider for InternalRateLimiterProvider {
    fn current_rate_usage(&self, store: &Store, site_id: &str, ip: &str) -> u32 {
        crate::signals::rate_pressure::current_rate_usage(store, site_id, ip)
    }

    fn check_rate_limit(
        &self,
        store: &Store,
        site_id: &str,
        ip: &str,
        limit: u32,
    ) -> RateLimitDecision {
        if crate::enforcement::rate::check_rate_limit(store, site_id, ip, limit) {
            RateLimitDecision::Allowed
        } else {
            RateLimitDecision::Limited
        }
    }
}

impl BanStoreProvider for InternalBanStoreProvider {
    fn is_banned(&self, store: &Store, site_id: &str, ip: &str) -> bool {
        crate::enforcement::ban::is_banned(store, site_id, ip)
    }

    fn list_active_bans(
        &self,
        store: &Store,
        site_id: &str,
    ) -> Vec<(String, crate::enforcement::ban::BanEntry)> {
        crate::enforcement::ban::list_active_bans_with_scan(store, site_id)
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
        crate::enforcement::ban::ban_ip_with_fingerprint(
            store,
            site_id,
            ip,
            reason,
            duration_secs,
            fingerprint,
        );
    }

    fn unban_ip(&self, store: &Store, site_id: &str, ip: &str) {
        crate::enforcement::ban::unban_ip(store, site_id, ip);
    }
}

impl ChallengeEngineProvider for InternalChallengeEngineProvider {
    fn puzzle_path(&self) -> &'static str {
        crate::boundaries::challenge_puzzle_path()
    }

    fn render_challenge(&self, req: &Request, transform_count: usize) -> Response {
        crate::boundaries::render_challenge(req, transform_count)
    }

    fn serve_challenge_page(
        &self,
        req: &Request,
        test_mode: bool,
        transform_count: usize,
    ) -> Response {
        crate::boundaries::serve_challenge_page(req, test_mode, transform_count)
    }

    fn handle_challenge_submit_with_outcome(
        &self,
        store: &Store,
        req: &Request,
    ) -> (Response, crate::challenge::ChallengeSubmitOutcome) {
        crate::boundaries::handle_challenge_submit_with_outcome(store, req)
    }

    fn handle_pow_challenge(
        &self,
        ip: &str,
        user_agent: &str,
        enabled: bool,
        difficulty: u8,
        ttl_seconds: u64,
    ) -> Response {
        crate::challenge::pow::handle_pow_challenge(
            ip,
            user_agent,
            enabled,
            difficulty,
            ttl_seconds,
        )
    }

    fn handle_pow_verify(&self, req: &Request, ip: &str, enabled: bool) -> Response {
        crate::challenge::pow::handle_pow_verify(req, ip, enabled)
    }
}

impl MazeTarpitProvider for InternalMazeTarpitProvider {
    fn is_maze_path(&self, path: &str) -> bool {
        crate::boundaries::is_maze_path(path)
    }

    fn handle_maze_request(&self, path: &str) -> Response {
        crate::boundaries::handle_maze_request(path)
    }

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
    ) -> Response {
        crate::serve_maze_with_tracking(
            req,
            store,
            cfg,
            ip,
            user_agent,
            path,
            event_reason,
            event_outcome,
        )
    }
}

impl FingerprintSignalProvider for InternalFingerprintSignalProvider {
    fn report_path(&self) -> &'static str {
        "/cdp-report"
    }

    fn source_availability(
        &self,
        cfg: &crate::config::Config,
    ) -> crate::signals::botness::SignalAvailability {
        if cfg.cdp_detection_enabled {
            crate::signals::botness::SignalAvailability::Active
        } else {
            crate::signals::botness::SignalAvailability::Disabled
        }
    }

    fn handle_report(&self, store: &Store, req: &Request) -> Response {
        crate::signals::cdp::handle_cdp_report(store, req)
    }

    fn detection_script(&self) -> &'static str {
        crate::signals::cdp::get_cdp_detection_script()
    }

    fn report_script(&self, report_endpoint: &str) -> String {
        crate::signals::cdp::get_cdp_report_script(report_endpoint)
    }

    fn inject_detection(&self, html: &str, report_endpoint: Option<&str>) -> String {
        crate::signals::cdp::inject_cdp_detection(html, report_endpoint)
    }
}
