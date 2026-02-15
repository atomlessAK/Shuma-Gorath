#[cfg(test)]
mod tests {
    use crate::maze::runtime::{self, MazeServeDecision};
    use crate::maze::state::MazeStateStore;
    use crate::maze::token;
    use spin_sdk::http::{Method, Request};
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Mutex;

    #[derive(Default)]
    struct CountingStore {
        data: Mutex<HashMap<String, Vec<u8>>>,
        get_ops: AtomicU64,
        set_ops: AtomicU64,
        bytes_written: AtomicU64,
    }

    impl CountingStore {
        fn set_ops(&self) -> u64 {
            self.set_ops.load(Ordering::Relaxed)
        }

        fn bytes_written(&self) -> u64 {
            self.bytes_written.load(Ordering::Relaxed)
        }
    }

    impl MazeStateStore for CountingStore {
        fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ()> {
            self.get_ops.fetch_add(1, Ordering::Relaxed);
            Ok(self.data.lock().unwrap().get(key).cloned())
        }

        fn set(&self, key: &str, value: &[u8]) -> Result<(), ()> {
            self.set_ops.fetch_add(1, Ordering::Relaxed);
            self.bytes_written
                .fetch_add(value.len() as u64, Ordering::Relaxed);
            self.data
                .lock()
                .unwrap()
                .insert(key.to_string(), value.to_vec());
            Ok(())
        }
    }

    fn first_maze_link_and_pow(html: &str) -> Option<(String, Option<u8>)> {
        for fragment in html.split("<a ") {
            if !fragment.contains("data-link-kind=\"maze\"") {
                continue;
            }
            let href_idx = fragment.find("href=\"")?;
            let start = href_idx + 6;
            let rest = &fragment[start..];
            let end = rest.find('"')?;
            let href = rest[..end].to_string();

            let pow = fragment.find("data-pow-difficulty=\"").and_then(|idx| {
                let raw = &fragment[idx + "data-pow-difficulty=\"".len()..];
                let end = raw.find('"')?;
                raw[..end].parse::<u8>().ok()
            });
            return Some((href, pow));
        }
        None
    }

    fn extract_bootstrap_json(html: &str) -> serde_json::Value {
        let marker = "<script id=\"maze-bootstrap\" type=\"application/json\">";
        let start = html.find(marker).expect("bootstrap should exist") + marker.len();
        let end = html[start..]
            .find("</script>")
            .map(|offset| start + offset)
            .expect("bootstrap script should terminate");
        serde_json::from_str(&html[start..end]).expect("bootstrap json should parse")
    }

    fn query_param(uri: &str, key: &str) -> Option<String> {
        let (_, query) = uri.split_once('?')?;
        for part in query.split('&') {
            let (k, v) = part.split_once('=').unwrap_or((part, ""));
            if k == key {
                return Some(v.to_string());
            }
        }
        None
    }

    fn solve_pow_nonce(raw_token: &str, difficulty: u8) -> (u64, u64) {
        for nonce in 0..2_000_000u64 {
            if token::verify_micro_pow(raw_token, nonce.to_string().as_str(), difficulty) {
                return (nonce, nonce + 1);
            }
        }
        panic!("expected PoW nonce should be found for benchmark");
    }

    fn with_pow_nonce(uri: &str, difficulty: Option<u8>, pow_iterations: &mut u64) -> String {
        let Some(bits) = difficulty else {
            return uri.to_string();
        };
        let raw_token = query_param(uri, "mt").expect("tokenized maze href should include mt");
        let (nonce, attempts) = solve_pow_nonce(raw_token.as_str(), bits);
        *pow_iterations = pow_iterations.saturating_add(attempts);
        format!("{uri}&mpn={nonce}")
    }

    #[test]
    fn maze_asymmetry_benchmark_guardrails_hold() {
        let store = CountingStore::default();
        let mut cfg = crate::config::defaults().clone();
        cfg.maze_client_expansion_enabled = true;
        cfg.maze_max_links = 8;
        cfg.maze_server_visible_links = 4;
        cfg.maze_token_branch_budget = 4;
        cfg.maze_micro_pow_enabled = true;
        cfg.maze_micro_pow_depth_start = 5;
        cfg.maze_micro_pow_base_difficulty = 12;

        let ip = "198.51.100.88";
        let ua = "BenchmarkCrawler/1.0";
        let legacy_bytes = crate::maze::generate_maze_page(
            "/maze/benchmark-baseline",
            &crate::maze::MazeConfig::default(),
        )
        .len();

        let mut uri = "/maze/benchmark-entry".to_string();
        let mut pages_served = 0u64;
        let mut total_page_bytes = 0usize;
        let mut attacker_requests = 0u64;
        let mut attacker_pow_iterations = 0u64;
        let mut issue_links_calls = 0u64;

        for _ in 0..6 {
            attacker_requests += 1;
            let req = Request::builder()
                .method(Method::Get)
                .uri(uri.as_str())
                .body(Vec::<u8>::new())
                .build();
            let path = uri.split('?').next().expect("path should exist");
            let decision = runtime::serve(&store, &cfg, &req, ip, ua, path, Some(9));
            let MazeServeDecision::Serve(page) = decision else {
                panic!("benchmark traversal should continue serving maze pages");
            };
            pages_served += 1;
            total_page_bytes = total_page_bytes.saturating_add(page.bytes);

            let bootstrap = extract_bootstrap_json(page.html.as_str());
            let checkpoint_token = bootstrap
                .get("checkpoint_token")
                .and_then(|value| value.as_str())
                .unwrap_or_default();
            if !checkpoint_token.is_empty() {
                attacker_requests += 1;
                let checkpoint_req = Request::builder()
                    .method(Method::Post)
                    .uri("/maze/checkpoint")
                    .header("Content-Type", "application/json")
                    .body(
                        serde_json::json!({
                            "token": checkpoint_token,
                            "flow_id": bootstrap.get("flow_id").and_then(|value| value.as_str()).unwrap_or_default(),
                            "depth": bootstrap.get("depth").and_then(|value| value.as_u64()).unwrap_or(0),
                            "checkpoint_reason": "benchmark"
                        })
                        .to_string()
                        .into_bytes(),
                    )
                    .build();
                let checkpoint_response =
                    runtime::handle_checkpoint(&store, &cfg, &checkpoint_req, ip, ua);
                assert_eq!(*checkpoint_response.status(), 204);

                let expansion = bootstrap
                    .get("client_expansion")
                    .cloned()
                    .unwrap_or_else(|| serde_json::json!({}));
                let hidden_count = expansion
                    .get("hidden_count")
                    .and_then(|value| value.as_u64())
                    .unwrap_or(0);
                if hidden_count > 0 {
                    attacker_requests += 1;
                    issue_links_calls += 1;
                    let issue_req = Request::builder()
                        .method(Method::Post)
                        .uri("/maze/issue-links")
                        .header("Content-Type", "application/json")
                        .body(
                            serde_json::json!({
                                "parent_token": checkpoint_token,
                                "flow_id": bootstrap.get("flow_id").and_then(|value| value.as_str()).unwrap_or_default(),
                                "entropy_nonce": bootstrap.get("entropy_nonce").and_then(|value| value.as_str()).unwrap_or_default(),
                                "path_prefix": bootstrap.get("path_prefix").and_then(|value| value.as_str()).unwrap_or("/maze/"),
                                "seed": expansion.get("seed").and_then(|value| value.as_u64()).unwrap_or(0),
                                "seed_sig": expansion.get("seed_sig").and_then(|value| value.as_str()).unwrap_or_default(),
                                "hidden_count": hidden_count,
                                "requested_hidden_count": hidden_count.min(2),
                                "segment_len": expansion.get("segment_len").and_then(|value| value.as_u64()).unwrap_or(16),
                                "candidates": []
                            })
                            .to_string()
                            .into_bytes(),
                        )
                        .build();
                    let issue_response =
                        runtime::handle_issue_links(&store, &cfg, &issue_req, ip, ua);
                    assert_eq!(*issue_response.status(), 200);
                }
            }

            let (next_href, pow_bits) =
                first_maze_link_and_pow(page.html.as_str()).expect("maze page should include link");
            uri = with_pow_nonce(next_href.as_str(), pow_bits, &mut attacker_pow_iterations);
        }

        let average_page_bytes = total_page_bytes / pages_served as usize;
        let host_set_ops = store.set_ops();
        let host_write_bytes = store.bytes_written();

        eprintln!(
            "maze_benchmark pages={} avg_page_bytes={} legacy_bytes={} host_set_ops={} host_write_bytes={} attacker_requests={} issue_links_calls={} attacker_pow_iterations={}",
            pages_served,
            average_page_bytes,
            legacy_bytes,
            host_set_ops,
            host_write_bytes,
            attacker_requests,
            issue_links_calls,
            attacker_pow_iterations
        );

        assert!(
            average_page_bytes < legacy_bytes,
            "modern maze pages should remain smaller than legacy inline pages"
        );
        assert!(
            average_page_bytes <= 10_000,
            "average maze page payload should stay within budget guardrail"
        );
        assert!(
            host_set_ops <= pages_served.saturating_mul(14),
            "per-hop host KV writes should remain bounded"
        );
        assert!(
            attacker_requests >= pages_served.saturating_add(4),
            "traversal should require attacker request amplification"
        );
        assert!(
            issue_links_calls >= 3,
            "progressive hidden-link issuance should be exercised repeatedly"
        );
        assert!(
            attacker_pow_iterations > 0,
            "deep traversal should require attacker-side PoW work"
        );
    }
}
