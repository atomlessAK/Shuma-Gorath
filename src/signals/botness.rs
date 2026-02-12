use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct BotSignal {
    pub key: &'static str,
    pub label: &'static str,
    pub active: bool,
    pub contribution: u8,
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
    use super::{BotSignal, SignalAccumulator};

    #[test]
    fn accumulator_keeps_signal_order_and_score() {
        let mut accumulator = SignalAccumulator::with_capacity(2);
        accumulator.push(BotSignal {
            key: "a",
            label: "A",
            active: true,
            contribution: 3,
        });
        accumulator.push(BotSignal {
            key: "b",
            label: "B",
            active: false,
            contribution: 0,
        });

        let (score, signals) = accumulator.finish();
        assert_eq!(score, 3);
        assert_eq!(signals[0].key, "a");
        assert_eq!(signals[1].key, "b");
    }

    #[test]
    fn accumulator_saturates_to_botness_cap() {
        let mut accumulator = SignalAccumulator::with_capacity(2);
        accumulator.push(BotSignal {
            key: "a",
            label: "A",
            active: true,
            contribution: 9,
        });
        accumulator.push(BotSignal {
            key: "b",
            label: "B",
            active: true,
            contribution: 9,
        });

        let (score, _signals) = accumulator.finish();
        assert_eq!(score, 10);
    }
}
