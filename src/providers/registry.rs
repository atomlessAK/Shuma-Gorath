use crate::config::{Config, ProviderBackend, ProviderBackends};

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
}

#[cfg(test)]
mod tests {
    use super::{ProviderCapability, ProviderRegistry};
    use crate::config::{defaults, ProviderBackend};

    #[test]
    fn provider_capability_has_stable_labels() {
        assert_eq!(ProviderCapability::RateLimiter.as_str(), "rate_limiter");
        assert_eq!(ProviderCapability::BanStore.as_str(), "ban_store");
        assert_eq!(ProviderCapability::ChallengeEngine.as_str(), "challenge_engine");
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
}
