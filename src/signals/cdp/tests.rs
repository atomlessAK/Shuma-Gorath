use super::*;

#[test]
fn test_get_cdp_detection_script_returns_javascript() {
    let script = get_cdp_detection_script();

    assert!(
        !script.is_empty(),
        "CDP detection script should not be empty"
    );
    assert!(
        script.contains("_checkCDPAutomation"),
        "Script should define _checkCDPAutomation function"
    );
    assert!(
        script.contains("detectCDP"),
        "Script should contain detectCDP function"
    );
    assert!(
        script.contains("checkWebDriver"),
        "Script should contain webdriver check"
    );
    assert!(
        script.contains("checkAutomationProperties"),
        "Script should contain automation properties check"
    );
}

#[test]
fn test_get_cdp_report_script_with_endpoint() {
    let endpoint = "/cdp-report";
    let script = get_cdp_report_script(endpoint);

    assert!(
        script.contains(endpoint),
        "Script should contain the report endpoint"
    );
    assert!(script.contains("fetch"), "Script should contain fetch call");
    assert!(script.contains("POST"), "Script should use POST method");
    assert!(
        script.contains("cdp_detected"),
        "Script should send cdp_detected field"
    );
}

#[test]
fn test_inject_cdp_detection_into_head() {
    let html = r#"<!DOCTYPE html>
<html>
<head>
    <title>Test</title>
</head>
<body>
    <p>Content</p>
</body>
</html>"#;

    let result = inject_cdp_detection(html, None);

    assert!(
        result.contains("_checkCDPAutomation"),
        "Injection should add detection script"
    );
    assert!(result.contains("</head>"), "Should preserve </head> tag");

    let head_end_pos = result.find("</head>").unwrap();
    let script_pos = result.find("_checkCDPAutomation").unwrap();
    assert!(
        script_pos < head_end_pos,
        "Script should be injected before </head>"
    );
}

#[test]
fn test_inject_cdp_detection_into_body_fallback() {
    let html = r#"<!DOCTYPE html>
<html>
<body>
    <p>Content</p>
</body>
</html>"#;

    let result = inject_cdp_detection(html, None);

    assert!(
        result.contains("_checkCDPAutomation"),
        "Injection should add detection script"
    );
    assert!(result.contains("</body>"), "Should preserve </body> tag");
}

#[test]
fn test_inject_cdp_detection_with_report_endpoint() {
    let html = r#"<html><head></head><body></body></html>"#;
    let endpoint = "/api/cdp-report";

    let result = inject_cdp_detection(html, Some(endpoint));

    assert!(
        result.contains("_checkCDPAutomation"),
        "Should include detection script"
    );
    assert!(result.contains(endpoint), "Should include report endpoint");
    assert!(
        result.contains("fetch"),
        "Should include fetch for reporting"
    );
}

#[test]
fn test_inject_cdp_detection_minimal_html() {
    let html = "<p>Just some text</p>";

    let result = inject_cdp_detection(html, None);

    assert!(
        result.contains("_checkCDPAutomation"),
        "Should inject script even for minimal HTML"
    );
    assert!(
        result.contains("<p>Just some text</p>"),
        "Original content should be preserved"
    );
}

#[test]
fn test_cdp_report_deserialization() {
    let json = r#"{"cdp_detected":true,"score":0.95,"checks":["webdriver","automation_props"]}"#;
    let report: CdpReport = serde_json::from_str(json).expect("Should deserialize CdpReport");

    assert!(report.cdp_detected, "cdp_detected should be true");
    assert!((report.score - 0.95).abs() < 0.01, "score should be 0.95");
    assert_eq!(report.checks.len(), 2, "should have 2 checks");
    assert!(report.checks.contains(&"webdriver".to_string()));
    assert!(report.checks.contains(&"automation_props".to_string()));
}

#[test]
fn test_cdp_report_serialization() {
    let report = CdpReport {
        cdp_detected: true,
        score: 0.85,
        checks: vec!["cdp_timing".to_string()],
    };

    let json = serde_json::to_string(&report).expect("Should serialize CdpReport");
    assert!(json.contains("\"cdp_detected\":true"));
    assert!(json.contains("\"score\":0.85") || json.contains("\"score\": 0.85"));
    assert!(json.contains("cdp_timing"));
}

#[test]
fn test_cdp_tier_strong_when_hard_signal_present() {
    let report = CdpReport {
        cdp_detected: true,
        score: 0.2,
        checks: vec!["webdriver".to_string()],
    };
    assert_eq!(classify_cdp_tier(&report, 0.8), CdpTier::Strong);
}

#[test]
fn test_cdp_tier_strong_when_soft_signals_are_high_confidence() {
    let report = CdpReport {
        cdp_detected: true,
        score: 1.3,
        checks: vec![
            "cdp_timing".to_string(),
            "plugins".to_string(),
            "chrome_obj".to_string(),
        ],
    };
    assert_eq!(classify_cdp_tier(&report, 0.8), CdpTier::Strong);
}

#[test]
fn test_cdp_tier_medium_when_threshold_met_without_hard_signal() {
    let report = CdpReport {
        cdp_detected: true,
        score: 0.9,
        checks: vec!["plugins".to_string()],
    };
    assert_eq!(classify_cdp_tier(&report, 0.8), CdpTier::Medium);
}

#[test]
fn test_cdp_tier_low_when_only_weak_signal_present() {
    let report = CdpReport {
        cdp_detected: false,
        score: 0.2,
        checks: vec!["plugins".to_string()],
    };
    assert_eq!(classify_cdp_tier(&report, 0.8), CdpTier::Low);
}
