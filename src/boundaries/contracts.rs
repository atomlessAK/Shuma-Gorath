use spin_sdk::http::{Request, Response};

pub(crate) trait ChallengeBoundary {
    fn puzzle_path(&self) -> &'static str;
    fn not_a_bot_path(&self) -> &'static str;
    fn render_challenge(&self, req: &Request, transform_count: usize) -> Response;
    fn render_not_a_bot(&self, req: &Request, cfg: &crate::config::Config) -> Response;
    fn serve_challenge_page(
        &self,
        req: &Request,
        test_mode: bool,
        transform_count: usize,
    ) -> Response;
    fn serve_not_a_bot_page(
        &self,
        req: &Request,
        test_mode: bool,
        cfg: &crate::config::Config,
    ) -> Response;
    fn handle_challenge_submit_with_outcome<S: crate::challenge::KeyValueStore>(
        &self,
        store: &S,
        req: &Request,
    ) -> (Response, crate::challenge::ChallengeSubmitOutcome);
    fn handle_not_a_bot_submit_with_outcome<S: crate::challenge::KeyValueStore>(
        &self,
        store: &S,
        req: &Request,
        cfg: &crate::config::Config,
    ) -> crate::challenge::NotABotSubmitResult;
}

pub(crate) trait MazeBoundary {
    fn is_maze_path(&self, path: &str) -> bool;
}

pub(crate) trait AdminBoundary {
    fn handle_admin(&self, req: &Request) -> Response;
}
