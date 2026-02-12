use spin_sdk::http::{Request, Response};

use super::contracts::{AdminBoundary, ChallengeBoundary, MazeBoundary};

pub(crate) struct DefaultChallengeBoundary;
pub(crate) struct DefaultMazeBoundary;
pub(crate) struct DefaultAdminBoundary;

impl ChallengeBoundary for DefaultChallengeBoundary {
    fn puzzle_path(&self) -> &'static str {
        crate::challenge::PUZZLE_PATH
    }

    fn render_challenge(&self, req: &Request, transform_count: usize) -> Response {
        crate::challenge::render_challenge(req, transform_count)
    }

    fn serve_challenge_page(
        &self,
        req: &Request,
        test_mode: bool,
        transform_count: usize,
    ) -> Response {
        crate::challenge::serve_challenge_page(req, test_mode, transform_count)
    }

    fn handle_challenge_submit_with_outcome<S: crate::challenge::KeyValueStore>(
        &self,
        store: &S,
        req: &Request,
    ) -> (Response, crate::challenge::ChallengeSubmitOutcome) {
        crate::challenge::handle_challenge_submit_with_outcome(store, req)
    }
}

impl MazeBoundary for DefaultMazeBoundary {
    fn is_maze_path(&self, path: &str) -> bool {
        crate::maze::is_maze_path(path)
    }

    fn handle_maze_request(&self, path: &str) -> Response {
        crate::maze::handle_maze_request(path)
    }
}

impl AdminBoundary for DefaultAdminBoundary {
    fn handle_admin(&self, req: &Request) -> Response {
        crate::admin::handle_admin(req)
    }
}

const CHALLENGE: DefaultChallengeBoundary = DefaultChallengeBoundary;
const MAZE: DefaultMazeBoundary = DefaultMazeBoundary;
const ADMIN: DefaultAdminBoundary = DefaultAdminBoundary;

pub(crate) fn challenge_puzzle_path() -> &'static str {
    CHALLENGE.puzzle_path()
}

pub(crate) fn render_challenge(req: &Request, transform_count: usize) -> Response {
    CHALLENGE.render_challenge(req, transform_count)
}

pub(crate) fn serve_challenge_page(
    req: &Request,
    test_mode: bool,
    transform_count: usize,
) -> Response {
    CHALLENGE.serve_challenge_page(req, test_mode, transform_count)
}

pub(crate) fn handle_challenge_submit_with_outcome<S: crate::challenge::KeyValueStore>(
    store: &S,
    req: &Request,
) -> (Response, crate::challenge::ChallengeSubmitOutcome) {
    CHALLENGE.handle_challenge_submit_with_outcome(store, req)
}

pub(crate) fn is_maze_path(path: &str) -> bool {
    MAZE.is_maze_path(path)
}

pub(crate) fn handle_maze_request(path: &str) -> Response {
    MAZE.handle_maze_request(path)
}

pub(crate) fn handle_admin(req: &Request) -> Response {
    ADMIN.handle_admin(req)
}
