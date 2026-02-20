mod adapters;
mod contracts;

pub(crate) use crate::challenge::ChallengeSubmitOutcome;
pub(crate) use adapters::{
    challenge_not_a_bot_path, challenge_puzzle_path, handle_admin,
    handle_challenge_submit_with_outcome, handle_not_a_bot_submit_with_outcome, is_maze_path,
    render_challenge, render_not_a_bot, serve_challenge_page, serve_not_a_bot_page,
};
