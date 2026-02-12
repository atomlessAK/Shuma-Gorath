// src/browser.rs
// Outdated browser detection for WASM Bot Defence

pub fn is_outdated_browser(user_agent: &str, block_list: &[(String, u32)]) -> bool {
    for (name, min_version) in block_list {
        if let Some(ver) = extract_version(user_agent, name) {
            if ver < *min_version {
                return true;
            }
        }
    }
    false
}

pub fn extract_version(ua: &str, name: &str) -> Option<u32> {
    // Safari versioning is exposed in Version/x.y while Safari/x tracks WebKit build.
    if name.eq_ignore_ascii_case("safari") {
        if let Some(ver) = extract_version_after_marker(ua, "Version/") {
            return Some(ver);
        }
    }

    let marker = format!("{}/", name);
    extract_version_after_marker(ua, &marker)
}

fn extract_version_after_marker(ua: &str, marker: &str) -> Option<u32> {
    let idx = ua.find(marker)?;
    let ver_str = &ua[idx + marker.len()..];
    let token = ver_str.split_whitespace().next().unwrap_or(ver_str);
    let start = token.find(|c: char| c.is_ascii_digit())?;
    let digits = token[start..].split(|c: char| !c.is_ascii_digit()).next()?;
    if digits.is_empty() {
        return None;
    }
    digits.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::{extract_version, is_outdated_browser};

    #[test]
    fn extract_version_reads_chrome_major() {
        let ua = "Mozilla/5.0 Chrome/120.0.1 Safari/537.36";
        assert_eq!(extract_version(ua, "Chrome"), Some(120));
    }

    #[test]
    fn extract_version_handles_non_digit_prefix_after_marker() {
        let ua = "Mozilla/5.0 Chrome/v120.0 Safari/537.36";
        assert_eq!(extract_version(ua, "Chrome"), Some(120));
    }

    #[test]
    fn extract_version_for_safari_prefers_version_token() {
        let ua = "Mozilla/5.0 Version/17.3 Safari/605.1.15";
        assert_eq!(extract_version(ua, "Safari"), Some(17));
    }

    #[test]
    fn extract_version_for_safari_falls_back_when_version_missing() {
        let ua = "Mozilla/5.0 Safari/605.1.15";
        assert_eq!(extract_version(ua, "Safari"), Some(605));
    }

    #[test]
    fn extract_version_returns_none_when_no_digits_found() {
        let ua = "Mozilla/5.0 Chrome/abc Safari/537.36";
        assert_eq!(extract_version(ua, "Chrome"), None);
    }

    #[test]
    fn safari_outdated_detection_uses_version_token() {
        let ua = "Mozilla/5.0 Version/14.1 Safari/605.1.15";
        let block_list = vec![("Safari".to_string(), 15)];
        assert!(is_outdated_browser(ua, &block_list));
    }
}
