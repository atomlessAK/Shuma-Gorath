#[cfg(test)]
mod tests {
    use crate::maze::runtime::{self, MazeFallbackReason, MazeServeDecision};
    use crate::maze::state::MazeStateStore;
    use spin_sdk::http::{Method, Request};
    use std::collections::HashMap;
    use std::sync::Mutex;
    use std::thread;
    use std::time::Duration;

    #[derive(Default)]
    struct MemStore {
        data: Mutex<HashMap<String, Vec<u8>>>,
    }

    impl MazeStateStore for MemStore {
        fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ()> {
            Ok(self.data.lock().unwrap().get(key).cloned())
        }

        fn set(&self, key: &str, value: &[u8]) -> Result<(), ()> {
            self.data
                .lock()
                .unwrap()
                .insert(key.to_string(), value.to_vec());
            Ok(())
        }

        fn delete(&self, key: &str) -> Result<(), ()> {
            self.data.lock().unwrap().remove(key);
            Ok(())
        }
    }

    struct CrawlerHarness {
        store: MemStore,
        cfg: crate::config::Config,
        ip: String,
        ua: String,
    }

    impl CrawlerHarness {
        fn new(ip: &str, ua: &str) -> Self {
            Self {
                store: MemStore::default(),
                cfg: crate::config::defaults().clone(),
                ip: ip.to_string(),
                ua: ua.to_string(),
            }
        }

        fn serve(&self, uri: &str) -> MazeServeDecision {
            let req = Request::builder()
                .method(Method::Get)
                .uri(uri)
                .body(Vec::<u8>::new())
                .build();
            let path = uri.split('?').next().unwrap_or(uri);
            runtime::serve(
                &self.store,
                &self.cfg,
                &req,
                self.ip.as_str(),
                self.ua.as_str(),
                path,
            )
        }

        fn checkpoint(&self, token: &str) -> u16 {
            let body = serde_json::json!({
                "token": token,
                "flow_id": "ignored",
                "depth": 0,
                "checkpoint_reason": "simulation"
            })
            .to_string();
            let req = Request::builder()
                .method(Method::Post)
                .uri("/maze/checkpoint")
                .header("Content-Type", "application/json")
                .body(body.into_bytes())
                .build();
            *runtime::handle_checkpoint(
                &self.store,
                &self.cfg,
                &req,
                self.ip.as_str(),
                self.ua.as_str(),
            )
            .status()
        }
    }

    fn first_maze_link(html: &str) -> Option<String> {
        let marker = r#"href="/maze/"#;
        let idx = html.find(marker)?;
        let start = idx + 6; // start at /maze...
        let rest = &html[start..];
        let end = rest.find('"')?;
        Some(rest[..end].to_string())
    }

    fn mt_token_from_uri(uri: &str) -> Option<String> {
        let (_, query) = uri.split_once('?')?;
        for part in query.split('&') {
            let (key, value) = part.split_once('=').unwrap_or((part, ""));
            if key == "mt" && !value.is_empty() {
                return Some(value.to_string());
            }
        }
        None
    }

    #[test]
    fn crawler_harness_detects_replay_attempts() {
        let harness = CrawlerHarness::new("198.51.100.21", "ReplayBot/1.0");
        let entry = harness.serve("/maze/sim-entry");
        let MazeServeDecision::Serve(entry_page) = entry else {
            panic!("entry request should serve maze page");
        };
        let first = first_maze_link(entry_page.html.as_str()).expect("expected first maze link");

        let once = harness.serve(first.as_str());
        assert!(matches!(once, MazeServeDecision::Serve(_)));

        let replay = harness.serve(first.as_str());
        match replay {
            MazeServeDecision::Fallback(reason) => {
                assert_eq!(reason, MazeFallbackReason::TokenReplay)
            }
            MazeServeDecision::Serve(_) => panic!("replayed token should not be accepted"),
        }
    }

    #[test]
    fn crawler_harness_exposes_entropy_window_rotation() {
        let mut harness = CrawlerHarness::new("198.51.100.30", "FingerprintBot/2.0");
        harness.cfg.maze_entropy_window_seconds = 1;

        let first = harness.serve("/maze/fingerprint-target");
        let MazeServeDecision::Serve(first_page) = first else {
            panic!("first maze render should succeed");
        };

        thread::sleep(Duration::from_millis(1200));

        let second = harness.serve("/maze/fingerprint-target");
        let MazeServeDecision::Serve(second_page) = second else {
            panic!("second maze render should succeed");
        };

        assert_ne!(
            first_page.variant_id, second_page.variant_id,
            "variant IDs should rotate across entropy windows"
        );
    }

    #[test]
    fn crawler_harness_no_js_cohort_hits_checkpoint_fallback() {
        let harness = CrawlerHarness::new("198.51.100.41", "NoJsBot/1.0");
        let mut uri = "/maze/no-js-cohort".to_string();
        let mut saw_checkpoint_fallback = false;

        for _ in 0..8 {
            match harness.serve(uri.as_str()) {
                MazeServeDecision::Serve(page) => {
                    uri = first_maze_link(page.html.as_str()).expect("expected navigable maze link");
                }
                MazeServeDecision::Fallback(reason) => {
                    assert_eq!(reason, MazeFallbackReason::CheckpointMissing);
                    saw_checkpoint_fallback = true;
                    break;
                }
            }
        }

        assert!(
            saw_checkpoint_fallback,
            "no-JS traversal should trigger checkpoint fallback at deeper tiers"
        );
    }

    #[test]
    fn crawler_harness_js_cohort_progresses_with_checkpoints() {
        let harness = CrawlerHarness::new("198.51.100.42", "JsCapableCrawler/1.0");
        let mut uri = "/maze/js-cohort".to_string();

        for _ in 0..6 {
            let served = harness.serve(uri.as_str());
            let MazeServeDecision::Serve(page) = served else {
                panic!("js cohort should continue to receive maze pages");
            };

            if let Some(token) = mt_token_from_uri(uri.as_str()) {
                assert_eq!(
                    harness.checkpoint(token.as_str()),
                    204,
                    "checkpoint submission should be accepted"
                );
            }

            uri = first_maze_link(page.html.as_str()).expect("expected next maze link");
        }
    }

    #[test]
    fn crawler_harness_rejects_token_binding_bypass() {
        let first_harness = CrawlerHarness::new("198.51.100.55", "BypassBot/1.0");
        let second_harness = CrawlerHarness::new("203.0.113.19", "BypassBot/1.0");

        let entry = first_harness.serve("/maze/binding-source");
        let MazeServeDecision::Serve(entry_page) = entry else {
            panic!("source crawler should get maze entry page");
        };
        let tokenized_link = first_maze_link(entry_page.html.as_str()).expect("tokenized maze link");

        let bypass_attempt = second_harness.serve(tokenized_link.as_str());
        match bypass_attempt {
            MazeServeDecision::Fallback(reason) => {
                assert_eq!(reason, MazeFallbackReason::TokenBindingMismatch)
            }
            MazeServeDecision::Serve(_) => panic!("token reuse from different IP should fail"),
        }
    }
}
