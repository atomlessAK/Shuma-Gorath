// src/risk_tests.rs
// Unit tests for challenge risk scoring

#[cfg(test)]
mod tests {
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
}
