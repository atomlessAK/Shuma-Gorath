use spin_sdk::http::{Request, Response};

pub(crate) trait ChallengeBoundary {
    fn puzzle_path(&self) -> &'static str;
    fn render_challenge(&self, req: &Request, transform_count: usize) -> Response;
    fn serve_challenge_page(
        &self,
        req: &Request,
        test_mode: bool,
        transform_count: usize,
    ) -> Response;
    fn handle_challenge_submit_with_outcome<S: crate::challenge::KeyValueStore>(
        &self,
        store: &S,
        req: &Request,
    ) -> (Response, crate::challenge::ChallengeSubmitOutcome);
}

pub(crate) trait MazeBoundary {
    fn is_maze_path(&self, path: &str) -> bool;
    fn handle_maze_request(&self, path: &str) -> Response;
}

pub(crate) trait AdminBoundary {
    fn handle_admin(&self, req: &Request) -> Response;
}
