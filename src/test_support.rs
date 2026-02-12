use once_cell::sync::Lazy;
use spin_sdk::http::{Method, Request, Response};
use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};

#[derive(Default)]
pub(crate) struct InMemoryStore {
    map: Mutex<HashMap<String, Vec<u8>>>,
}

impl crate::challenge::KeyValueStore for InMemoryStore {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ()> {
        let map = self
            .map
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        Ok(map.get(key).cloned())
    }

    fn set(&self, key: &str, value: &[u8]) -> Result<(), ()> {
        let mut map = self
            .map
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        map.insert(key.to_string(), value.to_vec());
        Ok(())
    }

    fn delete(&self, key: &str) -> Result<(), ()> {
        let mut map = self
            .map
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        map.remove(key);
        Ok(())
    }
}

static ENV_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

pub(crate) fn lock_env() -> MutexGuard<'static, ()> {
    ENV_MUTEX
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub(crate) fn request_with_headers(path: &str, headers: &[(&str, &str)]) -> Request {
    request_with_method_and_headers(Method::Get, path, headers)
}

pub(crate) fn request_with_method_and_headers(
    method: Method,
    path: &str,
    headers: &[(&str, &str)],
) -> Request {
    let mut builder = Request::builder();
    builder.method(method).uri(path);
    for (key, value) in headers {
        builder.header(*key, *value);
    }
    builder.build()
}

pub(crate) fn has_header(resp: &Response, name: &str) -> bool {
    resp.headers()
        .any(|(key, _)| key.eq_ignore_ascii_case(name))
}
