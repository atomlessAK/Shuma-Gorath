use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NotABotSeed {
    pub operation_id: String,
    pub flow_id: String,
    pub step_id: String,
    pub step_index: u8,
    pub issued_at: u64,
    pub expires_at: u64,
    pub token_version: u8,
    pub ip_bucket: String,
    pub ua_bucket: String,
    pub path_class: String,
    pub return_to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub(crate) struct NotABotTelemetry {
    pub has_pointer: bool,
    pub pointer_move_count: u16,
    pub pointer_path_length: f32,
    pub pointer_direction_changes: u16,
    pub down_up_ms: u32,
    pub focus_changes: u8,
    pub visibility_changes: u8,
    pub interaction_elapsed_ms: u32,
    pub keyboard_used: bool,
    pub touch_used: bool,
    pub events_order_valid: bool,
    pub activation_method: String,
    pub activation_trusted: bool,
    pub activation_count: u8,
    pub control_focused: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NotABotDecision {
    Pass,
    EscalatePuzzle,
    MazeOrBlock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NotABotSubmitOutcome {
    Pass,
    EscalatePuzzle,
    MazeOrBlock,
    Replay,
    InvalidSeed,
    MissingSeed,
    Expired,
    SequenceViolation,
    BindingMismatch,
    InvalidTelemetry,
    AttemptLimitExceeded,
}

#[derive(Debug, Clone)]
pub(crate) struct NotABotSubmitResult {
    pub outcome: NotABotSubmitOutcome,
    pub decision: NotABotDecision,
    pub return_to: String,
    pub marker_cookie: Option<String>,
    pub solve_ms: Option<u64>,
}
