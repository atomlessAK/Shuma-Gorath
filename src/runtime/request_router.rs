use spin_sdk::http::{Method, Request, Response};
use spin_sdk::key_value::Store;

fn record_sequence_violation_for_challenge_submit(
    store: &Store,
    req: &Request,
    transition: crate::runtime::policy_taxonomy::PolicyTransition,
    reason: &str,
) {
    let policy_match = crate::runtime::policy_taxonomy::resolve_policy_match(transition);
    crate::observability::metrics::record_policy_match(store, &policy_match);
    crate::admin::log_event(
        store,
        &crate::admin::EventLogEntry {
            ts: crate::admin::now_ts(),
            event: crate::admin::EventType::Challenge,
            ip: Some(crate::extract_client_ip(req)),
            reason: Some(reason.to_string()),
            outcome: Some(policy_match.annotate_outcome("challenge_submit_rejected")),
            admin: None,
        },
    );
}

fn handle_not_a_bot_submit(
    store: &Store,
    req: &Request,
    cfg: &crate::config::Config,
) -> Response {
    let submit_result = crate::boundaries::handle_not_a_bot_submit_with_outcome(store, req, cfg);
    let provider_registry = crate::providers::registry::ProviderRegistry::from_config(cfg);
    let ip = crate::extract_client_ip(req);
    let ua = req
        .header("user-agent")
        .and_then(|value| value.as_str())
        .unwrap_or("");
    let monitoring_outcome = match submit_result.outcome {
        crate::challenge::NotABotSubmitOutcome::Pass => "pass",
        crate::challenge::NotABotSubmitOutcome::EscalatePuzzle => "escalate",
        crate::challenge::NotABotSubmitOutcome::Replay => "replay",
        _ => "fail",
    };
    crate::observability::monitoring::record_not_a_bot_submit(
        store,
        monitoring_outcome,
        submit_result.solve_ms,
    );

    match submit_result.decision {
        crate::challenge::NotABotDecision::Pass => {
            crate::observability::metrics::increment(
                store,
                crate::observability::metrics::MetricName::NotABotPassTotal,
                None,
            );
            crate::admin::log_event(
                store,
                &crate::admin::EventLogEntry {
                    ts: crate::admin::now_ts(),
                    event: crate::admin::EventType::Challenge,
                    ip: Some(ip.clone()),
                    reason: Some("not_a_bot_pass".to_string()),
                    outcome: Some(format!(
                        "return_to={} solve_ms={}",
                        submit_result.return_to,
                        submit_result.solve_ms.unwrap_or_default()
                    )),
                    admin: None,
                },
            );
            let mut builder = Response::builder();
            builder.status(303);
            builder.header("Location", submit_result.return_to.as_str());
            builder.header("Cache-Control", "no-store");
            if let Some(marker_cookie) = submit_result.marker_cookie {
                builder.header("Set-Cookie", marker_cookie.as_str());
            }
            builder.body(Vec::new()).build()
        }
        crate::challenge::NotABotDecision::EscalatePuzzle => {
            crate::observability::metrics::increment(
                store,
                crate::observability::metrics::MetricName::NotABotEscalateTotal,
                None,
            );
            crate::admin::log_event(
                store,
                &crate::admin::EventLogEntry {
                    ts: crate::admin::now_ts(),
                    event: crate::admin::EventType::Challenge,
                    ip: Some(ip.clone()),
                    reason: Some("not_a_bot_escalate_puzzle".to_string()),
                    outcome: Some(format!("{:?}", submit_result.outcome)),
                    admin: None,
                },
            );
            if cfg.challenge_puzzle_enabled {
                crate::observability::metrics::increment(
                    store,
                    crate::observability::metrics::MetricName::ChallengesTotal,
                    None,
                );
                crate::observability::metrics::increment(
                    store,
                    crate::observability::metrics::MetricName::ChallengeServedTotal,
                    None,
                );
                return provider_registry
                    .challenge_engine_provider()
                    .render_challenge(req, cfg.challenge_puzzle_transform_count as usize);
            }
            if cfg.maze_enabled {
                return provider_registry
                    .maze_tarpit_provider()
                    .serve_maze_with_tracking(
                        req,
                        store,
                        cfg,
                        ip.as_str(),
                        ua,
                        crate::maze::entry_path("not-a-bot-escalate-fallback").as_str(),
                        "not_a_bot_escalate_puzzle_fallback_maze",
                        "not_a_bot_escalate_puzzle challenge_disabled",
                        None,
                    );
            }
            Response::new(
                403,
                crate::enforcement::block_page::render_block_page(
                    crate::enforcement::block_page::BlockReason::GeoPolicy,
                ),
            )
        }
        crate::challenge::NotABotDecision::MazeOrBlock => {
            crate::observability::metrics::increment(
                store,
                crate::observability::metrics::MetricName::NotABotFailTotal,
                None,
            );
            if submit_result.outcome == crate::challenge::NotABotSubmitOutcome::Replay {
                crate::observability::metrics::increment(
                    store,
                    crate::observability::metrics::MetricName::NotABotReplayTotal,
                    None,
                );
            }
            crate::admin::log_event(
                store,
                &crate::admin::EventLogEntry {
                    ts: crate::admin::now_ts(),
                    event: crate::admin::EventType::Challenge,
                    ip: Some(ip.clone()),
                    reason: Some("not_a_bot_fail".to_string()),
                    outcome: Some(format!("{:?}", submit_result.outcome)),
                    admin: None,
                },
            );
            if cfg.maze_enabled {
                return provider_registry
                    .maze_tarpit_provider()
                    .serve_maze_with_tracking(
                        req,
                        store,
                        cfg,
                        ip.as_str(),
                        ua,
                        crate::maze::entry_path("not-a-bot-fail").as_str(),
                        "not_a_bot_submit_fail_maze",
                        format!("{:?}", submit_result.outcome).as_str(),
                        None,
                    );
            }
            Response::new(
                403,
                crate::enforcement::block_page::render_block_page(
                    crate::enforcement::block_page::BlockReason::GeoPolicy,
                ),
            )
        }
    }
}

pub(crate) fn maybe_handle_early_route(req: &Request, path: &str) -> Option<Response> {
    if let Some(response) = crate::maze::assets::maybe_handle_asset(path, req.method()) {
        return Some(response);
    }

    if path == "/dashboard" && (*req.method() == Method::Get || *req.method() == Method::Head) {
        return Some(
            Response::builder()
                .status(308)
                .header("Location", "/dashboard/index.html")
                .header("Cache-Control", "no-store, max-age=0, must-revalidate")
                .body(Vec::new())
                .build(),
        );
    }

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

    if path == crate::boundaries::challenge_not_a_bot_path() && *req.method() == Method::Post {
        if let Ok(store) = Store::open_default() {
            let cfg = match crate::load_runtime_config(&store, "default", path) {
                Ok(cfg) => cfg,
                Err(resp) => return Some(resp),
            };
            return Some(handle_not_a_bot_submit(&store, req, &cfg));
        }
        return Some(Response::new(500, "Key-value store error"));
    }

    if path == crate::boundaries::challenge_not_a_bot_path() && *req.method() == Method::Get {
        if let Ok(store) = Store::open_default() {
            let cfg = match crate::load_runtime_config(&store, "default", path) {
                Ok(cfg) => cfg,
                Err(resp) => return Some(resp),
            };
            let response = crate::boundaries::serve_not_a_bot_page(req, cfg.test_mode, &cfg);
            if *response.status() == 200 {
                crate::observability::metrics::increment(
                    &store,
                    crate::observability::metrics::MetricName::NotABotServedTotal,
                    None,
                );
                crate::observability::monitoring::record_not_a_bot_served(&store);
            }
            return Some(response);
        }
        return Some(Response::new(500, "Key-value store error"));
    }

    // Challenge POST handler
    if path == crate::boundaries::challenge_puzzle_path() && *req.method() == Method::Post {
        if let Ok(store) = Store::open_default() {
            let (response, outcome) =
                crate::boundaries::handle_challenge_submit_with_outcome(&store, req);
            let challenge_ip = crate::extract_client_ip(req);
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
                    crate::observability::monitoring::record_challenge_failure(
                        &store,
                        challenge_ip.as_str(),
                        "incorrect",
                    );
                }
                crate::boundaries::ChallengeSubmitOutcome::SequenceOpMissing => {
                    crate::observability::monitoring::record_challenge_failure(
                        &store,
                        challenge_ip.as_str(),
                        "sequence_violation",
                    );
                    record_sequence_violation_for_challenge_submit(
                        &store,
                        req,
                        crate::runtime::policy_taxonomy::PolicyTransition::SeqOpMissing,
                        "challenge_submit_missing_operation_id",
                    );
                }
                crate::boundaries::ChallengeSubmitOutcome::SequenceOpInvalid => {
                    crate::observability::monitoring::record_challenge_failure(
                        &store,
                        challenge_ip.as_str(),
                        "sequence_violation",
                    );
                    record_sequence_violation_for_challenge_submit(
                        &store,
                        req,
                        crate::runtime::policy_taxonomy::PolicyTransition::SeqOpInvalid,
                        "challenge_submit_invalid_operation_envelope",
                    );
                }
                crate::boundaries::ChallengeSubmitOutcome::SequenceOpExpired => {
                    crate::observability::metrics::increment(
                        &store,
                        crate::observability::metrics::MetricName::ChallengeExpiredReplayTotal,
                        None,
                    );
                    crate::observability::monitoring::record_challenge_failure(
                        &store,
                        challenge_ip.as_str(),
                        "expired_replay",
                    );
                    record_sequence_violation_for_challenge_submit(
                        &store,
                        req,
                        crate::runtime::policy_taxonomy::PolicyTransition::SeqOpExpired,
                        "challenge_submit_expired_operation",
                    );
                }
                crate::boundaries::ChallengeSubmitOutcome::SequenceOpReplay => {
                    crate::observability::metrics::increment(
                        &store,
                        crate::observability::metrics::MetricName::ChallengeExpiredReplayTotal,
                        None,
                    );
                    crate::observability::monitoring::record_challenge_failure(
                        &store,
                        challenge_ip.as_str(),
                        "expired_replay",
                    );
                    record_sequence_violation_for_challenge_submit(
                        &store,
                        req,
                        crate::runtime::policy_taxonomy::PolicyTransition::SeqOpReplay,
                        "challenge_submit_operation_replay",
                    );
                }
                crate::boundaries::ChallengeSubmitOutcome::SequenceWindowExceeded => {
                    crate::observability::metrics::increment(
                        &store,
                        crate::observability::metrics::MetricName::ChallengeExpiredReplayTotal,
                        None,
                    );
                    crate::observability::monitoring::record_challenge_failure(
                        &store,
                        challenge_ip.as_str(),
                        "expired_replay",
                    );
                    record_sequence_violation_for_challenge_submit(
                        &store,
                        req,
                        crate::runtime::policy_taxonomy::PolicyTransition::SeqWindowExceeded,
                        "challenge_submit_sequence_window_exceeded",
                    );
                }
                crate::boundaries::ChallengeSubmitOutcome::SequenceOrderViolation => {
                    crate::observability::monitoring::record_challenge_failure(
                        &store,
                        challenge_ip.as_str(),
                        "sequence_violation",
                    );
                    record_sequence_violation_for_challenge_submit(
                        &store,
                        req,
                        crate::runtime::policy_taxonomy::PolicyTransition::SeqOrderViolation,
                        "challenge_submit_order_violation",
                    );
                }
                crate::boundaries::ChallengeSubmitOutcome::SequenceBindingMismatch => {
                    crate::observability::monitoring::record_challenge_failure(
                        &store,
                        challenge_ip.as_str(),
                        "sequence_violation",
                    );
                    record_sequence_violation_for_challenge_submit(
                        &store,
                        req,
                        crate::runtime::policy_taxonomy::PolicyTransition::SeqBindingMismatch,
                        "challenge_submit_binding_mismatch",
                    );
                }
                crate::boundaries::ChallengeSubmitOutcome::SequenceTimingTooFast => {
                    crate::observability::monitoring::record_challenge_failure(
                        &store,
                        challenge_ip.as_str(),
                        "sequence_violation",
                    );
                    record_sequence_violation_for_challenge_submit(
                        &store,
                        req,
                        crate::runtime::policy_taxonomy::PolicyTransition::SeqTimingTooFast,
                        "challenge_submit_timing_too_fast",
                    );
                }
                crate::boundaries::ChallengeSubmitOutcome::SequenceTimingTooRegular => {
                    crate::observability::monitoring::record_challenge_failure(
                        &store,
                        challenge_ip.as_str(),
                        "sequence_violation",
                    );
                    record_sequence_violation_for_challenge_submit(
                        &store,
                        req,
                        crate::runtime::policy_taxonomy::PolicyTransition::SeqTimingTooRegular,
                        "challenge_submit_timing_too_regular",
                    );
                }
                crate::boundaries::ChallengeSubmitOutcome::SequenceTimingTooSlow => {
                    crate::observability::monitoring::record_challenge_failure(
                        &store,
                        challenge_ip.as_str(),
                        "sequence_violation",
                    );
                    record_sequence_violation_for_challenge_submit(
                        &store,
                        req,
                        crate::runtime::policy_taxonomy::PolicyTransition::SeqTimingTooSlow,
                        "challenge_submit_timing_too_slow",
                    );
                }
                crate::boundaries::ChallengeSubmitOutcome::Forbidden => {
                    crate::observability::monitoring::record_challenge_failure(
                        &store,
                        challenge_ip.as_str(),
                        "forbidden",
                    );
                }
                crate::boundaries::ChallengeSubmitOutcome::InvalidOutput => {
                    crate::observability::monitoring::record_challenge_failure(
                        &store,
                        challenge_ip.as_str(),
                        "invalid_output",
                    );
                }
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
                cfg.challenge_puzzle_transform_count as usize,
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
