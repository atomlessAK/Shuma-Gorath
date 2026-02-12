use spin_sdk::http::{Request, Response};
use spin_sdk::key_value::Store;

pub(crate) fn maybe_handle_honeypot(
    store: &Store,
    cfg: &crate::config::Config,
    site_id: &str,
    ip: &str,
    path: &str,
) -> Option<Response> {
    if !crate::enforcement::honeypot::is_honeypot(path, &cfg.honeypots) {
        return None;
    }

    crate::enforcement::ban::ban_ip_with_fingerprint(
        store,
        site_id,
        ip,
        "honeypot",
        cfg.get_ban_duration("honeypot"),
        Some(crate::enforcement::ban::BanFingerprint {
            score: None,
            signals: vec!["honeypot".to_string()],
            summary: Some(format!("path={}", path)),
        }),
    );
    crate::observability::metrics::increment(
        store,
        crate::observability::metrics::MetricName::BansTotal,
        Some("honeypot"),
    );
    crate::observability::metrics::increment(store, crate::observability::metrics::MetricName::BlocksTotal, None);
    crate::admin::log_event(
        store,
        &crate::admin::EventLogEntry {
            ts: crate::admin::now_ts(),
            event: crate::admin::EventType::Ban,
            ip: Some(ip.to_string()),
            reason: Some("honeypot".to_string()),
            outcome: Some("banned".to_string()),
            admin: None,
        },
    );
    Some(Response::new(
        403,
        crate::enforcement::block_page::render_block_page(crate::enforcement::block_page::BlockReason::Honeypot),
    ))
}

pub(crate) fn maybe_handle_rate_limit(
    store: &Store,
    cfg: &crate::config::Config,
    site_id: &str,
    ip: &str,
) -> Option<Response> {
    if !cfg.rate_action_enabled() {
        return None;
    }

    if crate::enforcement::rate::check_rate_limit(store, site_id, ip, cfg.rate_limit) {
        return None;
    }

    crate::enforcement::ban::ban_ip_with_fingerprint(
        store,
        site_id,
        ip,
        "rate",
        cfg.get_ban_duration("rate"),
        Some(crate::enforcement::ban::BanFingerprint {
            score: None,
            signals: vec!["rate_limit_exceeded".to_string()],
            summary: Some(format!("rate_limit={}", cfg.rate_limit)),
        }),
    );
    crate::observability::metrics::increment(
        store,
        crate::observability::metrics::MetricName::BansTotal,
        Some("rate_limit"),
    );
    crate::observability::metrics::increment(store, crate::observability::metrics::MetricName::BlocksTotal, None);
    crate::admin::log_event(
        store,
        &crate::admin::EventLogEntry {
            ts: crate::admin::now_ts(),
            event: crate::admin::EventType::Ban,
            ip: Some(ip.to_string()),
            reason: Some("rate".to_string()),
            outcome: Some("banned".to_string()),
            admin: None,
        },
    );
    Some(Response::new(
        429,
        crate::enforcement::block_page::render_block_page(crate::enforcement::block_page::BlockReason::RateLimit),
    ))
}

pub(crate) fn maybe_handle_existing_ban(
    store: &Store,
    site_id: &str,
    ip: &str,
) -> Option<Response> {
    if !crate::enforcement::ban::is_banned(store, site_id, ip) {
        return None;
    }

    crate::observability::metrics::increment(store, crate::observability::metrics::MetricName::BlocksTotal, None);
    crate::admin::log_event(
        store,
        &crate::admin::EventLogEntry {
            ts: crate::admin::now_ts(),
            event: crate::admin::EventType::Ban,
            ip: Some(ip.to_string()),
            reason: Some("banned".to_string()),
            outcome: Some("block page".to_string()),
            admin: None,
        },
    );
    Some(Response::new(
        403,
        crate::enforcement::block_page::render_block_page(crate::enforcement::block_page::BlockReason::Honeypot),
    ))
}

pub(crate) fn maybe_handle_geo_policy(
    req: &Request,
    store: &Store,
    cfg: &crate::config::Config,
    ip: &str,
    geo_assessment: &crate::GeoAssessment,
) -> Option<Response> {
    if !cfg.geo_action_enabled() {
        return None;
    }

    match geo_assessment.route {
        crate::signals::geo::GeoPolicyRoute::Block => {
            crate::observability::metrics::increment(store, crate::observability::metrics::MetricName::BlocksTotal, None);
            crate::admin::log_event(
                store,
                &crate::admin::EventLogEntry {
                    ts: crate::admin::now_ts(),
                    event: crate::admin::EventType::Block,
                    ip: Some(ip.to_string()),
                    reason: Some("geo_policy_block".to_string()),
                    outcome: Some(format!(
                        "country={}",
                        geo_assessment.country.as_deref().unwrap_or("unknown")
                    )),
                    admin: None,
                },
            );
            Some(Response::new(
                403,
                crate::enforcement::block_page::render_block_page(crate::enforcement::block_page::BlockReason::GeoPolicy),
            ))
        }
        crate::signals::geo::GeoPolicyRoute::Maze => {
            if cfg.maze_enabled {
                return Some(crate::serve_maze_with_tracking(
                    store,
                    cfg,
                    ip,
                    "/maze/geo-policy",
                    "geo_policy_maze",
                    &format!(
                        "country={}",
                        geo_assessment.country.as_deref().unwrap_or("unknown")
                    ),
                ));
            }
            crate::observability::metrics::increment(store, crate::observability::metrics::MetricName::ChallengesTotal, None);
            crate::observability::metrics::increment(
                store,
                crate::observability::metrics::MetricName::ChallengeServedTotal,
                None,
            );
            crate::admin::log_event(
                store,
                &crate::admin::EventLogEntry {
                    ts: crate::admin::now_ts(),
                    event: crate::admin::EventType::Challenge,
                    ip: Some(ip.to_string()),
                    reason: Some("geo_policy_challenge_fallback".to_string()),
                    outcome: Some("maze_disabled".to_string()),
                    admin: None,
                },
            );
            Some(crate::boundaries::render_challenge(
                req,
                cfg.challenge_transform_count as usize,
            ))
        }
        crate::signals::geo::GeoPolicyRoute::Challenge => {
            crate::observability::metrics::increment(store, crate::observability::metrics::MetricName::ChallengesTotal, None);
            crate::observability::metrics::increment(
                store,
                crate::observability::metrics::MetricName::ChallengeServedTotal,
                None,
            );
            crate::admin::log_event(
                store,
                &crate::admin::EventLogEntry {
                    ts: crate::admin::now_ts(),
                    event: crate::admin::EventType::Challenge,
                    ip: Some(ip.to_string()),
                    reason: Some("geo_policy_challenge".to_string()),
                    outcome: Some(format!(
                        "country={}",
                        geo_assessment.country.as_deref().unwrap_or("unknown")
                    )),
                    admin: None,
                },
            );
            Some(crate::boundaries::render_challenge(
                req,
                cfg.challenge_transform_count as usize,
            ))
        }
        crate::signals::geo::GeoPolicyRoute::Allow | crate::signals::geo::GeoPolicyRoute::None => None,
    }
}

pub(crate) fn compute_needs_js(
    req: &Request,
    store: &Store,
    cfg: &crate::config::Config,
    site_id: &str,
    path: &str,
    ip: &str,
) -> bool {
    if !cfg.js_signal_enabled() && !cfg.js_action_enabled() {
        return false;
    }

    let js_missing_verification = path != "/health"
        && crate::signals::js_verification::needs_js_verification_with_whitelist(
            req,
            store,
            site_id,
            ip,
            &cfg.browser_whitelist,
        );
    js_missing_verification
}

pub(crate) fn maybe_handle_botness(
    req: &Request,
    store: &Store,
    cfg: &crate::config::Config,
    site_id: &str,
    ip: &str,
    needs_js: bool,
    geo_assessment: &crate::GeoAssessment,
) -> Option<Response> {
    let geo_risk = geo_assessment.scored_risk;
    let geo_signal_available = geo_assessment.headers_trusted && geo_assessment.country.is_some();
    let rate_usage = crate::signals::rate_pressure::current_rate_usage(store, site_id, ip);
    let botness = crate::compute_botness_assessment(
        crate::BotnessSignalContext {
            js_needed: needs_js,
            geo_signal_available,
            geo_risk,
            rate_count: rate_usage,
            rate_limit: cfg.rate_limit,
        },
        cfg,
    );
    crate::observability::metrics::record_botness_visibility(store, cfg, &botness);
    let botness_summary = crate::botness_signals_summary(&botness);
    let botness_state_summary = crate::botness_signal_states_summary(&botness);
    let mode_summary = crate::defence_modes_effective_summary(cfg);

    if cfg.maze_enabled && botness.score >= cfg.botness_maze_threshold {
        return Some(crate::serve_maze_with_tracking(
            store,
            cfg,
            ip,
            "/maze/botness-gate",
            "botness_gate_maze",
            &format!(
                "score={} signals={} signal_states={} modes={}",
                botness.score, botness_summary, botness_state_summary, mode_summary
            ),
        ));
    }

    if botness.score >= cfg.challenge_risk_threshold {
        crate::observability::metrics::increment(store, crate::observability::metrics::MetricName::ChallengesTotal, None);
        crate::observability::metrics::increment(
            store,
            crate::observability::metrics::MetricName::ChallengeServedTotal,
            None,
        );
        crate::admin::log_event(
            store,
            &crate::admin::EventLogEntry {
                ts: crate::admin::now_ts(),
                event: crate::admin::EventType::Challenge,
                ip: Some(ip.to_string()),
                reason: Some("botness_gate_challenge".to_string()),
                outcome: Some(format!(
                    "score={} signals={} signal_states={} modes={}",
                    botness.score, botness_summary, botness_state_summary, mode_summary
                )),
                admin: None,
            },
        );
        return Some(crate::boundaries::render_challenge(
            req,
            cfg.challenge_transform_count as usize,
        ));
    }

    None
}

pub(crate) fn maybe_handle_js(
    store: &Store,
    cfg: &crate::config::Config,
    ip: &str,
    needs_js: bool,
) -> Option<Response> {
    if !cfg.js_action_enabled() {
        return None;
    }
    if !needs_js {
        return None;
    }
    crate::observability::metrics::increment(store, crate::observability::metrics::MetricName::ChallengesTotal, None);
    crate::admin::log_event(
        store,
        &crate::admin::EventLogEntry {
            ts: crate::admin::now_ts(),
            event: crate::admin::EventType::Challenge,
            ip: Some(ip.to_string()),
            reason: Some("js_verification".to_string()),
            outcome: Some("js challenge".to_string()),
            admin: None,
        },
    );
    Some(crate::signals::js_verification::inject_js_challenge(
        ip,
        cfg.pow_enabled,
        cfg.pow_difficulty,
        cfg.pow_ttl_seconds,
    ))
}
