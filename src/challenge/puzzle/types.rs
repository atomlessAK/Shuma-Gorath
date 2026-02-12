use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ChallengeSeed {
    pub seed_id: String,
    pub issued_at: u64,
    pub expires_at: u64,
    pub ip_bucket: String,
    pub grid_size: u8,
    pub active_cells: u8,
    pub transforms: Vec<Transform>,
    pub training_count: u8,
    pub seed: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Transform {
    RotateCw90,
    RotateCcw90,
    MirrorHorizontal,
    MirrorVertical,
    ShiftUp,
    ShiftDown,
    ShiftLeft,
    ShiftRight,
    DropTop,
    DropBottom,
    DropLeft,
    DropRight,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ChallengePuzzle {
    pub training_pairs: Vec<(Vec<u8>, Vec<u8>)>,
    pub test_input: Vec<u8>,
    pub test_output: Vec<u8>,
    pub grid_size: usize,
}
