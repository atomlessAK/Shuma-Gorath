// src/cdp.rs
// CDP (Chrome DevTools Protocol) Detection for WASM Bot Defence
// Detects automated browsers using Puppeteer, Playwright, Selenium, etc.
//
// This module provides JavaScript-based detection that identifies when a browser
// is being controlled via CDP (Chrome DevTools Protocol). This is the most reliable
// modern technique for detecting browser automation as it targets the fundamental
// mechanism all automation libraries use.
//
// References:
// - DataDome research by Antoine Vastel (June 2024)
// - https://rebrowser.net/blog/how-to-fix-runtime-enable-cdp-detection-of-puppeteer-playwright-and-other-automation-libraries
// - https://kaliiiiiiiiii.github.io/brotector/

use serde::{Deserialize, Serialize};
use spin_sdk::http::{Request, Response};
use spin_sdk::key_value::Store;

const MAX_CDP_CHECKS: usize = 32;

/// CDP detection report from client-side JavaScript
#[derive(Debug, Serialize, Deserialize)]
pub struct CdpReport {
    pub cdp_detected: bool,
    pub score: f32,
    pub checks: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CdpTier {
    Low,
    Medium,
    Strong,
}

fn has_check(checks: &[String], needle: &str) -> bool {
    checks
        .iter()
        .any(|check| check.eq_ignore_ascii_case(needle))
}

fn soft_signal_count(checks: &[String]) -> usize {
    ["cdp_timing", "chrome_obj", "plugins"]
        .iter()
        .filter(|name| has_check(checks, name))
        .count()
}

/// Stateless tiering for CDP reports.
///
/// Tiering intentionally treats hard automation checks as strongest evidence and
/// only uses thresholded score as supporting evidence when hard checks are absent.
pub fn classify_cdp_tier(report: &CdpReport, threshold: f32) -> CdpTier {
    let has_hard_signal =
        has_check(&report.checks, "webdriver") || has_check(&report.checks, "automation_props");
    if has_hard_signal {
        return CdpTier::Strong;
    }

    let soft_count = soft_signal_count(&report.checks);
    let threshold_clamped = threshold.clamp(0.0, 10.0);
    let strong_score_without_hard = report.score >= threshold_clamped + 0.4 && soft_count >= 2;
    if strong_score_without_hard {
        return CdpTier::Strong;
    }

    let medium_without_hard =
        report.score >= threshold_clamped || has_check(&report.checks, "cdp_timing");
    if medium_without_hard {
        return CdpTier::Medium;
    }

    CdpTier::Low
}

fn cdp_tier_label(tier: CdpTier) -> &'static str {
    match tier {
        CdpTier::Low => "low",
        CdpTier::Medium => "medium",
        CdpTier::Strong => "strong",
    }
}

fn increment_kv_counter(store: &Store, key: &str) {
    let current: u64 = store
        .get(key)
        .ok()
        .flatten()
        .and_then(|v| String::from_utf8(v).ok())
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);
    if let Err(e) = store.set(key, current.saturating_add(1).to_string().as_bytes()) {
        eprintln!("[cdp] failed to increment counter {}: {:?}", key, e);
    }
}

/// Handles incoming CDP detection reports from client-side JavaScript.
/// Auto-bans are only applied for strong-tier automation detections.
pub fn handle_cdp_report(store: &Store, req: &Request) -> Response {
    let ip = crate::extract_client_ip(req);
    let cfg = match crate::config::load_runtime_cached(store, "default") {
        Ok(cfg) => cfg,
        Err(_) => return Response::new(500, "Configuration unavailable"),
    };

    // Only process if CDP detection is enabled
    if !cfg.cdp_detection_enabled {
        return Response::new(200, "CDP detection disabled");
    }

    // Parse and validate the report body.
    let body = req.body();
    if let Err(e) = crate::input_validation::enforce_body_size(
        body,
        crate::input_validation::MAX_CDP_REPORT_BYTES,
    ) {
        return Response::new(400, e);
    }
    let mut report: CdpReport = match serde_json::from_slice(body) {
        Ok(r) => r,
        Err(_) => return Response::new(400, "Invalid CDP report format"),
    };
    if !report.score.is_finite() || report.score < 0.0 || report.score > 5.0 {
        return Response::new(400, "Invalid CDP score");
    }
    if report.checks.len() > MAX_CDP_CHECKS {
        return Response::new(400, "Too many CDP checks");
    }
    let mut sanitized_checks = Vec::new();
    for check in report.checks {
        let Some(clean) = crate::input_validation::sanitize_check_name(check.as_str()) else {
            return Response::new(400, "Invalid CDP check name");
        };
        if !sanitized_checks.contains(&clean) {
            sanitized_checks.push(clean);
        }
    }
    report.checks = sanitized_checks;

    let cdp_tier = classify_cdp_tier(&report, cfg.cdp_detection_threshold);
    let tier_label = cdp_tier_label(cdp_tier);

    // Log the CDP detection event
    crate::admin::log_event(
        store,
        &crate::admin::EventLogEntry {
            ts: crate::admin::now_ts(),
            event: crate::admin::EventType::Challenge,
            ip: Some(ip.clone()),
            reason: Some(format!(
                "cdp_detected:tier={} score={:.2}",
                tier_label, report.score
            )),
            outcome: Some(format!("checks:{}", report.checks.join(","))),
            admin: None,
        },
    );

    // Increment metrics
    crate::metrics::increment(store, crate::metrics::MetricName::CdpDetections, None);
    increment_kv_counter(store, "cdp:detections");

    // Auto-ban only for strong-tier detections when enabled.
    if cfg.cdp_auto_ban && cdp_tier == CdpTier::Strong {
        crate::ban::ban_ip_with_fingerprint(
            store,
            "default",
            &ip,
            "cdp_automation",
            cfg.get_ban_duration("cdp"),
            Some(crate::ban::BanFingerprint {
                score: Some((report.score * 10.0).round().clamp(0.0, 10.0) as u8),
                signals: vec!["cdp_automation".to_string()],
                summary: Some(format!(
                    "cdp_tier={} score={:.2} threshold={:.2} checks={}",
                    tier_label,
                    report.score,
                    cfg.cdp_detection_threshold,
                    report.checks.join(",")
                )),
            }),
        );
        crate::metrics::increment(
            store,
            crate::metrics::MetricName::BansTotal,
            Some("cdp_automation"),
        );
        increment_kv_counter(store, "cdp:auto_bans");

        crate::admin::log_event(
            store,
            &crate::admin::EventLogEntry {
                ts: crate::admin::now_ts(),
                event: crate::admin::EventType::Ban,
                ip: Some(ip.clone()),
                reason: Some("cdp_automation".to_string()),
                outcome: Some(format!(
                    "banned:tier={} score={:.2}",
                    tier_label, report.score
                )),
                admin: None,
            },
        );

        return Response::new(200, "Automation detected - banned");
    }

    Response::new(200, "Report received")
}

/// JavaScript code that detects CDP automation
/// This script checks for the Runtime.Enable CDP leak which affects all major
/// automation libraries (Puppeteer, Playwright, Selenium)
pub const CDP_DETECTION_JS: &str = r#"
(function() {
    'use strict';
    
    // CDP Detection - checks for Runtime.Enable leak
    // When CDP is active with Runtime.Enable, console methods trigger
    // Runtime.consoleAPICalled events with detectable side effects
    
    var cdpDetected = false;
    var detectionComplete = false;
    
    function detectCDP() {
        return new Promise(function(resolve) {
            // Method 1: Error stack trace analysis
            // CDP Runtime.Enable modifies how error stacks are generated
            try {
                var err = new Error();
                var stack = err.stack || '';
                
                // Check for CDP-specific patterns in stack traces
                if (stack.indexOf('puppeteer') !== -1 || 
                    stack.indexOf('playwright') !== -1 ||
                    stack.indexOf('__puppeteer_evaluation_script__') !== -1) {
                    resolve(true);
                    return;
                }
            } catch(e) {}
            
            // Method 2: Console timing analysis
            // When CDP is listening, console calls have slightly different timing
            var timings = [];
            var iterations = 5;
            
            function measureConsole(i) {
                if (i >= iterations) {
                    // Analyze timing variance
                    var sum = 0;
                    for (var j = 0; j < timings.length; j++) {
                        sum += timings[j];
                    }
                    var avg = sum / timings.length;
                    
                    // CDP typically shows higher variance in console timing
                    var variance = 0;
                    for (var k = 0; k < timings.length; k++) {
                        variance += Math.pow(timings[k] - avg, 2);
                    }
                    variance = variance / timings.length;
                    
                    // Threshold determined by testing - CDP usually shows variance > 0.5
                    // This is a soft signal, combined with other checks
                    var timingAnomaly = variance > 0.8;
                    resolve(timingAnomaly);
                    return;
                }
                
                var start = performance.now();
                console.debug(''); // Minimal console call
                var end = performance.now();
                timings.push(end - start);
                
                // Use setTimeout to avoid blocking
                setTimeout(function() { measureConsole(i + 1); }, 0);
            }
            
            measureConsole(0);
        });
    }
    
    // Method 3: WebDriver property check (classic but still useful)
    function checkWebDriver() {
        return navigator.webdriver === true;
    }
    
    // Method 4: Check for automation-related properties
    function checkAutomationProperties() {
        // Properties typically set by automation tools
        var suspicious = [
            'callPhantom',
            '_phantom',
            '__nightmare',
            '_selenium',
            'callSelenium',
            '_Selenium_IDE_Recorder',
            '__webdriver_evaluate',
            '__selenium_evaluate',
            '__webdriver_script_function',
            '__webdriver_script_func',
            '__webdriver_script_fn',
            '__fxdriver_evaluate',
            '__driver_unwrapped',
            '__webdriver_unwrapped',
            '__driver_evaluate',
            '__selenium_unwrapped',
            '__fxdriver_unwrapped',
            'domAutomation',
            'domAutomationController'
        ];
        
        for (var i = 0; i < suspicious.length; i++) {
            if (window[suspicious[i]] !== undefined) {
                return true;
            }
        }
        
        // Check document properties
        if (document.documentElement) {
            var attrs = document.documentElement.getAttributeNames();
            for (var j = 0; j < attrs.length; j++) {
                if (attrs[j].indexOf('webdriver') !== -1 ||
                    attrs[j].indexOf('selenium') !== -1 ||
                    attrs[j].indexOf('driver') !== -1) {
                    return true;
                }
            }
        }
        
        return false;
    }
    
    // Method 5: Chrome object consistency check
    function checkChromeObject() {
        // In headless Chrome, window.chrome exists but may have inconsistencies
        if (window.chrome) {
            // Check for missing runtime (suspicious in real Chrome)
            if (!window.chrome.runtime) {
                // Could be legitimate old Chrome, weight accordingly
                return 0.3;
            }
            // Check for csi and loadTimes (missing in automation)
            if (!window.chrome.csi || !window.chrome.loadTimes) {
                return 0.2;
            }
        } else if (/Chrome/.test(navigator.userAgent)) {
            // Claims to be Chrome but no chrome object
            return 0.8;
        }
        return 0;
    }
    
    // Method 6: Plugin/MimeType array check
    function checkPlugins() {
        // Headless browsers typically have no plugins
        if (navigator.plugins && navigator.plugins.length === 0) {
            // Weight based on claimed browser
            if (/Chrome|Firefox/.test(navigator.userAgent)) {
                return 0.4; // Suspicious but not definitive
            }
        }
        
        // Check if plugins is a real PluginArray
        try {
            if (navigator.plugins && 
                Object.prototype.toString.call(navigator.plugins) !== '[object PluginArray]') {
                return 0.6;
            }
        } catch(e) {
            return 0.3;
        }
        
        return 0;
    }
    
    // Combined detection with weighted scoring
    window._checkCDPAutomation = function() {
        return new Promise(function(resolve) {
            if (detectionComplete) {
                resolve({ detected: cdpDetected, score: window._cdpScore || 0 });
                return;
            }
            
            var score = 0;
            var checks = [];
            
            // Immediate checks
            if (checkWebDriver()) {
                score += 1.0; // Definitive
                checks.push('webdriver');
            }
            
            if (checkAutomationProperties()) {
                score += 0.9;
                checks.push('automation_props');
            }
            
            score += checkChromeObject();
            if (checkChromeObject() > 0) checks.push('chrome_obj');
            
            score += checkPlugins();
            if (checkPlugins() > 0) checks.push('plugins');
            
            // Async CDP detection
            detectCDP().then(function(cdpResult) {
                if (cdpResult) {
                    score += 0.7;
                    checks.push('cdp_timing');
                }
                
                cdpDetected = score >= 0.8;
                window._cdpScore = score;
                window._cdpChecks = checks;
                detectionComplete = true;
                
                resolve({
                    detected: cdpDetected,
                    score: score,
                    checks: checks
                });
            });
        });
    };
    
    // Auto-run detection on load
    if (document.readyState === 'complete') {
        window._checkCDPAutomation();
    } else {
        window.addEventListener('load', function() {
            window._checkCDPAutomation();
        });
    }
})();
"#;

/// Returns the CDP detection JavaScript as a string
pub fn get_cdp_detection_script() -> &'static str {
    CDP_DETECTION_JS
}

/// JavaScript snippet to report CDP detection result back to the server.
/// Used by inject_cdp_detection() and available for custom injection scenarios.
#[allow(dead_code)]
pub fn get_cdp_report_script(report_endpoint: &str) -> String {
    format!(
        r#"
<script>
(function() {{
    window._checkCDPAutomation().then(function(result) {{
        if (result.detected) {{
            // Report automation detection to server
            fetch('{}', {{
                method: 'POST',
                headers: {{ 'Content-Type': 'application/json' }},
                body: JSON.stringify({{
                    cdp_detected: true,
                    score: result.score,
                    checks: result.checks
                }})
            }});
        }}
    }});
}})();
</script>
"#,
        report_endpoint
    )
}

/// Injects CDP detection into an HTML page.
/// Useful for injecting detection into external HTML content (e.g., proxy scenarios).
/// Currently used by tests; main integration uses get_cdp_detection_script() directly.
#[allow(dead_code)]
pub fn inject_cdp_detection(html: &str, report_endpoint: Option<&str>) -> String {
    let detection_script = format!("<script>{}</script>", CDP_DETECTION_JS);

    let report_script = if let Some(endpoint) = report_endpoint {
        get_cdp_report_script(endpoint)
    } else {
        String::new()
    };

    // Inject before </head> if present, otherwise before </body>
    if html.contains("</head>") {
        html.replace(
            "</head>",
            &format!("{}{}</head>", detection_script, report_script),
        )
    } else if html.contains("</body>") {
        html.replace(
            "</body>",
            &format!("{}{}</body>", detection_script, report_script),
        )
    } else {
        format!("{}{}{}", html, detection_script, report_script)
    }
}

#[cfg(test)]
mod tests;
