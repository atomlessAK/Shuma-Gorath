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
