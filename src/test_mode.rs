use spin_sdk::http::Response;

fn log_test_mode_event<S: crate::challenge::KeyValueStore>(
    store: &S,
    event: crate::admin::EventType,
    ip: &str,
    reason: &str,
    outcome: &str,
    record_test_mode_action: &impl Fn(),
) {
    record_test_mode_action();
    crate::admin::log_event(
        store,
        &crate::admin::EventLogEntry {
            ts: crate::admin::now_ts(),
            event,
            ip: Some(ip.to_string()),
            reason: Some(reason.to_string()),
            outcome: Some(outcome.to_string()),
            admin: None,
        },
    );
}

pub(crate) fn maybe_handle_test_mode<S, F, G>(
    store: &S,
    cfg: &crate::config::Config,
    site_id: &str,
    ip: &str,
    ua: &str,
    path: &str,
    geo_route: crate::geo::GeoPolicyRoute,
    needs_js_verification: F,
    record_test_mode_action: G,
) -> Option<Response>
where
    S: crate::challenge::KeyValueStore,
    F: Fn() -> bool,
    G: Fn(),
{
    if !cfg.test_mode {
        return None;
    }

    if path.starts_with("/pow") {
        return Some(Response::new(200, "TEST MODE: PoW bypassed"));
    }

    if crate::honeypot::is_honeypot(path, &cfg.honeypots) {
        crate::log_line(&format!("[TEST MODE] Would ban IP {ip} for honeypot"));
        log_test_mode_event(
            store,
            crate::admin::EventType::Block,
            ip,
            "honeypot [TEST MODE]",
            "would_block",
            &record_test_mode_action,
        );
        return Some(Response::new(200, "TEST MODE: Would block (honeypot)"));
    }

    if !crate::rate::check_rate_limit(store, site_id, ip, cfg.rate_limit) {
        crate::log_line(&format!("[TEST MODE] Would ban IP {ip} for rate limit"));
        log_test_mode_event(
            store,
            crate::admin::EventType::Block,
            ip,
            "rate_limit [TEST MODE]",
            "would_block",
            &record_test_mode_action,
        );
        return Some(Response::new(200, "TEST MODE: Would block (rate limit)"));
    }

    if crate::ban::is_banned(store, site_id, ip) {
        crate::log_line(&format!(
            "[TEST MODE] Would serve challenge to banned IP {ip}"
        ));
        log_test_mode_event(
            store,
            crate::admin::EventType::Block,
            ip,
            "banned [TEST MODE]",
            "would_serve_challenge",
            &record_test_mode_action,
        );
        return Some(Response::new(200, "TEST MODE: Would serve challenge"));
    }

    if cfg.js_required_enforced && path != "/health" && needs_js_verification() {
        crate::log_line(&format!(
            "[TEST MODE] Would inject JS challenge for IP {ip}"
        ));
        log_test_mode_event(
            store,
            crate::admin::EventType::Challenge,
            ip,
            "js_verification [TEST MODE]",
            "would_challenge",
            &record_test_mode_action,
        );
        return Some(Response::new(200, "TEST MODE: Would inject JS challenge"));
    }

    if crate::browser::is_outdated_browser(ua, &cfg.browser_block) {
        crate::log_line(&format!(
            "[TEST MODE] Would ban IP {ip} for outdated browser"
        ));
        log_test_mode_event(
            store,
            crate::admin::EventType::Block,
            ip,
            "browser [TEST MODE]",
            "would_block",
            &record_test_mode_action,
        );
        return Some(Response::new(
            200,
            "TEST MODE: Would block (outdated browser)",
        ));
    }

    match geo_route {
        crate::geo::GeoPolicyRoute::Block => {
            crate::log_line(&format!("[TEST MODE] Would block IP {ip} for GEO policy"));
            log_test_mode_event(
                store,
                crate::admin::EventType::Block,
                ip,
                "geo_policy_block [TEST MODE]",
                "would_block",
                &record_test_mode_action,
            );
            return Some(Response::new(200, "TEST MODE: Would block (geo policy)"));
        }
        crate::geo::GeoPolicyRoute::Maze => {
            crate::log_line(&format!(
                "[TEST MODE] Would route IP {ip} to maze for GEO policy"
            ));
            log_test_mode_event(
                store,
                crate::admin::EventType::Challenge,
                ip,
                "geo_policy_maze [TEST MODE]",
                "would_route_maze",
                &record_test_mode_action,
            );
            return Some(Response::new(
                200,
                "TEST MODE: Would route to maze (geo policy)",
            ));
        }
        crate::geo::GeoPolicyRoute::Challenge => {
            crate::log_line(&format!(
                "[TEST MODE] Would challenge IP {ip} for GEO policy"
            ));
            log_test_mode_event(
                store,
                crate::admin::EventType::Challenge,
                ip,
                "geo_policy_challenge [TEST MODE]",
                "would_challenge",
                &record_test_mode_action,
            );
            return Some(Response::new(
                200,
                "TEST MODE: Would serve challenge (geo policy)",
            ));
        }
        crate::geo::GeoPolicyRoute::Allow | crate::geo::GeoPolicyRoute::None => {}
    }

    Some(Response::new(
        200,
        "TEST MODE: Would allow (passed bot defence)",
    ))
}

#[cfg(test)]
mod tests;
