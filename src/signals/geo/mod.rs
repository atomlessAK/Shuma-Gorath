// src/geo.rs
// Geo-based risk logic for WASM Bot Defence
// Checks for high-risk geographies using edge-provided headers (e.g., X-Geo-Country)

use spin_sdk::http::Request;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeoPolicyRoute {
    None,
    Allow,
    Challenge,
    Maze,
    Block,
}

/// Extract and normalize country code from trusted edge headers.
/// Returns None when geo headers are not trusted for this request.
pub fn extract_geo_country(req: &Request, headers_trusted: bool) -> Option<String> {
    if !headers_trusted {
        return None;
    }
    req.header("x-geo-country")
        .and_then(|header| header.as_str())
        .map(str::trim)
        .and_then(normalize_country_code)
}

/// Normalize a country code to two-letter uppercase ISO form.
pub fn normalize_country_code(value: &str) -> Option<String> {
    crate::input_validation::normalize_country_code_iso(value)
}

/// Normalize, deduplicate, and preserve order for configured country lists.
pub fn normalize_country_list(values: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut normalized = Vec::new();
    for value in values {
        if let Some(code) = normalize_country_code(value) {
            if seen.insert(code.clone()) {
                normalized.push(code);
            }
        }
    }
    normalized
}

/// Returns true when the given country appears in the provided country list.
pub fn country_in_list(country: &str, list: &[String]) -> bool {
    list.iter().any(|c| c.eq_ignore_ascii_case(country))
}

/// Evaluate configured GEO policy routing for a normalized country code.
/// Precedence is most restrictive first: Block > Maze > Challenge > Allow.
pub fn evaluate_geo_policy(country: Option<&str>, cfg: &crate::config::Config) -> GeoPolicyRoute {
    let Some(country) = country else {
        return GeoPolicyRoute::None;
    };
    let Some(normalized) = normalize_country_code(country) else {
        return GeoPolicyRoute::None;
    };

    if country_in_list(normalized.as_str(), &cfg.geo_block) {
        return GeoPolicyRoute::Block;
    }
    if country_in_list(normalized.as_str(), &cfg.geo_maze) {
        return GeoPolicyRoute::Maze;
    }
    if country_in_list(normalized.as_str(), &cfg.geo_challenge) {
        return GeoPolicyRoute::Challenge;
    }
    if country_in_list(normalized.as_str(), &cfg.geo_allow) {
        return GeoPolicyRoute::Allow;
    }
    GeoPolicyRoute::None
}

pub fn bot_signal(
    signal_available: bool,
    scored_risk: bool,
    weight: u8,
) -> crate::signals::botness::BotSignal {
    if !signal_available {
        return crate::signals::botness::BotSignal::unavailable(
            "geo_risk",
            "High-risk geography",
        );
    }
    crate::signals::botness::BotSignal::scored(
        "geo_risk",
        "High-risk geography",
        scored_risk,
        weight,
    )
}

#[cfg(test)]
mod tests;
