use spin_sdk::http;

use super::renders::generate_maze_page;
use super::types::MazeConfig;

/// Check whether a request path targets a maze entry point.
pub fn is_maze_path(path: &str) -> bool {
    path.starts_with("/trap/") || path.starts_with("/maze/")
}

/// Handle a maze request and return the generated response.
pub fn handle_maze_request(path: &str) -> http::Response {
    let config = MazeConfig::default();
    let html = generate_maze_page(path, &config);

    http::Response::builder()
        .status(200)
        .header("Content-Type", "text/html; charset=utf-8")
        .header("Cache-Control", "no-store, no-cache, must-revalidate")
        .header("X-Robots-Tag", "noindex, nofollow")
        .body(html)
        .build()
}
