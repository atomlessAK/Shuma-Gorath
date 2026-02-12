mod adapters;
mod contracts;

pub(crate) use crate::challenge::ChallengeSubmitOutcome;
pub(crate) use adapters::{
    challenge_puzzle_path, handle_admin, handle_challenge_submit_with_outcome, handle_maze_request,
    is_maze_path, render_challenge, serve_challenge_page,
};
