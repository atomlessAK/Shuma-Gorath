use spin_sdk::http::Response;
use spin_sdk::key_value::Store;

pub(crate) fn open_store_or_fail_mode_response() -> Result<Store, Response> {
    match Store::open_default() {
        Ok(store) => Ok(store),
        Err(_e) => {
            let fail_open = crate::shuma_fail_open();
            let mode = crate::fail_mode_label(fail_open);
            crate::log_line(&format!(
                "[KV OUTAGE] Store unavailable during request handling; SHUMA_KV_STORE_FAIL_OPEN={}",
                fail_open
            ));

            if !fail_open {
                return Err(crate::response_with_optional_debug_headers(
                    500,
                    "Key-value store error (fail-closed)",
                    "unavailable",
                    mode,
                ));
            }

            Err(crate::response_with_optional_debug_headers(
                200,
                "OK (bot defence: store unavailable, all checks bypassed)",
                "unavailable",
                mode,
            ))
        }
    }
}
