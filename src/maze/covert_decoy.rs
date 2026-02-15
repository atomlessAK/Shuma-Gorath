use spin_sdk::http::{Method, Request, Response};
use std::time::{SystemTime, UNIX_EPOCH};

use super::token;

const DECOY_MARKER: &str = "data-shuma-covert-decoy=\"1\"";
const SEARCH_ENGINE_UA_SUBSTRINGS: &[&str] = &[
    "googlebot",
    "bingbot",
    "slurp",
    "duckduckbot",
    "baiduspider",
    "yandexbot",
    "facebot",
    "twitterbot",
    "linkedinbot",
];

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn is_search_engine_user_agent(cfg: &crate::config::Config, user_agent: &str) -> bool {
    if !cfg.robots_allow_search_engines {
        return false;
    }
    let normalized = user_agent.to_ascii_lowercase();
    SEARCH_ENGINE_UA_SUBSTRINGS
        .iter()
        .any(|needle| normalized.contains(needle))
}

fn medium_suspicion_score(cfg: &crate::config::Config, suspicion_score: u8) -> bool {
    suspicion_score >= cfg.challenge_risk_threshold && suspicion_score < cfg.botness_maze_threshold
}

fn is_html_like_response(response: &Response) -> bool {
    if let Some((_, value)) = response
        .headers()
        .find(|(name, _)| name.eq_ignore_ascii_case("content-type"))
    {
        return value
            .as_str()
            .map(|v| v.to_ascii_lowercase().contains("text/html"))
            .unwrap_or(false);
    }
    let raw = response.body();
    raw.starts_with(b"<html")
        || raw.starts_with(b"<!DOCTYPE html")
        || raw
            .windows(5)
            .any(|window| window.eq_ignore_ascii_case(b"<html"))
}

fn covert_decoy_href(
    cfg: &crate::config::Config,
    ip: &str,
    user_agent: &str,
    request_path: &str,
    now: u64,
) -> String {
    let ip_bucket = crate::signals::ip_identity::bucket_ip(ip);
    let ua_bucket = token::ua_bucket(user_agent);
    let nonce = token::flow_id_from(ip_bucket.as_str(), ua_bucket.as_str(), request_path, now);
    let path_digest = token::digest(format!("{request_path}:{ip_bucket}:{now}").as_str());
    let segment = &path_digest[..12];
    let decoy_path = format!("/maze/decoy/{segment}");
    let child = token::issue_child_token(
        None,
        decoy_path.as_str(),
        "/maze/",
        ip_bucket.as_str(),
        ua_bucket.as_str(),
        cfg.maze_token_ttl_seconds,
        cfg.maze_token_max_depth,
        cfg.maze_token_branch_budget,
        nonce.as_str(),
        99,
        now,
    );
    let signed = token::sign(&child, token::secret_from_env().as_str());
    format!("{decoy_path}?mt={signed}&dc=1")
}

fn inject_decoy_html(html: &str, href: &str) -> String {
    if html.contains(DECOY_MARKER) {
        return html.to_string();
    }
    let decoy = format!(
        r#"<div aria-hidden="true" {DECOY_MARKER} style="position:absolute;left:-10000px;top:auto;width:1px;height:1px;overflow:hidden;">
<a href="{href}" rel="nofollow" tabindex="-1">catalog index</a>
</div>"#
    );
    if let Some(idx) = html.rfind("</body>") {
        let mut out = String::with_capacity(html.len() + decoy.len() + 1);
        out.push_str(&html[..idx]);
        out.push_str(decoy.as_str());
        out.push_str(&html[idx..]);
        out
    } else {
        let mut out = String::with_capacity(html.len() + decoy.len() + 1);
        out.push_str(html);
        out.push('\n');
        out.push_str(decoy.as_str());
        out
    }
}

fn clone_response_with_body(original: &Response, body: String) -> Response {
    let mut builder = Response::builder();
    let builder = builder.status(*original.status());
    for (name, value) in original.headers() {
        if let Some(value) = value.as_str() {
            builder.header(name, value);
        }
    }
    builder.body(body).build()
}

pub(crate) fn maybe_inject_non_maze_decoy(
    req: &Request,
    cfg: &crate::config::Config,
    ip: &str,
    user_agent: &str,
    response: Response,
    suspicion_score: u8,
) -> Response {
    if !cfg.maze_enabled || !cfg.maze_covert_decoys_enabled {
        return response;
    }
    if !medium_suspicion_score(cfg, suspicion_score) {
        return response;
    }
    if *req.method() != Method::Get {
        return response;
    }
    let path = req.path();
    if super::is_maze_path(path) || path.starts_with("/admin") {
        return response;
    }
    if matches!(path, "/health" | "/metrics" | "/robots.txt") {
        return response;
    }
    if is_search_engine_user_agent(cfg, user_agent) {
        return response;
    }
    if *response.status() != 200 || !is_html_like_response(&response) {
        return response;
    }
    let Ok(html) = std::str::from_utf8(response.body()) else {
        return response;
    };

    let href = covert_decoy_href(cfg, ip, user_agent, path, now_secs());
    let updated = inject_decoy_html(html, href.as_str());
    clone_response_with_body(&response, updated)
}

#[cfg(test)]
mod tests {
    use super::maybe_inject_non_maze_decoy;
    use spin_sdk::http::{Method, Request, Response};

    fn html_response(body: &str) -> Response {
        Response::builder()
            .status(200)
            .header("Content-Type", "text/html; charset=utf-8")
            .body(body)
            .build()
    }

    fn request(path: &str) -> Request {
        Request::builder()
            .method(Method::Get)
            .uri(path)
            .body(Vec::<u8>::new())
            .build()
    }

    #[test]
    fn injects_hidden_decoy_for_medium_suspicion_html() {
        let cfg = crate::config::defaults().clone();
        let req = request("/challenge/puzzle");
        let response = html_response("<html><body><h1>Challenge</h1></body></html>");
        let injected = maybe_inject_non_maze_decoy(
            &req,
            &cfg,
            "198.51.100.9",
            "Mozilla/5.0",
            response,
            cfg.challenge_risk_threshold,
        );
        let body = String::from_utf8_lossy(injected.body());
        assert!(body.contains("data-shuma-covert-decoy=\"1\""));
        assert!(body.contains("dc=1"));
        assert!(body.contains("/maze/decoy/"));
    }

    #[test]
    fn skips_decoy_for_search_engine_user_agent() {
        let cfg = crate::config::defaults().clone();
        let req = request("/challenge/puzzle");
        let response = html_response("<html><body><h1>Challenge</h1></body></html>");
        let injected = maybe_inject_non_maze_decoy(
            &req,
            &cfg,
            "198.51.100.9",
            "Mozilla/5.0 (compatible; Googlebot/2.1)",
            response,
            cfg.challenge_risk_threshold,
        );
        let body = String::from_utf8_lossy(injected.body());
        assert!(!body.contains("data-shuma-covert-decoy=\"1\""));
    }

    #[test]
    fn skips_decoy_for_high_suspicion_maze_tier() {
        let cfg = crate::config::defaults().clone();
        let req = request("/challenge/puzzle");
        let response = html_response("<html><body><h1>Challenge</h1></body></html>");
        let injected = maybe_inject_non_maze_decoy(
            &req,
            &cfg,
            "198.51.100.9",
            "Mozilla/5.0",
            response,
            cfg.botness_maze_threshold,
        );
        let body = String::from_utf8_lossy(injected.body());
        assert!(!body.contains("data-shuma-covert-decoy=\"1\""));
    }

    #[test]
    fn skips_non_html_responses() {
        let cfg = crate::config::defaults().clone();
        let req = request("/challenge/puzzle");
        let response = Response::new(200, "plain");
        let injected = maybe_inject_non_maze_decoy(
            &req,
            &cfg,
            "198.51.100.9",
            "Mozilla/5.0",
            response,
            cfg.challenge_risk_threshold,
        );
        assert_eq!(String::from_utf8_lossy(injected.body()), "plain");
    }
}
