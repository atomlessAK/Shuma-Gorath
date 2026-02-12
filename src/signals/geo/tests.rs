use super::*;
use spin_sdk::http::{Method, Request};

fn build_request(headers: &[(&str, &str)]) -> Request {
    let mut builder = Request::builder();
    builder.method(Method::Get).uri("/health");
    for (name, value) in headers {
        builder.header(*name, *value);
    }
    builder.build()
}

#[test]
fn extract_geo_country_ignores_untrusted_headers() {
    let req = build_request(&[("x-geo-country", "US")]);
    let country = extract_geo_country(&req, false);
    assert_eq!(country, None);
}

#[test]
fn extract_geo_country_trims_and_normalizes() {
    let req = build_request(&[("x-geo-country", " us ")]);
    let country = extract_geo_country(&req, true);
    assert_eq!(country.as_deref(), Some("US"));
}

#[test]
fn extract_geo_country_rejects_non_iso_code() {
    let req = build_request(&[("x-geo-country", "zz")]);
    let country = extract_geo_country(&req, true);
    assert_eq!(country, None);
}

#[test]
fn geo_policy_uses_most_restrictive_match_precedence() {
    let mut cfg = crate::config::defaults().clone();
    cfg.geo_allow = vec!["US".to_string()];
    cfg.geo_challenge = vec!["US".to_string()];
    cfg.geo_maze = vec!["US".to_string()];
    cfg.geo_block = vec!["US".to_string()];

    let route = evaluate_geo_policy(Some("US"), &cfg);
    assert_eq!(route, GeoPolicyRoute::Block);
}

#[test]
fn geo_policy_returns_none_when_no_match() {
    let cfg = crate::config::defaults().clone();
    let route = evaluate_geo_policy(Some("US"), &cfg);
    assert_eq!(route, GeoPolicyRoute::None);
}
