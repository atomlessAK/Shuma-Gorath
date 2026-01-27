// src/geo.rs
// Geo-based risk logic for WASM Bot Trap
// Checks for high-risk geographies using edge-provided headers (e.g., X-Geo-Country)

use spin_sdk::http::Request;

/// Returns true if the request is from a high-risk country (case-insensitive match).
/// Uses the geo_risk config list and the X-Geo-Country header set by the edge.
pub fn is_high_risk_geo(req: &Request, geo_risk: &[String]) -> bool {
    if let Some(header) = req.header("x-geo-country") {
        let country = header.as_str().unwrap_or("");
        geo_risk.iter().any(|c| c.eq_ignore_ascii_case(country))
    } else {
        false
    }
}
