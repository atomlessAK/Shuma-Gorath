use super::*;
use crate::challenge::KeyValueStore;

#[test]
fn test_ban_and_expiry() {
    let store = crate::test_support::InMemoryStore::default();
    let site_id = "testsite";
    let ip = "1.2.3.4";
    ban_ip(&store, site_id, ip, "test", 1);
    assert!(is_banned(&store, site_id, ip));
}

#[test]
fn test_ban_and_unban_unknown_ip() {
    let store = crate::test_support::InMemoryStore::default();
    let site_id = "testsite";
    let ip = "unknown";
    ban_ip(&store, site_id, ip, "test", 60);
    assert!(is_banned(&store, site_id, ip));
    unban_ip(&store, site_id, ip);
    assert!(!is_banned(&store, site_id, ip));
}

#[test]
fn test_unban_ip_function() {
    let store = crate::test_support::InMemoryStore::default();
    let site_id = "testsite";
    let ip = "192.168.1.100";

    ban_ip(&store, site_id, ip, "test_reason", 3600);
    assert!(
        is_banned(&store, site_id, ip),
        "IP should be banned after ban_ip"
    );

    unban_ip(&store, site_id, ip);
    assert!(
        !is_banned(&store, site_id, ip),
        "IP should not be banned after unban_ip"
    );
}

#[test]
fn test_unban_ip_nonexistent() {
    let store = crate::test_support::InMemoryStore::default();
    let site_id = "testsite";
    let ip = "10.0.0.1";

    unban_ip(&store, site_id, ip);
    assert!(
        !is_banned(&store, site_id, ip),
        "Non-existent IP should not be banned"
    );
}

#[test]
fn test_ban_entry_serialization() {
    let entry = BanEntry {
        reason: "test".to_string(),
        expires: 1234567890,
        banned_at: 1234560000,
        fingerprint: Some(BanFingerprint {
            score: Some(6),
            signals: vec!["rate_limit_exceeded".to_string()],
            summary: Some("rate_limit=80".to_string()),
        }),
    };
    let json = serde_json::to_string(&entry).unwrap();
    let de: BanEntry = serde_json::from_str(&json).unwrap();
    assert_eq!(de.reason, "test");
    assert_eq!(de.expires, 1234567890);
    assert_eq!(de.banned_at, 1234560000);
    assert!(de.fingerprint.is_some());
}

#[test]
fn test_legacy_ban_entry_deserializes_with_defaults() {
    let legacy = r#"{"reason":"legacy","expires":42}"#;
    let de: BanEntry = serde_json::from_str(legacy).unwrap();
    assert_eq!(de.reason, "legacy");
    assert_eq!(de.expires, 42);
    assert!(de.fingerprint.is_none());
    assert!(de.banned_at > 0);
}

#[test]
fn test_ban_metadata_is_sanitized_before_persist() {
    let store = crate::test_support::InMemoryStore::default();
    let site_id = "testsite";
    let ip = "198.51.100.10";
    let raw_reason = format!("{}\n{}", "r".repeat(200), "tail");
    let raw_summary = format!("ua=\t{}\r\n", "a".repeat(700));

    ban_ip_with_fingerprint(
        &store,
        site_id,
        ip,
        &raw_reason,
        60,
        Some(BanFingerprint {
            score: Some(5),
            signals: vec!["outdated_browser".to_string()],
            summary: Some(raw_summary),
        }),
    );

    let key = format!("ban:{}:{}", site_id, ip);
    let raw = store.get(&key).unwrap().unwrap();
    let entry: BanEntry = serde_json::from_slice(&raw).unwrap();

    assert!(!entry.reason.chars().any(|c| c.is_control()));
    assert_eq!(
        entry.reason.chars().count(),
        crate::input_validation::MAX_BAN_REASON_LEN
    );

    let summary = entry
        .fingerprint
        .and_then(|f| f.summary)
        .expect("summary should be present");
    assert!(!summary.chars().any(|c| c.is_control()));
    assert_eq!(
        summary.chars().count(),
        crate::input_validation::MAX_BAN_SUMMARY_LEN
    );
}
