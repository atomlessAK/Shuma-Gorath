use spin_sdk::http::Response;
use spin_sdk::key_value::Store;

pub(crate) mod not_a_bot;
pub(crate) mod pow;
mod puzzle;

#[cfg(test)]
pub(crate) use puzzle::{
    apply_transform, build_puzzle, generate_pair, parse_submission, parse_transform_count,
    select_transform_pair, transforms_for_count, ChallengeSeed, Transform,
};
pub(crate) use puzzle::render_challenge;
#[cfg(test)]
pub use puzzle::handle_challenge_submit;
pub(crate) use puzzle::{
    handle_challenge_submit_with_outcome, serve_challenge_page, ChallengeSubmitOutcome,
};
#[cfg(test)]
pub(crate) use puzzle::make_seed_token;

pub trait KeyValueStore {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ()>;
    fn set(&self, key: &str, value: &[u8]) -> Result<(), ()>;
    fn delete(&self, key: &str) -> Result<(), ()>;
    fn get_keys(&self) -> Result<Vec<String>, ()> {
        Ok(Vec::new())
    }
}

pub(crate) const PUZZLE_PATH: &str = "/challenge/puzzle";

impl KeyValueStore for Store {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ()> {
        Store::get(self, key).map_err(|_| ())
    }

    fn set(&self, key: &str, value: &[u8]) -> Result<(), ()> {
        Store::set(self, key, value).map_err(|_| ())
    }

    fn delete(&self, key: &str) -> Result<(), ()> {
        Store::delete(self, key).map_err(|_| ())
    }

    fn get_keys(&self) -> Result<Vec<String>, ()> {
        Store::get_keys(self).map_err(|_| ())
    }
}

const CHALLENGE_CACHE_CONTROL: &str = "no-store";
const CHALLENGE_CONTENT_TYPE: &str = "text/html; charset=utf-8";

pub(crate) fn challenge_response(status: u16, body: &str) -> Response {
    Response::builder()
        .status(status)
        .header("Cache-Control", CHALLENGE_CACHE_CONTROL)
        .header("Content-Type", CHALLENGE_CONTENT_TYPE)
        .body(body)
        .build()
}

#[cfg(test)]
mod tests;
