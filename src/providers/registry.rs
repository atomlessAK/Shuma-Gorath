use crate::config::{Config, ProviderBackend, ProviderBackends};
use crate::providers::contracts::{
    BanStoreProvider, ChallengeEngineProvider, FingerprintSignalProvider, MazeTarpitProvider,
    RateLimiterProvider,
};
use crate::providers::{external, internal};
use crate::signals::botness::SignalAvailability;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProviderCapability {
    RateLimiter,
    BanStore,
    ChallengeEngine,
    MazeTarpit,
    FingerprintSignal,
}

impl ProviderCapability {
    pub fn as_str(self) -> &'static str {
        match self {
            ProviderCapability::RateLimiter => "rate_limiter",
            ProviderCapability::BanStore => "ban_store",
            ProviderCapability::ChallengeEngine => "challenge_engine",
            ProviderCapability::MazeTarpit => "maze_tarpit",
            ProviderCapability::FingerprintSignal => "fingerprint_signal",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProviderRegistry {
    selections: ProviderBackends,
}

impl ProviderRegistry {
    pub fn from_config(cfg: &Config) -> Self {
        Self::from_backends(cfg.provider_backends.clone())
    }

    pub fn from_backends(selections: ProviderBackends) -> Self {
        Self { selections }
    }

    pub fn backend_for(&self, capability: ProviderCapability) -> ProviderBackend {
        match capability {
            ProviderCapability::RateLimiter => self.selections.rate_limiter,
            ProviderCapability::BanStore => self.selections.ban_store,
            ProviderCapability::ChallengeEngine => self.selections.challenge_engine,
            ProviderCapability::MazeTarpit => self.selections.maze_tarpit,
            ProviderCapability::FingerprintSignal => self.selections.fingerprint_signal,
        }
    }

    pub fn implementation_for(&self, capability: ProviderCapability) -> &'static str {
        match (capability, self.backend_for(capability)) {
            (_, ProviderBackend::Internal) => "internal",
            (ProviderCapability::RateLimiter, ProviderBackend::External) => {
                "external_redis_with_internal_fallback"
            }
            (ProviderCapability::BanStore, ProviderBackend::External) => {
                "external_redis_with_internal_fallback"
            }
            (ProviderCapability::FingerprintSignal, ProviderBackend::External) => {
                "external_stub_fingerprint"
            }
            (_, ProviderBackend::External) => "external_stub_unsupported",
        }
    }

    pub fn has_external_provider(&self) -> bool {
        [
            ProviderCapability::RateLimiter,
            ProviderCapability::BanStore,
            ProviderCapability::ChallengeEngine,
            ProviderCapability::MazeTarpit,
            ProviderCapability::FingerprintSignal,
        ]
        .iter()
        .any(|capability| self.backend_for(*capability) == ProviderBackend::External)
    }

    pub fn selections(&self) -> &ProviderBackends {
        &self.selections
    }

    pub fn rate_limiter_provider(&self) -> &'static dyn RateLimiterProvider {
        match self.backend_for(ProviderCapability::RateLimiter) {
            ProviderBackend::Internal => &internal::RATE_LIMITER,
            ProviderBackend::External => &external::RATE_LIMITER,
        }
    }

    pub fn ban_store_provider(&self) -> &'static dyn BanStoreProvider {
        match self.backend_for(ProviderCapability::BanStore) {
            ProviderBackend::Internal => &internal::BAN_STORE,
            ProviderBackend::External => &external::BAN_STORE,
        }
    }

    pub fn challenge_engine_provider(&self) -> &'static dyn ChallengeEngineProvider {
        match self.backend_for(ProviderCapability::ChallengeEngine) {
            ProviderBackend::Internal => &internal::CHALLENGE_ENGINE,
            ProviderBackend::External => &external::UNSUPPORTED_CHALLENGE_ENGINE,
        }
    }

    pub fn maze_tarpit_provider(&self) -> &'static dyn MazeTarpitProvider {
        match self.backend_for(ProviderCapability::MazeTarpit) {
            ProviderBackend::Internal => &internal::MAZE_TARPIT,
            ProviderBackend::External => &external::UNSUPPORTED_MAZE_TARPIT,
        }
    }

    pub fn fingerprint_signal_provider(&self) -> &'static dyn FingerprintSignalProvider {
        match self.backend_for(ProviderCapability::FingerprintSignal) {
            ProviderBackend::Internal => &internal::FINGERPRINT_SIGNAL,
            ProviderBackend::External => &external::FINGERPRINT_SIGNAL,
        }
    }

    pub fn fingerprint_signal_source_availability(&self, cfg: &Config) -> SignalAvailability {
        self.fingerprint_signal_provider().source_availability(cfg)
    }
}

#[cfg(test)]
mod tests {
    use super::{ProviderCapability, ProviderRegistry};
    use crate::config::{defaults, ProviderBackend};
    use crate::providers::contracts::BanSyncResult;

    #[test]
    fn provider_capability_has_stable_labels() {
        assert_eq!(ProviderCapability::RateLimiter.as_str(), "rate_limiter");
        assert_eq!(ProviderCapability::BanStore.as_str(), "ban_store");
        assert_eq!(
            ProviderCapability::ChallengeEngine.as_str(),
            "challenge_engine"
        );
        assert_eq!(ProviderCapability::MazeTarpit.as_str(), "maze_tarpit");
        assert_eq!(
            ProviderCapability::FingerprintSignal.as_str(),
            "fingerprint_signal"
        );
    }

    #[test]
    fn registry_defaults_to_internal_providers() {
        let cfg = defaults().clone();
        let registry = ProviderRegistry::from_config(&cfg);
        assert_eq!(
            registry.backend_for(ProviderCapability::RateLimiter),
            ProviderBackend::Internal
        );
        assert_eq!(
            registry.backend_for(ProviderCapability::BanStore),
            ProviderBackend::Internal
        );
        assert_eq!(
            registry.backend_for(ProviderCapability::ChallengeEngine),
            ProviderBackend::Internal
        );
        assert_eq!(
            registry.backend_for(ProviderCapability::MazeTarpit),
            ProviderBackend::Internal
        );
        assert_eq!(
            registry.backend_for(ProviderCapability::FingerprintSignal),
            ProviderBackend::Internal
        );
        assert!(!registry.has_external_provider());
    }

    #[test]
    fn registry_reflects_external_selection_from_config() {
        let mut cfg = defaults().clone();
        cfg.provider_backends.rate_limiter = ProviderBackend::External;
        cfg.provider_backends.fingerprint_signal = ProviderBackend::External;
        let registry = ProviderRegistry::from_config(&cfg);

        assert_eq!(
            registry.backend_for(ProviderCapability::RateLimiter),
            ProviderBackend::External
        );
        assert_eq!(
            registry.backend_for(ProviderCapability::FingerprintSignal),
            ProviderBackend::External
        );
        assert_eq!(
            registry.backend_for(ProviderCapability::BanStore),
            ProviderBackend::Internal
        );
        assert!(registry.has_external_provider());
    }

    #[test]
    fn registry_routes_external_fingerprint_to_stub_contract() {
        let mut cfg = defaults().clone();
        cfg.provider_backends.fingerprint_signal = ProviderBackend::External;
        let registry = ProviderRegistry::from_config(&cfg);
        let provider = registry.fingerprint_signal_provider();

        assert_eq!(provider.report_path(), "/fingerprint-report");
        assert_eq!(provider.source_availability(&cfg).as_str(), "unavailable");
        assert_eq!(provider.detection_script(), "");
        assert_eq!(provider.report_script("/report-endpoint"), "");
        assert_eq!(
            provider.inject_detection("<html><body>ok</body></html>", Some("/report-endpoint")),
            "<html><body>ok</body></html>"
        );
    }

    #[test]
    fn fingerprint_signal_contract_reports_active_disabled_unavailable_states() {
        let mut internal_cfg = defaults().clone();
        let internal_registry = ProviderRegistry::from_config(&internal_cfg);
        assert_eq!(
            internal_registry
                .fingerprint_signal_source_availability(&internal_cfg)
                .as_str(),
            "active"
        );

        internal_cfg.cdp_detection_enabled = false;
        assert_eq!(
            internal_registry
                .fingerprint_signal_source_availability(&internal_cfg)
                .as_str(),
            "disabled"
        );

        let mut external_cfg = defaults().clone();
        external_cfg.provider_backends.fingerprint_signal = ProviderBackend::External;
        let external_registry = ProviderRegistry::from_config(&external_cfg);
        assert_eq!(
            external_registry
                .fingerprint_signal_source_availability(&external_cfg)
                .as_str(),
            "unavailable"
        );

        external_cfg.cdp_detection_enabled = false;
        assert_eq!(
            external_registry
                .fingerprint_signal_source_availability(&external_cfg)
                .as_str(),
            "disabled"
        );
    }

    #[test]
    fn registry_external_backends_expose_safe_contracts() {
        let _lock = crate::test_support::lock_env();
        std::env::remove_var("SHUMA_BAN_STORE_REDIS_URL");
        let mut cfg = defaults().clone();
        cfg.provider_backends.rate_limiter = ProviderBackend::External;
        cfg.provider_backends.ban_store = ProviderBackend::External;
        cfg.provider_backends.challenge_engine = ProviderBackend::External;
        cfg.provider_backends.maze_tarpit = ProviderBackend::External;
        let registry = ProviderRegistry::from_config(&cfg);

        assert_eq!(
            registry.ban_store_provider().sync_ban("default", "1.2.3.4"),
            BanSyncResult::Failed
        );
        assert_eq!(
            registry
                .ban_store_provider()
                .sync_unban("default", "1.2.3.4"),
            BanSyncResult::Failed
        );
        assert_eq!(
            registry.challenge_engine_provider().puzzle_path(),
            crate::boundaries::challenge_puzzle_path()
        );
        assert!(registry
            .maze_tarpit_provider()
            .is_maze_path("/maze/external-stub"));

        std::env::set_var("SHUMA_BAN_STORE_REDIS_URL", "redis://redis:6379");
        assert_eq!(
            registry.ban_store_provider().sync_ban("default", "1.2.3.4"),
            BanSyncResult::Synced
        );
        assert_eq!(
            registry
                .ban_store_provider()
                .sync_unban("default", "1.2.3.4"),
            BanSyncResult::Synced
        );
        std::env::remove_var("SHUMA_BAN_STORE_REDIS_URL");
    }

    #[test]
    fn registry_reports_active_provider_implementation_labels() {
        let mut cfg = defaults().clone();
        cfg.provider_backends.rate_limiter = ProviderBackend::External;
        cfg.provider_backends.ban_store = ProviderBackend::External;
        cfg.provider_backends.fingerprint_signal = ProviderBackend::External;
        let registry = ProviderRegistry::from_config(&cfg);

        assert_eq!(
            registry.implementation_for(ProviderCapability::RateLimiter),
            "external_redis_with_internal_fallback"
        );
        assert_eq!(
            registry.implementation_for(ProviderCapability::FingerprintSignal),
            "external_stub_fingerprint"
        );
        assert_eq!(
            registry.implementation_for(ProviderCapability::BanStore),
            "external_redis_with_internal_fallback"
        );
    }
}
