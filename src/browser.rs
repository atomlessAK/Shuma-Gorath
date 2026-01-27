// src/browser.rs
// Outdated browser detection for WASM Bot Trap

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

fn extract_version(ua: &str, name: &str) -> Option<u32> {
    let marker = format!("{}/", name);
    if let Some(idx) = ua.find(&marker) {
        let ver_str = &ua[idx + marker.len()..];
        let ver = ver_str.split(|c: char| !c.is_digit(10)).next()?;
        return ver.parse().ok();
    }
    None
}
