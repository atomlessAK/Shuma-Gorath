use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Simple seeded pseudo-random number generator (xorshift64).
/// We use this instead of rand crate to keep WASM size small.
pub(super) struct SeededRng {
    state: u64,
}

impl SeededRng {
    pub(super) fn new(seed: u64) -> Self {
        // Ensure non-zero state
        SeededRng {
            state: if seed == 0 { 0xDEADBEEF } else { seed },
        }
    }

    pub(super) fn next(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    pub(super) fn range(&mut self, min: usize, max: usize) -> usize {
        if min >= max {
            return min;
        }
        min + (self.next() as usize % (max - min + 1))
    }

    pub(super) fn pick(&mut self, items: &[&'static str]) -> &'static str {
        let idx = self.next() as usize % items.len();
        items[idx]
    }
}

/// Hash a path into a deterministic seed.
pub(super) fn path_to_seed(path: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    hasher.finish()
}

/// Generate a random hex string for link paths.
pub(super) fn generate_path_segment(rng: &mut SeededRng, len: usize) -> String {
    const HEX_CHARS: &[u8] = b"0123456789abcdef";
    (0..len)
        .map(|_| HEX_CHARS[rng.next() as usize % HEX_CHARS.len()] as char)
        .collect()
}
