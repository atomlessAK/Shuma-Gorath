use super::*;

#[test]
fn maybe_handle_test_mode_returns_none_when_disabled() {
    let store = crate::test_support::InMemoryStore::default();
    let mut cfg = crate::config::defaults().clone();
    cfg.test_mode = false;

    let resp = maybe_handle_test_mode(
        &store,
        &cfg,
        "default",
        "1.2.3.4",
        "Mozilla/5.0",
        "/",
        &crate::signals::ip_range_policy::Evaluation::NoMatch,
        crate::signals::geo::GeoPolicyRoute::None,
        || false,
        || {},
    );

    assert!(resp.is_none());
}

#[test]
fn maybe_handle_test_mode_pow_path_bypasses() {
    let store = crate::test_support::InMemoryStore::default();
    let mut cfg = crate::config::defaults().clone();
    cfg.test_mode = true;

    let resp = maybe_handle_test_mode(
        &store,
        &cfg,
        "default",
        "1.2.3.4",
        "Mozilla/5.0",
        "/pow",
        &crate::signals::ip_range_policy::Evaluation::NoMatch,
        crate::signals::geo::GeoPolicyRoute::None,
        || false,
        || {},
    )
    .unwrap();

    assert_eq!(*resp.status(), 200u16);
    assert_eq!(
        String::from_utf8(resp.into_body()).unwrap(),
        "TEST MODE: PoW bypassed"
    );
}

#[test]
fn maybe_handle_test_mode_honeypot_blocks_without_calling_js_check() {
    let store = crate::test_support::InMemoryStore::default();
    let mut cfg = crate::config::defaults().clone();
    cfg.test_mode = true;
    cfg.honeypots = vec!["/trap-me".to_string()];

    let resp = maybe_handle_test_mode(
        &store,
        &cfg,
        "default",
        "1.2.3.4",
        "Mozilla/5.0",
        "/trap-me",
        &crate::signals::ip_range_policy::Evaluation::NoMatch,
        crate::signals::geo::GeoPolicyRoute::None,
        || panic!("js check should not run for honeypot branch"),
        || {},
    )
    .unwrap();

    assert_eq!(*resp.status(), 200u16);
    assert_eq!(
        String::from_utf8(resp.into_body()).unwrap(),
        "TEST MODE: Would block (honeypot)"
    );
}

#[test]
fn maybe_handle_test_mode_honeypot_disabled_does_not_block() {
    let store = crate::test_support::InMemoryStore::default();
    let mut cfg = crate::config::defaults().clone();
    cfg.test_mode = true;
    cfg.honeypot_enabled = false;
    cfg.honeypots = vec!["/trap-me".to_string()];

    let resp = maybe_handle_test_mode(
        &store,
        &cfg,
        "default",
        "1.2.3.4",
        "Mozilla/5.0",
        "/trap-me",
        &crate::signals::ip_range_policy::Evaluation::NoMatch,
        crate::signals::geo::GeoPolicyRoute::Allow,
        || false,
        || {},
    )
    .unwrap();

    assert_eq!(*resp.status(), 200u16);
    assert_eq!(
        String::from_utf8(resp.into_body()).unwrap(),
        "TEST MODE: Would allow (passed bot defence)"
    );
}

#[test]
fn maybe_handle_test_mode_allows_when_no_checks_trigger() {
    let store = crate::test_support::InMemoryStore::default();
    let mut cfg = crate::config::defaults().clone();
    cfg.test_mode = true;
    cfg.rate_limit = 10;
    cfg.js_required_enforced = false;
    cfg.honeypots = vec!["/trap-only".to_string()];

    let resp = maybe_handle_test_mode(
        &store,
        &cfg,
        "default",
        "1.2.3.4",
        "Mozilla/5.0",
        "/home",
        &crate::signals::ip_range_policy::Evaluation::NoMatch,
        crate::signals::geo::GeoPolicyRoute::Allow,
        || false,
        || {},
    )
    .unwrap();

    assert_eq!(*resp.status(), 200u16);
    assert_eq!(
        String::from_utf8(resp.into_body()).unwrap(),
        "TEST MODE: Would allow (passed bot defence)"
    );
}

#[test]
fn maybe_handle_test_mode_geo_challenge_falls_back_to_maze_when_challenge_disabled() {
    let store = crate::test_support::InMemoryStore::default();
    let mut cfg = crate::config::defaults().clone();
    cfg.test_mode = true;
    cfg.challenge_puzzle_enabled = false;
    cfg.maze_enabled = true;

    let resp = maybe_handle_test_mode(
        &store,
        &cfg,
        "default",
        "1.2.3.4",
        "Mozilla/5.0",
        "/",
        &crate::signals::ip_range_policy::Evaluation::NoMatch,
        crate::signals::geo::GeoPolicyRoute::Challenge,
        || false,
        || {},
    )
    .unwrap();

    assert_eq!(*resp.status(), 200u16);
    assert_eq!(
        String::from_utf8(resp.into_body()).unwrap(),
        "TEST MODE: Would route to maze (geo challenge fallback)"
    );
}

#[test]
fn maybe_handle_test_mode_geo_challenge_falls_back_to_block_when_disabled() {
    let store = crate::test_support::InMemoryStore::default();
    let mut cfg = crate::config::defaults().clone();
    cfg.test_mode = true;
    cfg.challenge_puzzle_enabled = false;
    cfg.maze_enabled = false;

    let resp = maybe_handle_test_mode(
        &store,
        &cfg,
        "default",
        "1.2.3.4",
        "Mozilla/5.0",
        "/",
        &crate::signals::ip_range_policy::Evaluation::NoMatch,
        crate::signals::geo::GeoPolicyRoute::Challenge,
        || false,
        || {},
    )
    .unwrap();

    assert_eq!(*resp.status(), 200u16);
    assert_eq!(
        String::from_utf8(resp.into_body()).unwrap(),
        "TEST MODE: Would block (geo challenge fallback, challenge disabled)"
    );
}

#[test]
fn maybe_handle_test_mode_reports_ip_range_actions() {
    let store = crate::test_support::InMemoryStore::default();
    let mut cfg = crate::config::defaults().clone();
    cfg.test_mode = true;

    let ip_range = crate::signals::ip_range_policy::Evaluation::Matched(
        crate::signals::ip_range_policy::MatchDetails {
            source: crate::signals::ip_range_policy::MatchSource::CustomRule,
            source_id: "deny_dc".to_string(),
            action: crate::config::IpRangePolicyAction::Forbidden403,
            matched_cidr: "203.0.113.0/24".to_string(),
            redirect_url: None,
            custom_message: None,
        },
    );

    let resp = maybe_handle_test_mode(
        &store,
        &cfg,
        "default",
        "203.0.113.4",
        "Mozilla/5.0",
        "/",
        &ip_range,
        crate::signals::geo::GeoPolicyRoute::Allow,
        || false,
        || {},
    )
    .unwrap();

    assert_eq!(*resp.status(), 200u16);
    assert_eq!(
        String::from_utf8(resp.into_body()).unwrap(),
        "TEST MODE: Would apply IP range action (forbidden_403)"
    );
}
