// src/risk_tests.rs
// Unit tests for challenge risk scoring

#[cfg(test)]
mod tests {
    fn contribution<'a>(
        assessment: &'a crate::BotnessAssessment,
        key: &str,
    ) -> &'a crate::BotnessContribution {
        assessment
            .contributions
            .iter()
            .find(|c| c.key == key)
            .unwrap_or_else(|| panic!("missing contribution for key {}", key))
    }

    fn context(
        js_needed: bool,
        geo_signal_available: bool,
        geo_risk: bool,
        rate_count: u32,
        rate_limit: u32,
    ) -> crate::BotnessSignalContext {
        crate::BotnessSignalContext {
            js_needed,
            geo_signal_available,
            geo_risk,
            rate_count,
            rate_limit,
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
        let mut cfg = crate::config::defaults().clone();
        cfg.botness_weights.js_required = 3;
        cfg.botness_weights.geo_risk = 4;
        cfg.botness_weights.rate_medium = 2;
        cfg.botness_weights.rate_high = 1;

        let assessment = crate::compute_botness_assessment(context(true, true, true, 70, 80), &cfg);

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
        let cfg = crate::config::defaults().clone();

        let medium = crate::compute_botness_assessment(context(false, true, false, 40, 80), &cfg);
        let high = crate::compute_botness_assessment(context(false, true, false, 70, 80), &cfg);

        assert_eq!(medium.score, cfg.botness_weights.rate_medium);
        assert_eq!(
            high.score,
            cfg.botness_weights.rate_medium + cfg.botness_weights.rate_high
        );
    }

    #[test]
    fn botness_assessment_marks_disabled_and_unavailable_signals_explicitly() {
        let mut cfg = crate::config::defaults().clone();
        cfg.js_required_enforced = false;
        let assessment = crate::compute_botness_assessment(context(false, false, false, 0, 0), &cfg);

        let js = assessment
            .contributions
            .iter()
            .find(|c| c.key == "js_verification_required")
            .expect("js signal must exist");
        assert_eq!(
            js.availability,
            crate::signals::botness::SignalAvailability::Disabled
        );
        assert_eq!(js.contribution, 0);

        let geo = assessment
            .contributions
            .iter()
            .find(|c| c.key == "geo_risk")
            .expect("geo signal must exist");
        assert_eq!(
            geo.availability,
            crate::signals::botness::SignalAvailability::Unavailable
        );
        assert_eq!(geo.contribution, 0);
    }

    #[test]
    fn botness_assessment_respects_signal_modes() {
        let mut cfg = crate::config::defaults().clone();
        cfg.defence_modes.js = crate::config::ComposabilityMode::Off;
        cfg.defence_modes.geo = crate::config::ComposabilityMode::Signal;
        cfg.defence_modes.rate = crate::config::ComposabilityMode::Enforce;

        let assessment = crate::compute_botness_assessment(context(true, true, true, 70, 80), &cfg);

        let js = assessment
            .contributions
            .iter()
            .find(|c| c.key == "js_verification_required")
            .expect("js signal must exist");
        assert_eq!(
            js.availability,
            crate::signals::botness::SignalAvailability::Disabled
        );
        assert_eq!(js.contribution, 0);

        let geo = assessment
            .contributions
            .iter()
            .find(|c| c.key == "geo_risk")
            .expect("geo signal must exist");
        assert_eq!(geo.availability, crate::signals::botness::SignalAvailability::Active);
        assert_eq!(geo.contribution, cfg.botness_weights.geo_risk);

        let rate_medium = assessment
            .contributions
            .iter()
            .find(|c| c.key == "rate_pressure_medium")
            .expect("rate medium signal must exist");
        assert_eq!(
            rate_medium.availability,
            crate::signals::botness::SignalAvailability::Disabled
        );
        assert_eq!(rate_medium.contribution, 0);
    }

    #[test]
    fn botness_assessment_can_be_finalized_from_normalized_contributions() {
        let assessment = crate::compute_botness_assessment_from_contributions(vec![
            crate::BotnessContribution::scored("signal_a", "Signal A", true, 6),
            crate::BotnessContribution::disabled("signal_b", "Signal B"),
            crate::BotnessContribution::scored("signal_c", "Signal C", true, 7),
        ]);

        assert_eq!(assessment.score, 10);
        assert_eq!(
            assessment
                .contributions
                .iter()
                .map(|signal| signal.key)
                .collect::<Vec<_>>(),
            vec!["signal_a", "signal_b", "signal_c"]
        );
        assert_eq!(assessment.contributions[1].contribution, 0);
    }

    #[test]
    fn botness_mode_matrix_for_js_signal_path() {
        let cases = [
            (crate::config::ComposabilityMode::Off, false),
            (crate::config::ComposabilityMode::Signal, true),
            (crate::config::ComposabilityMode::Enforce, false),
            (crate::config::ComposabilityMode::Both, true),
        ];

        for (mode, expected_active) in cases {
            let mut cfg = crate::config::defaults().clone();
            cfg.js_required_enforced = true;
            cfg.defence_modes.js = mode;
            cfg.defence_modes.geo = crate::config::ComposabilityMode::Off;
            cfg.defence_modes.rate = crate::config::ComposabilityMode::Off;

            let assessment =
                crate::compute_botness_assessment(context(true, false, false, 0, 100), &cfg);
            let js = contribution(&assessment, "js_verification_required");

            if expected_active {
                assert_eq!(js.availability, crate::signals::botness::SignalAvailability::Active);
                assert_eq!(js.contribution, cfg.botness_weights.js_required);
            } else {
                assert_eq!(js.availability, crate::signals::botness::SignalAvailability::Disabled);
                assert_eq!(js.contribution, 0);
            }
        }
    }

    #[test]
    fn botness_mode_matrix_for_geo_signal_path() {
        let cases = [
            (crate::config::ComposabilityMode::Off, false),
            (crate::config::ComposabilityMode::Signal, true),
            (crate::config::ComposabilityMode::Enforce, false),
            (crate::config::ComposabilityMode::Both, true),
        ];

        for (mode, expected_active) in cases {
            let mut cfg = crate::config::defaults().clone();
            cfg.defence_modes.geo = mode;
            cfg.defence_modes.js = crate::config::ComposabilityMode::Off;
            cfg.defence_modes.rate = crate::config::ComposabilityMode::Off;

            let assessment =
                crate::compute_botness_assessment(context(false, true, true, 0, 100), &cfg);
            let geo = contribution(&assessment, "geo_risk");

            if expected_active {
                assert_eq!(geo.availability, crate::signals::botness::SignalAvailability::Active);
                assert_eq!(geo.contribution, cfg.botness_weights.geo_risk);
            } else {
                assert_eq!(geo.availability, crate::signals::botness::SignalAvailability::Disabled);
                assert_eq!(geo.contribution, 0);
            }
        }
    }

    #[test]
    fn botness_mode_matrix_for_rate_hybrid_signal_path() {
        let cases = [
            (crate::config::ComposabilityMode::Off, false),
            (crate::config::ComposabilityMode::Signal, true),
            (crate::config::ComposabilityMode::Enforce, false),
            (crate::config::ComposabilityMode::Both, true),
        ];

        for (mode, expected_active) in cases {
            let mut cfg = crate::config::defaults().clone();
            cfg.defence_modes.rate = mode;
            cfg.defence_modes.js = crate::config::ComposabilityMode::Off;
            cfg.defence_modes.geo = crate::config::ComposabilityMode::Off;

            let assessment =
                crate::compute_botness_assessment(context(false, false, false, 70, 80), &cfg);
            let rate_medium = contribution(&assessment, "rate_pressure_medium");
            let rate_high = contribution(&assessment, "rate_pressure_high");

            if expected_active {
                assert_eq!(
                    rate_medium.availability,
                    crate::signals::botness::SignalAvailability::Active
                );
                assert_eq!(
                    rate_high.availability,
                    crate::signals::botness::SignalAvailability::Active
                );
                assert_eq!(rate_medium.contribution, cfg.botness_weights.rate_medium);
                assert_eq!(rate_high.contribution, cfg.botness_weights.rate_high);
            } else {
                assert_eq!(
                    rate_medium.availability,
                    crate::signals::botness::SignalAvailability::Disabled
                );
                assert_eq!(
                    rate_high.availability,
                    crate::signals::botness::SignalAvailability::Disabled
                );
                assert_eq!(rate_medium.contribution, 0);
                assert_eq!(rate_high.contribution, 0);
            }
        }
    }

    #[test]
    fn botness_observability_summaries_include_signal_states_and_modes() {
        let mut cfg = crate::config::defaults().clone();
        cfg.defence_modes.rate = crate::config::ComposabilityMode::Signal;
        cfg.defence_modes.geo = crate::config::ComposabilityMode::Enforce;
        cfg.defence_modes.js = crate::config::ComposabilityMode::Both;

        let assessment =
            crate::compute_botness_assessment(context(true, true, true, 70, 80), &cfg);
        let state_summary = crate::botness_signal_states_summary(&assessment);
        assert!(state_summary.contains("js_verification_required:active:"));
        assert!(state_summary.contains("geo_risk:disabled:0"));
        assert!(state_summary.contains("rate_pressure_medium:active:"));

        let mode_summary = crate::defence_modes_effective_summary(&cfg);
        assert_eq!(
            mode_summary,
            "rate=signal/true/false geo=enforce/false/true js=both/true/true"
        );
    }
}
