use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SignalAvailability {
    Active,
    Disabled,
    Unavailable,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct BotSignal {
    pub key: &'static str,
    pub label: &'static str,
    pub active: bool,
    pub contribution: u8,
    pub availability: SignalAvailability,
}

impl BotSignal {
    pub fn scored(key: &'static str, label: &'static str, active: bool, weight: u8) -> Self {
        let contribution = if active { weight } else { 0 };
        Self {
            key,
            label,
            active,
            contribution,
            availability: SignalAvailability::Active,
        }
    }

    pub fn disabled(key: &'static str, label: &'static str) -> Self {
        Self {
            key,
            label,
            active: false,
            contribution: 0,
            availability: SignalAvailability::Disabled,
        }
    }

    pub fn unavailable(key: &'static str, label: &'static str) -> Self {
        Self {
            key,
            label,
            active: false,
            contribution: 0,
            availability: SignalAvailability::Unavailable,
        }
    }
}

#[derive(Debug, Default)]
pub struct SignalAccumulator {
    score: u8,
    signals: Vec<BotSignal>,
}

impl SignalAccumulator {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            score: 0,
            signals: Vec::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, signal: BotSignal) {
        self.score = self.score.saturating_add(signal.contribution);
        self.signals.push(signal);
    }

    pub fn finish(self) -> (u8, Vec<BotSignal>) {
        (self.score.clamp(0, 10), self.signals)
    }
}

#[cfg(test)]
mod tests {
    use super::{BotSignal, SignalAccumulator, SignalAvailability};

    #[test]
    fn accumulator_keeps_signal_order_and_score() {
        let mut accumulator = SignalAccumulator::with_capacity(2);
        accumulator.push(BotSignal::scored("a", "A", true, 3));
        accumulator.push(BotSignal::scored("b", "B", false, 3));

        let (score, signals) = accumulator.finish();
        assert_eq!(score, 3);
        assert_eq!(signals[0].key, "a");
        assert_eq!(signals[1].key, "b");
        assert_eq!(signals[0].availability, SignalAvailability::Active);
        assert_eq!(signals[1].availability, SignalAvailability::Active);
    }

    #[test]
    fn accumulator_saturates_to_botness_cap() {
        let mut accumulator = SignalAccumulator::with_capacity(2);
        accumulator.push(BotSignal::scored("a", "A", true, 9));
        accumulator.push(BotSignal::scored("b", "B", true, 9));

        let (score, _signals) = accumulator.finish();
        assert_eq!(score, 10);
    }

    #[test]
    fn disabled_and_unavailable_signals_are_explicit_zero_contribution() {
        let disabled = BotSignal::disabled("a", "A");
        let unavailable = BotSignal::unavailable("b", "B");

        assert_eq!(disabled.contribution, 0);
        assert!(!disabled.active);
        assert_eq!(disabled.availability, SignalAvailability::Disabled);

        assert_eq!(unavailable.contribution, 0);
        assert!(!unavailable.active);
        assert_eq!(unavailable.availability, SignalAvailability::Unavailable);
    }
}
