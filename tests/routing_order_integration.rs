use once_cell::sync::Lazy;
use spin_sdk::http::{Method, Request};
use std::sync::{Mutex, MutexGuard};

static ENV_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

fn lock_env() -> MutexGuard<'static, ()> {
    ENV_MUTEX
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn request(method: Method, path: &str, headers: &[(&str, &str)]) -> Request {
    let mut builder = Request::builder();
    builder.method(method).uri(path);
    for (key, value) in headers {
        builder.header(*key, *value);
    }
    builder.body(Vec::new());
    builder.build()
}

fn with_runtime_env<T>(f: impl FnOnce() -> T) -> T {
    let _lock = lock_env();
    let vars = [
        ("SHUMA_API_KEY", "test-admin-key"),
        ("SHUMA_JS_SECRET", "test-js-secret"),
        ("SHUMA_FORWARDED_IP_SECRET", "test-forwarded-secret"),
        ("SHUMA_EVENT_LOG_RETENTION_HOURS", "168"),
        ("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "false"),
        ("SHUMA_KV_STORE_FAIL_OPEN", "true"),
        ("SHUMA_ENFORCE_HTTPS", "false"),
        ("SHUMA_DEBUG_HEADERS", "false"),
        ("SHUMA_POW_CONFIG_MUTABLE", "false"),
        ("SHUMA_CHALLENGE_CONFIG_MUTABLE", "false"),
        ("SHUMA_BOTNESS_CONFIG_MUTABLE", "false"),
    ];
    for (key, value) in vars {
        std::env::set_var(key, value);
    }
    std::env::remove_var("SHUMA_HEALTH_SECRET");
    std::env::remove_var("SHUMA_ADMIN_IP_ALLOWLIST");
    f()
}

#[test]
fn admin_options_is_rejected_before_main_pipeline() {
    with_runtime_env(|| {
        let req = request(
            Method::Options,
            "/admin/config",
            &[
                ("origin", "https://example.com"),
                ("access-control-request-method", "POST"),
            ],
        );
        let resp = shuma_gorath::handle_bot_defence_impl(&req);

        assert_eq!(*resp.status(), 403u16);
        assert_eq!(String::from_utf8_lossy(resp.body()), "Forbidden");
    });
}

#[test]
fn admin_route_requires_auth_even_with_fail_open_enabled() {
    with_runtime_env(|| {
        let req = request(Method::Get, "/admin/config", &[]);
        let resp = shuma_gorath::handle_bot_defence_impl(&req);

        // Regression guard: /admin should be handled by the early router/admin adapter,
        // not fall through to KV fail-open bypass behavior.
        assert_eq!(*resp.status(), 401u16);
        assert_eq!(
            String::from_utf8_lossy(resp.body()),
            "Unauthorized: Invalid or missing API key"
        );
    });
}

#[test]
fn health_route_precedes_kv_fail_open_bypass() {
    with_runtime_env(|| {
        let req = request(Method::Get, "/health", &[]);
        let resp = shuma_gorath::handle_bot_defence_impl(&req);

        // Regression guard: /health should evaluate local/trusted-IP access first.
        assert_eq!(*resp.status(), 403u16);
        assert_eq!(String::from_utf8_lossy(resp.body()), "Forbidden");
    });
}

#[test]
fn static_asset_path_bypasses_expensive_bot_checks() {
    with_runtime_env(|| {
        let req = request(Method::Get, "/assets/app.bundle.js", &[]);
        let resp = shuma_gorath::handle_bot_defence_impl(&req);

        assert_eq!(*resp.status(), 200u16);
        assert_eq!(
            String::from_utf8_lossy(resp.body()),
            "OK (passed bot defence)"
        );
    });
}
