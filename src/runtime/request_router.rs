use spin_sdk::http::{Method, Request, Response};
use spin_sdk::key_value::Store;

pub(crate) fn maybe_handle_early_route(req: &Request, path: &str) -> Option<Response> {
    // Health check endpoint
    if path == "/health" {
        if !crate::health_secret_authorized(req) {
            return Some(Response::new(403, "Forbidden"));
        }
        let allowed = ["127.0.0.1", "::1"];
        let ip = crate::extract_health_client_ip(req);
        if !allowed.contains(&ip.as_str()) {
            return Some(Response::new(403, "Forbidden"));
        }
        let fail_open = crate::shuma_fail_open();
        let mode = crate::fail_mode_label(fail_open);
        if let Ok(store) = Store::open_default() {
            let test_key = "health:test";
            if let Err(e) = store.set(test_key, b"ok") {
                crate::log_line(&format!(
                    "[health] failed to write KV probe key {}: {:?}",
                    test_key, e
                ));
            }
            let ok = store.get(test_key).is_ok();
            if let Err(e) = store.delete(test_key) {
                crate::log_line(&format!(
                    "[health] failed to cleanup KV probe key {}: {:?}",
                    test_key, e
                ));
            }
            if ok {
                return Some(crate::response_with_optional_debug_headers(
                    200,
                    "OK",
                    "available",
                    mode,
                ));
            }
        }
        crate::log_line(&format!(
            "[KV OUTAGE] Key-value store unavailable; SHUMA_KV_STORE_FAIL_OPEN={}",
            fail_open
        ));
        return Some(crate::response_with_optional_debug_headers(
            500,
            "Key-value store error",
            "unavailable",
            mode,
        ));
    }

    // Challenge POST handler
    if path == crate::boundaries::challenge_puzzle_path() && *req.method() == Method::Post {
        if let Ok(store) = Store::open_default() {
            let (response, outcome) =
                crate::boundaries::handle_challenge_submit_with_outcome(&store, req);
            match outcome {
                crate::boundaries::ChallengeSubmitOutcome::Solved => {
                    crate::observability::metrics::increment(
                        &store,
                        crate::observability::metrics::MetricName::ChallengeSolvedTotal,
                        None,
                    );
                }
                crate::boundaries::ChallengeSubmitOutcome::Incorrect => {
                    crate::observability::metrics::increment(
                        &store,
                        crate::observability::metrics::MetricName::ChallengeIncorrectTotal,
                        None,
                    );
                }
                crate::boundaries::ChallengeSubmitOutcome::ExpiredReplay => {
                    crate::observability::metrics::increment(
                        &store,
                        crate::observability::metrics::MetricName::ChallengeExpiredReplayTotal,
                        None,
                    );
                }
                crate::boundaries::ChallengeSubmitOutcome::Forbidden
                | crate::boundaries::ChallengeSubmitOutcome::InvalidOutput => {}
            }
            return Some(response);
        }
        return Some(Response::new(500, "Key-value store error"));
    }
    if path == crate::boundaries::challenge_puzzle_path() && *req.method() == Method::Get {
        if let Ok(store) = Store::open_default() {
            let cfg = match crate::load_runtime_config(&store, "default", path) {
                Ok(cfg) => cfg,
                Err(resp) => return Some(resp),
            };
            let response = crate::boundaries::serve_challenge_page(
                req,
                cfg.test_mode,
                cfg.challenge_transform_count as usize,
            );
            if *response.status() == 200 {
                crate::observability::metrics::increment(
                    &store,
                    crate::observability::metrics::MetricName::ChallengeServedTotal,
                    None,
                );
            }
            return Some(response);
        }
        return Some(Response::new(500, "Key-value store error"));
    }

    // Prometheus metrics endpoint
    if path == "/metrics" {
        if let Ok(store) = Store::open_default() {
            return Some(crate::observability::metrics::handle_metrics(&store));
        }
        return Some(Response::new(500, "Key-value store error"));
    }

    // robots.txt - configurable AI crawler blocking
    if path == "/robots.txt" {
        if let Ok(store) = Store::open_default() {
            let cfg = match crate::load_runtime_config(&store, "default", path) {
                Ok(cfg) => cfg,
                Err(resp) => return Some(resp),
            };
            if cfg.robots_enabled {
                crate::observability::metrics::increment(
                    &store,
                    crate::observability::metrics::MetricName::RequestsTotal,
                    Some("robots_txt"),
                );
                let content = crate::crawler_policy::robots::generate_robots_txt(&cfg);
                let content_signal = crate::crawler_policy::robots::get_content_signal_header(&cfg);
                return Some(
                    Response::builder()
                        .status(200)
                        .header("Content-Type", "text/plain; charset=utf-8")
                        .header("Content-Signal", content_signal)
                        .header("Cache-Control", "public, max-age=3600")
                        .body(content)
                        .build(),
                );
            }
        }
        // If disabled or store error, return 404
        return Some(Response::new(404, "Not Found"));
    }

    // Admin API
    if path.starts_with("/admin") {
        if req.method() == &Method::Options {
            return Some(Response::new(403, "Forbidden"));
        }
        return Some(crate::boundaries::handle_admin(req));
    }

    None
}

#[cfg(test)]
mod tests;
