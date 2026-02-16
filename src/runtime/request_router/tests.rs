use super::*;
use spin_sdk::http::{Method, Request};

fn request(method: Method, path: &str) -> Request {
    let mut builder = Request::builder();
    builder.method(method).uri(path);
    builder.build()
}

#[test]
fn early_router_short_circuits_health_path() {
    let req = request(Method::Get, "/health");
    let resp = maybe_handle_early_route(&req, "/health");
    assert!(resp.is_some());
    assert_eq!(*resp.unwrap().status(), 403u16);
}

#[test]
fn early_router_short_circuits_admin_options() {
    let req = request(Method::Options, "/admin/config");
    let resp = maybe_handle_early_route(&req, "/admin/config");
    assert!(resp.is_some());
    assert_eq!(*resp.unwrap().status(), 403u16);
}

#[test]
fn early_router_does_not_consume_cdp_report_path() {
    let req = request(Method::Post, "/cdp-report");
    let resp = maybe_handle_early_route(&req, "/cdp-report");
    assert!(resp.is_none());
}

#[test]
fn early_router_does_not_consume_unrelated_paths() {
    let req = request(Method::Get, "/totally-unrelated");
    let resp = maybe_handle_early_route(&req, "/totally-unrelated");
    assert!(resp.is_none());
}

#[test]
fn early_router_short_circuits_maze_asset_paths() {
    let path = crate::maze::assets::maze_script_path();
    let req = request(Method::Get, path);
    let resp = maybe_handle_early_route(&req, path);
    assert!(resp.is_some());
    assert_eq!(*resp.unwrap().status(), 200u16);
}

#[test]
fn early_router_redirects_dashboard_root_to_index_html() {
    let req = request(Method::Get, "/dashboard");
    let resp = maybe_handle_early_route(&req, "/dashboard");
    assert!(resp.is_some());
    let resp = resp.unwrap();
    assert_eq!(*resp.status(), 308u16);
    let location = resp
        .headers()
        .find(|(name, _)| name.eq_ignore_ascii_case("location"))
        .and_then(|(_, value)| value.as_str())
        .unwrap_or("");
    assert_eq!(location, "/dashboard/index.html");
}
