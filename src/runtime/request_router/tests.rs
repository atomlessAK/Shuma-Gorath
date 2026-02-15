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
    let req = request(Method::Get, crate::maze::assets::MAZE_SCRIPT_PATH);
    let resp = maybe_handle_early_route(&req, crate::maze::assets::MAZE_SCRIPT_PATH);
    assert!(resp.is_some());
    assert_eq!(*resp.unwrap().status(), 200u16);
}
