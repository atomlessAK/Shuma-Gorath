use spin_sdk::http::{Request, Response};
use spin_sdk::key_value::Store;

fn active_botness_signal_ids(
    assessment: &crate::BotnessAssessment,
) -> Vec<crate::runtime::policy_taxonomy::SignalId> {
    assessment
        .contributions
        .iter()
        .filter(|contribution| contribution.active)
        .filter_map(|contribution| {
            crate::runtime::policy_taxonomy::signal_id_for_botness_key(contribution.key)
        })
        .collect()
}

fn ip_range_signal_ids(source: &crate::signals::ip_range_policy::MatchSource) -> Vec<crate::runtime::policy_taxonomy::SignalId> {
    match source {
        crate::signals::ip_range_policy::MatchSource::CustomRule => {
            vec![crate::runtime::policy_taxonomy::SignalId::IpRangeCustom]
        }
        crate::signals::ip_range_policy::MatchSource::ManagedSet => {
            vec![crate::runtime::policy_taxonomy::SignalId::IpRangeManaged]
        }
    }
}

fn ip_range_source_label(source: &crate::signals::ip_range_policy::MatchSource) -> &'static str {
    match source {
        crate::signals::ip_range_policy::MatchSource::CustomRule => "custom",
        crate::signals::ip_range_policy::MatchSource::ManagedSet => "managed",
    }
}

pub(crate) fn maybe_handle_ip_range_policy(
    req: &Request,
    store: &Store,
    cfg: &crate::config::Config,
    provider_registry: &crate::providers::registry::ProviderRegistry,
    site_id: &str,
    ip: &str,
    path: &str,
    evaluation: &crate::signals::ip_range_policy::Evaluation,
) -> Option<Response> {
    match evaluation {
        crate::signals::ip_range_policy::Evaluation::NoMatch => None,
        crate::signals::ip_range_policy::Evaluation::EmergencyAllowlisted { matched_cidr } => {
            crate::observability::metrics::increment(
                store,
                crate::observability::metrics::MetricName::WhitelistedTotal,
                None,
            );
            crate::admin::log_event(
                store,
                &crate::admin::EventLogEntry {
                    ts: crate::admin::now_ts(),
                    event: crate::admin::EventType::AdminAction,
                    ip: Some(ip.to_string()),
                    reason: Some("ip_range_emergency_allowlist".to_string()),
                    outcome: Some(format!("matched_cidr={}", matched_cidr)),
                    admin: None,
                },
            );
            Some(Response::new(200, "OK (ip range emergency allowlisted)"))
        }
        crate::signals::ip_range_policy::Evaluation::Matched(details) => {
            let source_label = ip_range_source_label(&details.source);
            let signal_ids = ip_range_signal_ids(&details.source);
            let base_outcome = format!(
                "source={} source_id={} action={} matched_cidr={}",
                source_label,
                details.source_id,
                details.action.as_str(),
                details.matched_cidr
            );

            if cfg.ip_range_policy_mode == crate::config::IpRangePolicyMode::Advisory {
                let policy_match = crate::runtime::policy_taxonomy::resolve_policy_match(
                    crate::runtime::policy_taxonomy::PolicyTransition::IpRangeAdvisory(signal_ids),
                );
                crate::observability::metrics::record_policy_match(store, &policy_match);
                crate::admin::log_event(
                    store,
                    &crate::admin::EventLogEntry {
                        ts: crate::admin::now_ts(),
                        event: crate::admin::EventType::AdminAction,
                        ip: Some(ip.to_string()),
                        reason: Some("ip_range_policy_advisory".to_string()),
                        outcome: Some(policy_match.annotate_outcome(base_outcome.as_str())),
                        admin: None,
                    },
                );
                return None;
            }

            match details.action {
                crate::config::IpRangePolicyAction::Forbidden403 => {
                    let policy_match = crate::runtime::policy_taxonomy::resolve_policy_match(
                        crate::runtime::policy_taxonomy::PolicyTransition::IpRangeForbidden(
                            signal_ids,
                        ),
                    );
                    crate::observability::metrics::record_policy_match(store, &policy_match);
                    crate::observability::metrics::increment(
                        store,
                        crate::observability::metrics::MetricName::BlocksTotal,
                        None,
                    );
                    crate::admin::log_event(
                        store,
                        &crate::admin::EventLogEntry {
                            ts: crate::admin::now_ts(),
                            event: crate::admin::EventType::Block,
                            ip: Some(ip.to_string()),
                            reason: Some("ip_range_policy_forbidden".to_string()),
                            outcome: Some(policy_match.annotate_outcome(base_outcome.as_str())),
                            admin: None,
                        },
                    );
                    Some(Response::new(
                        403,
                        crate::enforcement::block_page::render_block_page(
                            crate::enforcement::block_page::BlockReason::IpRangePolicy,
                        ),
                    ))
                }
                crate::config::IpRangePolicyAction::CustomMessage => {
                    let policy_match = crate::runtime::policy_taxonomy::resolve_policy_match(
                        crate::runtime::policy_taxonomy::PolicyTransition::IpRangeCustomMessage(
                            signal_ids,
                        ),
                    );
                    crate::observability::metrics::record_policy_match(store, &policy_match);
                    crate::observability::metrics::increment(
                        store,
                        crate::observability::metrics::MetricName::BlocksTotal,
                        None,
                    );
                    let message = details
                        .custom_message
                        .clone()
                        .unwrap_or_else(|| "Access blocked by IP range policy.".to_string());
                    crate::admin::log_event(
                        store,
                        &crate::admin::EventLogEntry {
                            ts: crate::admin::now_ts(),
                            event: crate::admin::EventType::Block,
                            ip: Some(ip.to_string()),
                            reason: Some("ip_range_policy_custom_message".to_string()),
                            outcome: Some(
                                policy_match.annotate_outcome(
                                    format!("{} message_len={}", base_outcome, message.len())
                                        .as_str(),
                                ),
                            ),
                            admin: None,
                        },
                    );
                    Some(
                        Response::builder()
                            .status(403)
                            .header("Content-Type", "text/plain; charset=utf-8")
                            .header("Cache-Control", "no-store")
                            .body(message.as_str())
                            .build(),
                    )
                }
                crate::config::IpRangePolicyAction::DropConnection => {
                    let policy_match = crate::runtime::policy_taxonomy::resolve_policy_match(
                        crate::runtime::policy_taxonomy::PolicyTransition::IpRangeDropConnection(
                            signal_ids,
                        ),
                    );
                    crate::observability::metrics::record_policy_match(store, &policy_match);
                    crate::observability::metrics::increment(
                        store,
                        crate::observability::metrics::MetricName::BlocksTotal,
                        None,
                    );
                    crate::admin::log_event(
                        store,
                        &crate::admin::EventLogEntry {
                            ts: crate::admin::now_ts(),
                            event: crate::admin::EventType::Block,
                            ip: Some(ip.to_string()),
                            reason: Some("ip_range_policy_drop_connection".to_string()),
                            outcome: Some(policy_match.annotate_outcome(base_outcome.as_str())),
                            admin: None,
                        },
                    );
                    Some(
                        Response::builder()
                            .status(444)
                            .header("Connection", "close")
                            .body("")
                            .build(),
                    )
                }
                crate::config::IpRangePolicyAction::Redirect308 => {
                    let Some(redirect_url) = details.redirect_url.clone() else {
                        let policy_match = crate::runtime::policy_taxonomy::resolve_policy_match(
                            crate::runtime::policy_taxonomy::PolicyTransition::IpRangeForbidden(
                                signal_ids,
                            ),
                        );
                        crate::observability::metrics::record_policy_match(store, &policy_match);
                        crate::observability::metrics::increment(
                            store,
                            crate::observability::metrics::MetricName::BlocksTotal,
                            None,
                        );
                        crate::admin::log_event(
                            store,
                            &crate::admin::EventLogEntry {
                                ts: crate::admin::now_ts(),
                                event: crate::admin::EventType::Block,
                                ip: Some(ip.to_string()),
                                reason: Some("ip_range_policy_redirect_missing_url".to_string()),
                                outcome: Some(policy_match.annotate_outcome(base_outcome.as_str())),
                                admin: None,
                            },
                        );
                        return Some(Response::new(
                            403,
                            crate::enforcement::block_page::render_block_page(
                                crate::enforcement::block_page::BlockReason::IpRangePolicy,
                            ),
                        ));
                    };
                    let policy_match = crate::runtime::policy_taxonomy::resolve_policy_match(
                        crate::runtime::policy_taxonomy::PolicyTransition::IpRangeRedirect(
                            signal_ids,
                        ),
                    );
                    crate::observability::metrics::record_policy_match(store, &policy_match);
                    crate::admin::log_event(
                        store,
                        &crate::admin::EventLogEntry {
                            ts: crate::admin::now_ts(),
                            event: crate::admin::EventType::AdminAction,
                            ip: Some(ip.to_string()),
                            reason: Some("ip_range_policy_redirect".to_string()),
                            outcome: Some(
                                policy_match.annotate_outcome(
                                    format!("{} location={}", base_outcome, redirect_url).as_str(),
                                ),
                            ),
                            admin: None,
                        },
                    );
                    Some(
                        Response::builder()
                            .status(308)
                            .header("Location", redirect_url.as_str())
                            .header("Cache-Control", "no-store")
                            .body("")
                            .build(),
                    )
                }
                crate::config::IpRangePolicyAction::RateLimit => {
                    let policy_match = crate::runtime::policy_taxonomy::resolve_policy_match(
                        crate::runtime::policy_taxonomy::PolicyTransition::IpRangeRateLimit(
                            signal_ids,
                        ),
                    );
                    crate::observability::metrics::record_policy_match(store, &policy_match);
                    crate::observability::metrics::increment(
                        store,
                        crate::observability::metrics::MetricName::BlocksTotal,
                        None,
                    );
                    crate::observability::monitoring::record_rate_violation_with_path(
                        store,
                        ip,
                        Some(path),
                        "limited",
                    );
                    crate::admin::log_event(
                        store,
                        &crate::admin::EventLogEntry {
                            ts: crate::admin::now_ts(),
                            event: crate::admin::EventType::Block,
                            ip: Some(ip.to_string()),
                            reason: Some("ip_range_policy_rate_limit".to_string()),
                            outcome: Some(policy_match.annotate_outcome(base_outcome.as_str())),
                            admin: None,
                        },
                    );
                    Some(Response::new(
                        429,
                        crate::enforcement::block_page::render_block_page(
                            crate::enforcement::block_page::BlockReason::RateLimit,
                        ),
                    ))
                }
                crate::config::IpRangePolicyAction::Honeypot => {
                    let policy_match = crate::runtime::policy_taxonomy::resolve_policy_match(
                        crate::runtime::policy_taxonomy::PolicyTransition::IpRangeHoneypot(
                            signal_ids,
                        ),
                    );
                    crate::observability::metrics::record_policy_match(store, &policy_match);
                    provider_registry.ban_store_provider().ban_ip_with_fingerprint(
                        store,
                        site_id,
                        ip,
                        "ip_range_honeypot",
                        cfg.get_ban_duration("honeypot"),
                        Some(crate::enforcement::ban::BanFingerprint {
                            score: None,
                            signals: vec!["ip_range_policy".to_string()],
                            summary: Some(base_outcome.clone()),
                        }),
                    );
                    crate::observability::monitoring::record_rate_violation_with_path(
                        store,
                        ip,
                        Some(path),
                        "banned",
                    );
                    crate::observability::metrics::increment(
                        store,
                        crate::observability::metrics::MetricName::BansTotal,
                        Some("ip_range_honeypot"),
                    );
                    crate::observability::metrics::increment(
                        store,
                        crate::observability::metrics::MetricName::BlocksTotal,
                        None,
                    );
                    crate::admin::log_event(
                        store,
                        &crate::admin::EventLogEntry {
                            ts: crate::admin::now_ts(),
                            event: crate::admin::EventType::Ban,
                            ip: Some(ip.to_string()),
                            reason: Some("ip_range_policy_honeypot".to_string()),
                            outcome: Some(policy_match.annotate_outcome(base_outcome.as_str())),
                            admin: None,
                        },
                    );
                    Some(Response::new(
                        403,
                        crate::enforcement::block_page::render_block_page(
                            crate::enforcement::block_page::BlockReason::Honeypot,
                        ),
                    ))
                }
                crate::config::IpRangePolicyAction::Maze => {
                    let policy_match = crate::runtime::policy_taxonomy::resolve_policy_match(
                        crate::runtime::policy_taxonomy::PolicyTransition::IpRangeMaze(signal_ids),
                    );
                    crate::observability::metrics::record_policy_match(store, &policy_match);
                    if cfg.maze_enabled {
                        let event_outcome =
                            policy_match.annotate_outcome(base_outcome.as_str());
                        return Some(
                            provider_registry
                                .maze_tarpit_provider()
                                .serve_maze_with_tracking(
                                    req,
                                    store,
                                    cfg,
                                    ip,
                                    req.header("user-agent")
                                        .map(|v| v.as_str().unwrap_or(""))
                                        .unwrap_or(""),
                                    crate::maze::entry_path("ip-range-policy").as_str(),
                                    "ip_range_policy_maze",
                                    event_outcome.as_str(),
                                    None,
                                ),
                        );
                    }
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
                        crate::admin::log_event(
                            store,
                            &crate::admin::EventLogEntry {
                                ts: crate::admin::now_ts(),
                                event: crate::admin::EventType::Challenge,
                                ip: Some(ip.to_string()),
                                reason: Some("ip_range_policy_maze_fallback_challenge".to_string()),
                                outcome: Some(
                                    policy_match.annotate_outcome(
                                        format!("{} maze_disabled", base_outcome).as_str(),
                                    ),
                                ),
                                admin: None,
                            },
                        );
                        return Some(
                            provider_registry
                                .challenge_engine_provider()
                                .render_challenge(req, cfg.challenge_puzzle_transform_count as usize),
                        );
                    }
                    crate::observability::metrics::increment(
                        store,
                        crate::observability::metrics::MetricName::BlocksTotal,
                        None,
                    );
                    crate::admin::log_event(
                        store,
                        &crate::admin::EventLogEntry {
                            ts: crate::admin::now_ts(),
                            event: crate::admin::EventType::Block,
                            ip: Some(ip.to_string()),
                            reason: Some("ip_range_policy_maze_fallback_block".to_string()),
                            outcome: Some(
                                policy_match.annotate_outcome(
                                    format!("{} maze_disabled challenge_disabled", base_outcome)
                                        .as_str(),
                                ),
                            ),
                            admin: None,
                        },
                    );
                    Some(Response::new(
                        403,
                        crate::enforcement::block_page::render_block_page(
                            crate::enforcement::block_page::BlockReason::IpRangePolicy,
                        ),
                    ))
                }
                crate::config::IpRangePolicyAction::Tarpit => {
                    let policy_match = crate::runtime::policy_taxonomy::resolve_policy_match(
                        crate::runtime::policy_taxonomy::PolicyTransition::IpRangeTarpit(signal_ids),
                    );
                    crate::observability::metrics::record_policy_match(store, &policy_match);
                    if let Some(response) = provider_registry
                        .maze_tarpit_provider()
                        .maybe_handle_tarpit(req, store, cfg, site_id, ip)
                    {
                        crate::admin::log_event(
                            store,
                            &crate::admin::EventLogEntry {
                                ts: crate::admin::now_ts(),
                                event: crate::admin::EventType::Challenge,
                                ip: Some(ip.to_string()),
                                reason: Some("ip_range_policy_tarpit".to_string()),
                                outcome: Some(policy_match.annotate_outcome(base_outcome.as_str())),
                                admin: None,
                            },
                        );
                        return Some(response);
                    }
                    if cfg.maze_enabled {
                        let event_outcome = policy_match.annotate_outcome(
                            format!("{} tarpit_unavailable fallback=maze", base_outcome).as_str(),
                        );
                        return Some(
                            provider_registry
                                .maze_tarpit_provider()
                                .serve_maze_with_tracking(
                                    req,
                                    store,
                                    cfg,
                                    ip,
                                    req.header("user-agent")
                                        .map(|v| v.as_str().unwrap_or(""))
                                        .unwrap_or(""),
                                    crate::maze::entry_path("ip-range-tarpit-fallback").as_str(),
                                    "ip_range_policy_tarpit_fallback_maze",
                                    event_outcome.as_str(),
                                    None,
                                ),
                        );
                    }
                    crate::observability::metrics::increment(
                        store,
                        crate::observability::metrics::MetricName::BlocksTotal,
                        None,
                    );
                    crate::admin::log_event(
                        store,
                        &crate::admin::EventLogEntry {
                            ts: crate::admin::now_ts(),
                            event: crate::admin::EventType::Block,
                            ip: Some(ip.to_string()),
                            reason: Some("ip_range_policy_tarpit_fallback_block".to_string()),
                            outcome: Some(
                                policy_match.annotate_outcome(
                                    format!("{} tarpit_unavailable fallback=block", base_outcome)
                                        .as_str(),
                                ),
                            ),
                            admin: None,
                        },
                    );
                    Some(Response::new(
                        403,
                        crate::enforcement::block_page::render_block_page(
                            crate::enforcement::block_page::BlockReason::IpRangePolicy,
                        ),
                    ))
                }
            }
        }
    }
}

pub(crate) fn maybe_handle_honeypot(
    store: &Store,
    cfg: &crate::config::Config,
    provider_registry: &crate::providers::registry::ProviderRegistry,
    site_id: &str,
    ip: &str,
    path: &str,
) -> Option<Response> {
    if !cfg.honeypot_enabled {
        return None;
    }
    if !crate::enforcement::honeypot::is_honeypot(path, &cfg.honeypots) {
        return None;
    }
    crate::observability::monitoring::record_honeypot_hit(store, ip, path);
    let policy_match = crate::runtime::policy_taxonomy::resolve_policy_match(
        crate::runtime::policy_taxonomy::PolicyTransition::HoneypotHit,
    );
    crate::observability::metrics::record_policy_match(store, &policy_match);

    provider_registry
        .ban_store_provider()
        .ban_ip_with_fingerprint(
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
    crate::observability::monitoring::record_rate_violation_with_path(
        store,
        ip,
        Some(path),
        "banned",
    );
    crate::observability::metrics::increment(
        store,
        crate::observability::metrics::MetricName::BansTotal,
        Some("honeypot"),
    );
    crate::observability::metrics::increment(
        store,
        crate::observability::metrics::MetricName::BlocksTotal,
        None,
    );
    crate::admin::log_event(
        store,
        &crate::admin::EventLogEntry {
            ts: crate::admin::now_ts(),
            event: crate::admin::EventType::Ban,
            ip: Some(ip.to_string()),
            reason: Some("honeypot".to_string()),
            outcome: Some(policy_match.annotate_outcome("banned")),
            admin: None,
        },
    );
    Some(Response::new(
        403,
        crate::enforcement::block_page::render_block_page(
            crate::enforcement::block_page::BlockReason::Honeypot,
        ),
    ))
}

pub(crate) fn maybe_handle_rate_limit(
    store: &Store,
    cfg: &crate::config::Config,
    provider_registry: &crate::providers::registry::ProviderRegistry,
    site_id: &str,
    ip: &str,
    path: &str,
) -> Option<Response> {
    if !cfg.rate_action_enabled() {
        return None;
    }

    if provider_registry.rate_limiter_provider().check_rate_limit(
        store,
        site_id,
        ip,
        cfg.rate_limit,
    ) == crate::providers::contracts::RateLimitDecision::Allowed
    {
        return None;
    }
    let policy_match = crate::runtime::policy_taxonomy::resolve_policy_match(
        crate::runtime::policy_taxonomy::PolicyTransition::RateLimitHit,
    );
    crate::observability::metrics::record_policy_match(store, &policy_match);

    provider_registry
        .ban_store_provider()
        .ban_ip_with_fingerprint(
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
    crate::observability::monitoring::record_rate_violation_with_path(
        store,
        ip,
        Some(path),
        "banned",
    );
    crate::observability::metrics::increment(
        store,
        crate::observability::metrics::MetricName::BansTotal,
        Some("rate_limit"),
    );
    crate::observability::metrics::increment(
        store,
        crate::observability::metrics::MetricName::BlocksTotal,
        None,
    );
    crate::admin::log_event(
        store,
        &crate::admin::EventLogEntry {
            ts: crate::admin::now_ts(),
            event: crate::admin::EventType::Ban,
            ip: Some(ip.to_string()),
            reason: Some("rate".to_string()),
            outcome: Some(policy_match.annotate_outcome("banned")),
            admin: None,
        },
    );
    Some(Response::new(
        429,
        crate::enforcement::block_page::render_block_page(
            crate::enforcement::block_page::BlockReason::RateLimit,
        ),
    ))
}

pub(crate) fn maybe_handle_existing_ban(
    store: &Store,
    provider_registry: &crate::providers::registry::ProviderRegistry,
    site_id: &str,
    ip: &str,
) -> Option<Response> {
    if !provider_registry
        .ban_store_provider()
        .is_banned(store, site_id, ip)
    {
        return None;
    }
    let policy_match = crate::runtime::policy_taxonomy::resolve_policy_match(
        crate::runtime::policy_taxonomy::PolicyTransition::ExistingBan,
    );
    crate::observability::metrics::record_policy_match(store, &policy_match);

    crate::observability::metrics::increment(
        store,
        crate::observability::metrics::MetricName::BlocksTotal,
        None,
    );
    crate::admin::log_event(
        store,
        &crate::admin::EventLogEntry {
            ts: crate::admin::now_ts(),
            event: crate::admin::EventType::Ban,
            ip: Some(ip.to_string()),
            reason: Some("banned".to_string()),
            outcome: Some(policy_match.annotate_outcome("block page")),
            admin: None,
        },
    );
    Some(Response::new(
        403,
        crate::enforcement::block_page::render_block_page(
            crate::enforcement::block_page::BlockReason::Honeypot,
        ),
    ))
}

pub(crate) fn maybe_handle_geo_policy(
    req: &Request,
    store: &Store,
    cfg: &crate::config::Config,
    provider_registry: &crate::providers::registry::ProviderRegistry,
    ip: &str,
    geo_assessment: &crate::GeoAssessment,
) -> Option<Response> {
    if !cfg.geo_action_enabled() {
        return None;
    }

    match geo_assessment.route {
        crate::signals::geo::GeoPolicyRoute::Block => {
            crate::observability::monitoring::record_geo_violation(
                store,
                geo_assessment.country.as_deref(),
                "block",
            );
            let policy_match = crate::runtime::policy_taxonomy::resolve_policy_match(
                crate::runtime::policy_taxonomy::PolicyTransition::GeoRouteBlock,
            );
            crate::observability::metrics::record_policy_match(store, &policy_match);
            crate::observability::metrics::increment(
                store,
                crate::observability::metrics::MetricName::BlocksTotal,
                None,
            );
            crate::admin::log_event(
                store,
                &crate::admin::EventLogEntry {
                    ts: crate::admin::now_ts(),
                    event: crate::admin::EventType::Block,
                    ip: Some(ip.to_string()),
                    reason: Some("geo_policy_block".to_string()),
                    outcome: Some(
                        policy_match.annotate_outcome(
                            format!(
                                "country={}",
                                geo_assessment.country.as_deref().unwrap_or("unknown")
                            )
                            .as_str(),
                        ),
                    ),
                    admin: None,
                },
            );
            Some(Response::new(
                403,
                crate::enforcement::block_page::render_block_page(
                    crate::enforcement::block_page::BlockReason::GeoPolicy,
                ),
            ))
        }
        crate::signals::geo::GeoPolicyRoute::Maze => {
            let country_summary = format!(
                "country={}",
                geo_assessment.country.as_deref().unwrap_or("unknown")
            );
            if cfg.maze_enabled {
                crate::observability::monitoring::record_geo_violation(
                    store,
                    geo_assessment.country.as_deref(),
                    "maze",
                );
                let policy_match = crate::runtime::policy_taxonomy::resolve_policy_match(
                    crate::runtime::policy_taxonomy::PolicyTransition::GeoRouteMaze,
                );
                crate::observability::metrics::record_policy_match(store, &policy_match);
                let event_outcome = policy_match.annotate_outcome(country_summary.as_str());
                return Some(
                    provider_registry
                        .maze_tarpit_provider()
                        .serve_maze_with_tracking(
                            req,
                            store,
                            cfg,
                            ip,
                            req.header("user-agent")
                                .map(|v| v.as_str().unwrap_or(""))
                                .unwrap_or(""),
                            crate::maze::entry_path("geo-policy").as_str(),
                            "geo_policy_maze",
                            event_outcome.as_str(),
                            None,
                        ),
                );
            }
            if cfg.challenge_puzzle_enabled {
                crate::observability::monitoring::record_geo_violation(
                    store,
                    geo_assessment.country.as_deref(),
                    "challenge",
                );
                let policy_match = crate::runtime::policy_taxonomy::resolve_policy_match(
                    crate::runtime::policy_taxonomy::PolicyTransition::GeoRouteMazeFallbackChallenge,
                );
                crate::observability::metrics::record_policy_match(store, &policy_match);
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
                crate::admin::log_event(
                    store,
                    &crate::admin::EventLogEntry {
                        ts: crate::admin::now_ts(),
                        event: crate::admin::EventType::Challenge,
                        ip: Some(ip.to_string()),
                        reason: Some("geo_policy_challenge_fallback".to_string()),
                        outcome: Some(policy_match.annotate_outcome("maze_disabled")),
                        admin: None,
                    },
                );
                return Some(
                    provider_registry
                        .challenge_engine_provider()
                        .render_challenge(req, cfg.challenge_puzzle_transform_count as usize),
                );
            }
            let policy_match = crate::runtime::policy_taxonomy::resolve_policy_match(
                crate::runtime::policy_taxonomy::PolicyTransition::ChallengeDisabledFallbackBlock(vec![
                    crate::runtime::policy_taxonomy::SignalId::GeoRouteMaze,
                ]),
            );
            crate::observability::monitoring::record_geo_violation(
                store,
                geo_assessment.country.as_deref(),
                "block",
            );
            crate::observability::metrics::record_policy_match(store, &policy_match);
            crate::observability::metrics::increment(
                store,
                crate::observability::metrics::MetricName::BlocksTotal,
                None,
            );
            crate::admin::log_event(
                store,
                &crate::admin::EventLogEntry {
                    ts: crate::admin::now_ts(),
                    event: crate::admin::EventType::Block,
                    ip: Some(ip.to_string()),
                    reason: Some("geo_policy_challenge_disabled_fallback_block".to_string()),
                    outcome: Some(policy_match.annotate_outcome("maze_disabled challenge_disabled")),
                    admin: None,
                },
            );
            Some(Response::new(
                403,
                crate::enforcement::block_page::render_block_page(
                    crate::enforcement::block_page::BlockReason::GeoPolicy,
                ),
            ))
        }
        crate::signals::geo::GeoPolicyRoute::Challenge => {
            let country_summary = format!(
                "country={}",
                geo_assessment.country.as_deref().unwrap_or("unknown")
            );
            if cfg.challenge_puzzle_enabled {
                crate::observability::monitoring::record_geo_violation(
                    store,
                    geo_assessment.country.as_deref(),
                    "challenge",
                );
                let policy_match = crate::runtime::policy_taxonomy::resolve_policy_match(
                    crate::runtime::policy_taxonomy::PolicyTransition::GeoRouteChallenge,
                );
                crate::observability::metrics::record_policy_match(store, &policy_match);
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
                crate::admin::log_event(
                    store,
                    &crate::admin::EventLogEntry {
                        ts: crate::admin::now_ts(),
                        event: crate::admin::EventType::Challenge,
                        ip: Some(ip.to_string()),
                        reason: Some("geo_policy_challenge".to_string()),
                        outcome: Some(policy_match.annotate_outcome(country_summary.as_str())),
                        admin: None,
                    },
                );
                return Some(
                    provider_registry
                        .challenge_engine_provider()
                        .render_challenge(req, cfg.challenge_puzzle_transform_count as usize),
                );
            }
            if cfg.maze_enabled {
                crate::observability::monitoring::record_geo_violation(
                    store,
                    geo_assessment.country.as_deref(),
                    "maze",
                );
                let policy_match = crate::runtime::policy_taxonomy::resolve_policy_match(
                    crate::runtime::policy_taxonomy::PolicyTransition::ChallengeDisabledFallbackMaze(vec![
                        crate::runtime::policy_taxonomy::SignalId::GeoRouteChallenge,
                    ]),
                );
                crate::observability::metrics::record_policy_match(store, &policy_match);
                let event_outcome = policy_match.annotate_outcome(
                    format!("{} challenge_disabled", country_summary).as_str(),
                );
                return Some(
                    provider_registry
                        .maze_tarpit_provider()
                        .serve_maze_with_tracking(
                            req,
                            store,
                            cfg,
                            ip,
                            req.header("user-agent")
                                .map(|v| v.as_str().unwrap_or(""))
                                .unwrap_or(""),
                            crate::maze::entry_path("geo-policy-challenge-fallback").as_str(),
                            "geo_policy_challenge_fallback_maze",
                            event_outcome.as_str(),
                            None,
                        ),
                );
            }
            let policy_match = crate::runtime::policy_taxonomy::resolve_policy_match(
                crate::runtime::policy_taxonomy::PolicyTransition::ChallengeDisabledFallbackBlock(vec![
                    crate::runtime::policy_taxonomy::SignalId::GeoRouteChallenge,
                ]),
            );
            crate::observability::monitoring::record_geo_violation(
                store,
                geo_assessment.country.as_deref(),
                "block",
            );
            crate::observability::metrics::record_policy_match(store, &policy_match);
            crate::observability::metrics::increment(
                store,
                crate::observability::metrics::MetricName::BlocksTotal,
                None,
            );
            crate::admin::log_event(
                store,
                &crate::admin::EventLogEntry {
                    ts: crate::admin::now_ts(),
                    event: crate::admin::EventType::Block,
                    ip: Some(ip.to_string()),
                    reason: Some("geo_policy_challenge_disabled_fallback_block".to_string()),
                    outcome: Some(policy_match.annotate_outcome("challenge_disabled maze_disabled")),
                    admin: None,
                },
            );
            Some(Response::new(
                403,
                crate::enforcement::block_page::render_block_page(
                    crate::enforcement::block_page::BlockReason::GeoPolicy,
                ),
            ))
        }
        crate::signals::geo::GeoPolicyRoute::Allow | crate::signals::geo::GeoPolicyRoute::None => {
            None
        }
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
    provider_registry: &crate::providers::registry::ProviderRegistry,
    site_id: &str,
    ip: &str,
    needs_js: bool,
    geo_assessment: &crate::GeoAssessment,
) -> Option<Response> {
    let geo_risk = geo_assessment.scored_risk;
    let geo_signal_available = geo_assessment.headers_trusted && geo_assessment.country.is_some();
    let rate_usage = provider_registry
        .rate_limiter_provider()
        .current_rate_usage(store, site_id, ip);
    let maze_behavior_score = crate::maze::runtime::current_behavior_score(store, ip);
    let fingerprint_signals = crate::signals::fingerprint::collect_bot_signals(
        store,
        req,
        cfg,
        ip,
        geo_assessment.headers_trusted,
    );
    let botness = crate::compute_botness_assessment(
        crate::BotnessSignalContext {
            js_needed: needs_js,
            geo_signal_available,
            geo_risk,
            rate_count: rate_usage,
            rate_limit: cfg.rate_limit,
            maze_behavior_score,
            fingerprint_signals,
        },
        cfg,
    );
    crate::observability::metrics::record_botness_visibility(store, cfg, &botness);
    let botness_summary = crate::botness_signals_summary(&botness);
    let botness_state_summary = crate::botness_signal_states_summary(&botness);
    let runtime_metadata_summary = crate::defence_runtime_metadata_summary(cfg);
    let provider_summary = crate::provider_implementations_summary(provider_registry);
    let ua = req
        .header("user-agent")
        .map(|v| v.as_str().unwrap_or(""))
        .unwrap_or("");
    let base_outcome = format!(
        "score={} signals={} signal_states={} {} providers={}",
        botness.score, botness_summary, botness_state_summary, runtime_metadata_summary, provider_summary
    );
    let botness_signal_ids = active_botness_signal_ids(&botness);

    if cfg.maze_enabled && botness.score >= cfg.botness_maze_threshold {
        let policy_match = crate::runtime::policy_taxonomy::resolve_policy_match(
            crate::runtime::policy_taxonomy::PolicyTransition::BotnessGateMaze(
                botness_signal_ids.clone(),
            ),
        );
        crate::observability::metrics::record_policy_match(store, &policy_match);
        let event_outcome = policy_match.annotate_outcome(
            format!(
                "score={} signals={} signal_states={} {} providers={}",
                botness.score,
                botness_summary,
                botness_state_summary,
                runtime_metadata_summary,
                provider_summary
            )
            .as_str(),
        );
        return Some(
            provider_registry
                .maze_tarpit_provider()
                .serve_maze_with_tracking(
                    req,
                    store,
                    cfg,
                    ip,
                    req.header("user-agent")
                        .map(|v| v.as_str().unwrap_or(""))
                        .unwrap_or(""),
                    crate::maze::entry_path("botness-gate").as_str(),
                    "botness_gate_maze",
                    event_outcome.as_str(),
                    Some(botness.score),
                ),
        );
    }

    let not_a_bot_threshold = cfg.not_a_bot_risk_threshold;
    if cfg.not_a_bot_enabled
        && cfg.challenge_puzzle_enabled
        && not_a_bot_threshold > 0
        && botness.score >= not_a_bot_threshold
        && botness.score < cfg.challenge_puzzle_risk_threshold
    {
        if crate::challenge::has_valid_not_a_bot_marker(req, ip, ua) {
            return None;
        }
        let policy_match = crate::runtime::policy_taxonomy::resolve_policy_match(
            crate::runtime::policy_taxonomy::PolicyTransition::BotnessGateNotABot(
                botness_signal_ids.clone(),
            ),
        );
        crate::observability::metrics::record_policy_match(store, &policy_match);
        crate::observability::metrics::increment(
            store,
            crate::observability::metrics::MetricName::ChallengesTotal,
            None,
        );
        crate::observability::metrics::increment(
            store,
            crate::observability::metrics::MetricName::NotABotServedTotal,
            None,
        );
        crate::observability::monitoring::record_not_a_bot_served(store);
        crate::admin::log_event(
            store,
            &crate::admin::EventLogEntry {
                ts: crate::admin::now_ts(),
                event: crate::admin::EventType::Challenge,
                ip: Some(ip.to_string()),
                reason: Some("botness_gate_not_a_bot".to_string()),
                outcome: Some(policy_match.annotate_outcome(base_outcome.as_str())),
                admin: None,
            },
        );
        let not_a_bot_response = provider_registry
            .challenge_engine_provider()
            .render_not_a_bot(req, cfg);
        let response = crate::maze::covert_decoy::maybe_inject_non_maze_decoy(
            req,
            cfg,
            ip,
            ua,
            not_a_bot_response,
            botness.score,
        );
        return Some(response);
    }

    if botness.score >= cfg.challenge_puzzle_risk_threshold {
        if cfg.challenge_puzzle_enabled {
            let policy_match = crate::runtime::policy_taxonomy::resolve_policy_match(
                crate::runtime::policy_taxonomy::PolicyTransition::BotnessGateChallenge(
                    botness_signal_ids,
                ),
            );
            crate::observability::metrics::record_policy_match(store, &policy_match);
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
            crate::admin::log_event(
                store,
                &crate::admin::EventLogEntry {
                    ts: crate::admin::now_ts(),
                    event: crate::admin::EventType::Challenge,
                    ip: Some(ip.to_string()),
                    reason: Some("botness_gate_challenge".to_string()),
                    outcome: Some(policy_match.annotate_outcome(base_outcome.as_str())),
                    admin: None,
                },
            );
            let challenge_response = provider_registry
                .challenge_engine_provider()
                .render_challenge(req, cfg.challenge_puzzle_transform_count as usize);
            let response = crate::maze::covert_decoy::maybe_inject_non_maze_decoy(
                req,
                cfg,
                ip,
                ua,
                challenge_response,
                botness.score,
            );
            return Some(response);
        }
        if cfg.maze_enabled {
            let policy_match = crate::runtime::policy_taxonomy::resolve_policy_match(
                crate::runtime::policy_taxonomy::PolicyTransition::ChallengeDisabledFallbackMaze(
                    botness_signal_ids.clone(),
                ),
            );
            crate::observability::metrics::record_policy_match(store, &policy_match);
            let event_outcome = policy_match
                .annotate_outcome(format!("{} challenge_disabled", base_outcome).as_str());
            return Some(
                provider_registry
                    .maze_tarpit_provider()
                    .serve_maze_with_tracking(
                        req,
                        store,
                        cfg,
                        ip,
                        ua,
                        crate::maze::entry_path("botness-challenge-fallback").as_str(),
                        "botness_gate_challenge_disabled_fallback_maze",
                        event_outcome.as_str(),
                        Some(botness.score),
                    ),
            );
        }
        let policy_match = crate::runtime::policy_taxonomy::resolve_policy_match(
            crate::runtime::policy_taxonomy::PolicyTransition::ChallengeDisabledFallbackBlock(
                botness_signal_ids,
            ),
        );
        crate::observability::metrics::record_policy_match(store, &policy_match);
        crate::observability::metrics::increment(
            store,
            crate::observability::metrics::MetricName::BlocksTotal,
            None,
        );
        crate::admin::log_event(
            store,
            &crate::admin::EventLogEntry {
                ts: crate::admin::now_ts(),
                event: crate::admin::EventType::Block,
                ip: Some(ip.to_string()),
                reason: Some("botness_gate_challenge_disabled_fallback_block".to_string()),
                outcome: Some(
                    policy_match
                        .annotate_outcome(format!("{} challenge_disabled maze_disabled", base_outcome).as_str()),
                ),
                admin: None,
            },
        );
        return Some(Response::new(
            403,
            crate::enforcement::block_page::render_block_page(
                crate::enforcement::block_page::BlockReason::GeoPolicy,
            ),
        ));
    }

    None
}

pub(crate) fn maybe_handle_js(
    store: &Store,
    cfg: &crate::config::Config,
    ip: &str,
    user_agent: &str,
    needs_js: bool,
) -> Option<Response> {
    if !cfg.js_action_enabled() {
        return None;
    }
    if !needs_js {
        return None;
    }
    let policy_match = crate::runtime::policy_taxonomy::resolve_policy_match(
        crate::runtime::policy_taxonomy::PolicyTransition::JsVerificationRequired,
    );
    crate::observability::metrics::record_policy_match(store, &policy_match);
    crate::observability::metrics::increment(
        store,
        crate::observability::metrics::MetricName::ChallengesTotal,
        None,
    );
    crate::admin::log_event(
        store,
        &crate::admin::EventLogEntry {
            ts: crate::admin::now_ts(),
            event: crate::admin::EventType::Challenge,
            ip: Some(ip.to_string()),
            reason: Some("js_verification".to_string()),
            outcome: Some(policy_match.annotate_outcome("js challenge")),
            admin: None,
        },
    );
    Some(crate::signals::js_verification::inject_js_challenge(
        ip,
        user_agent,
        cfg.pow_enabled,
        cfg.pow_difficulty,
        cfg.pow_ttl_seconds,
        cfg.cdp_probe_family,
        cfg.cdp_probe_rollout_percent,
    ))
}
