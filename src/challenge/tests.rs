// src/challenge/tests.rs
// Unit tests for the ARC-style challenge logic

#[cfg(test)]
mod tests {
    use super::super::{
        apply_transform, build_puzzle, generate_pair, handle_challenge_submit, make_seed_token,
        parse_submission, parse_transform_count, render_challenge, select_transform_pair,
        serve_challenge_page, transforms_for_count, ChallengeSeed, Transform,
    };
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    use spin_sdk::http::{Method, Request};
    use std::cell::RefCell;
    use std::collections::HashMap;

    fn grid_4x4() -> Vec<u8> {
        (1u8..=16u8).collect()
    }

    fn grid_to_tritstring(grid: &[u8]) -> String {
        grid.iter().map(|v| char::from(b'0' + *v)).collect()
    }

    fn header_value(resp: &spin_sdk::http::Response, name: &str) -> Option<String> {
        resp.headers()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .and_then(|(_, v)| v.as_str().map(|s| s.to_string()))
    }

    #[derive(Default)]
    struct TestStore {
        map: RefCell<HashMap<String, Vec<u8>>>,
    }

    impl super::super::KeyValueStore for TestStore {
        fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ()> {
            Ok(self.map.borrow().get(key).cloned())
        }
        fn set(&self, key: &str, value: &[u8]) -> Result<(), ()> {
            self.map
                .borrow_mut()
                .insert(key.to_string(), value.to_vec());
            Ok(())
        }
        fn delete(&self, key: &str) -> Result<(), ()> {
            self.map.borrow_mut().remove(key);
            Ok(())
        }
    }

    #[test]
    fn deterministic_seed_produces_same_output() {
        let seed = ChallengeSeed {
            seed_id: "seed-1".to_string(),
            issued_at: 1,
            expires_at: 999,
            ip_bucket: "bucket".to_string(),
            grid_size: 4,
            active_cells: 4,
            transforms: vec![Transform::RotateCw90, Transform::ShiftDown],
            training_count: 2,
            seed: 12345,
        };
        let a = build_puzzle(&seed);
        let b = build_puzzle(&seed);
        assert_eq!(a.test_output, b.test_output);
    }

    #[test]
    fn generated_inputs_include_both_colors() {
        let seed = ChallengeSeed {
            seed_id: "seed-colors".to_string(),
            issued_at: 1,
            expires_at: 999,
            ip_bucket: "bucket".to_string(),
            grid_size: 4,
            active_cells: 5,
            transforms: vec![Transform::ShiftLeft, Transform::ShiftDown],
            training_count: 2,
            seed: 6789,
        };
        let puzzle = build_puzzle(&seed);
        let has_one = puzzle.test_input.iter().any(|v| *v == 1);
        let has_two = puzzle.test_input.iter().any(|v| *v == 2);
        assert!(has_one && has_two);
    }

    #[test]
    fn transform_pair_returns_two_distinct_transforms() {
        let mut rng = StdRng::seed_from_u64(123);
        let available = transforms_for_count(8);
        for _ in 0..50 {
            let pair = select_transform_pair(&mut rng, &available);
            assert_ne!(pair[0], pair[1]);
        }
    }

    #[test]
    fn transform_pair_avoids_cancel_pairs() {
        let mut rng = StdRng::seed_from_u64(456);
        let available = transforms_for_count(8);
        for _ in 0..200 {
            let pair = select_transform_pair(&mut rng, &available);
            let is_cancel = matches!(
                (pair[0], pair[1]),
                (Transform::ShiftLeft, Transform::ShiftRight)
                    | (Transform::ShiftRight, Transform::ShiftLeft)
                    | (Transform::ShiftUp, Transform::ShiftDown)
                    | (Transform::ShiftDown, Transform::ShiftUp)
                    | (Transform::RotateCw90, Transform::RotateCcw90)
                    | (Transform::RotateCcw90, Transform::RotateCw90)
            );
            assert!(!is_cancel);
        }
    }

    #[test]
    fn transform_pair_uses_only_eight_transforms() {
        let mut rng = StdRng::seed_from_u64(321);
        let available = transforms_for_count(8);
        for _ in 0..100 {
            let pair = select_transform_pair(&mut rng, &available);
            for t in pair {
                assert!(matches!(
                    t,
                    Transform::RotateCw90
                        | Transform::RotateCcw90
                        | Transform::MirrorHorizontal
                        | Transform::MirrorVertical
                        | Transform::ShiftUp
                        | Transform::ShiftDown
                        | Transform::ShiftLeft
                        | Transform::ShiftRight
                ));
            }
        }
    }

    #[test]
    fn parse_transform_count_clamps_to_valid_range() {
        assert_eq!(parse_transform_count(None), 6);
        assert_eq!(parse_transform_count(Some("bogus")), 6);
        assert_eq!(parse_transform_count(Some("2")), 4);
        assert_eq!(parse_transform_count(Some("4")), 4);
        assert_eq!(parse_transform_count(Some("6")), 6);
        assert_eq!(parse_transform_count(Some("99")), 8);
    }

    #[test]
    fn transforms_for_count_uses_ordered_prefix() {
        assert_eq!(
            transforms_for_count(4),
            vec![
                Transform::ShiftUp,
                Transform::ShiftDown,
                Transform::ShiftLeft,
                Transform::ShiftRight,
            ]
        );
        assert_eq!(
            transforms_for_count(6),
            vec![
                Transform::ShiftUp,
                Transform::ShiftDown,
                Transform::ShiftLeft,
                Transform::ShiftRight,
                Transform::RotateCw90,
                Transform::RotateCcw90,
            ]
        );
        assert_eq!(
            transforms_for_count(8),
            vec![
                Transform::ShiftUp,
                Transform::ShiftDown,
                Transform::ShiftLeft,
                Transform::ShiftRight,
                Transform::RotateCw90,
                Transform::RotateCcw90,
                Transform::MirrorHorizontal,
                Transform::MirrorVertical,
            ]
        );
    }

    #[test]
    fn generate_pair_avoids_identity_output() {
        let mut rng = StdRng::seed_from_u64(999);
        let transforms = vec![Transform::MirrorHorizontal, Transform::ShiftLeft];
        let (input, output) = generate_pair(&mut rng, 4, 4, &transforms);
        assert_ne!(input, output);
    }

    #[test]
    fn transform_rotate_cw_works() {
        let grid = grid_4x4();
        let out = apply_transform(&grid, 4, Transform::RotateCw90);
        let expected = vec![13, 9, 5, 1, 14, 10, 6, 2, 15, 11, 7, 3, 16, 12, 8, 4];
        assert_eq!(out, expected);
    }

    #[test]
    fn transform_mirror_horizontal_works() {
        let grid = grid_4x4();
        let out = apply_transform(&grid, 4, Transform::MirrorHorizontal);
        let expected = vec![13, 14, 15, 16, 9, 10, 11, 12, 5, 6, 7, 8, 1, 2, 3, 4];
        assert_eq!(out, expected);
    }

    #[test]
    fn transform_mirror_vertical_works() {
        let grid = grid_4x4();
        let out = apply_transform(&grid, 4, Transform::MirrorVertical);
        let expected = vec![4, 3, 2, 1, 8, 7, 6, 5, 12, 11, 10, 9, 16, 15, 14, 13];
        assert_eq!(out, expected);
    }

    #[test]
    fn transform_shift_left_no_wrap() {
        let grid = grid_4x4();
        let out = apply_transform(&grid, 4, Transform::ShiftLeft);
        let expected = vec![2, 3, 4, 0, 6, 7, 8, 0, 10, 11, 12, 0, 14, 15, 16, 0];
        assert_eq!(out, expected);
    }

    #[test]
    fn transform_drop_right_matches_spec() {
        let grid = grid_4x4();
        let out = apply_transform(&grid, 4, Transform::DropRight);
        let expected = vec![0, 1, 2, 3, 0, 5, 6, 7, 0, 9, 10, 11, 0, 13, 14, 15];
        assert_eq!(out, expected);
    }

    #[test]
    fn parse_submission_accepts_tritstring() {
        let tritstring = "1200000000000002";
        let parsed = parse_submission(tritstring, 4).unwrap();
        assert_eq!(parsed[0], 1);
        assert_eq!(parsed[15], 2);
    }

    #[test]
    fn parse_submission_rejects_invalid_length() {
        let err = parse_submission("10", 4).unwrap_err();
        assert_eq!(err, "invalid length");
    }

    #[test]
    fn parse_submission_rejects_csv_format() {
        let err = parse_submission("0,15", 4).unwrap_err();
        assert_eq!(err, "invalid format");
    }

    #[test]
    fn render_challenge_includes_output_grid_id() {
        let req = Request::builder()
            .method(Method::Get)
            .uri("/challenge/puzzle")
            .body(Vec::new())
            .build();
        let resp = render_challenge(&req, 6);
        let body = String::from_utf8(resp.into_body()).unwrap();
        assert!(body.contains("id=\"challenge-output-grid\""));
    }

    #[test]
    fn render_challenge_uses_hidden_output_field() {
        let req = Request::builder()
            .method(Method::Get)
            .uri("/challenge/puzzle")
            .body(Vec::new())
            .build();
        let resp = render_challenge(&req, 6);
        let body = String::from_utf8(resp.into_body()).unwrap();
        assert!(body.contains("id=\"challenge-output\""));
        assert!(body.contains("type=\"hidden\""));
    }

    #[test]
    fn render_challenge_has_transform_selectors_and_one_example() {
        let req = Request::builder()
            .method(Method::Get)
            .uri("/challenge/puzzle")
            .body(Vec::new())
            .build();
        let resp = render_challenge(&req, 6);
        let body = String::from_utf8(resp.into_body()).unwrap();
        assert!(body.contains("Puzzle"));
        assert!(!body.contains("id=\"transform-1\""));
        assert!(!body.contains("id=\"transform-2\""));
        assert!(body.contains("name=\"transform_1\""));
        assert!(body.contains("name=\"transform_2\""));
        assert!(body.contains("type=\"radio\""));
        assert!(body.contains("class=\"legend-options\""));
        assert!(body.contains("class=\"legend-pick-label\""));
        assert!(!body.contains("class=\"legend-table\""));
        assert!(!body.contains("class=\"legend-check\""));
        assert!(body.contains("Which 2 transforms were applied?"));
        assert!(
            body.contains("<div class=\"legend-subtitle\">Which 2 transforms were applied?</div>")
        );
        assert!(!body.contains("Choose 2 transforms:"));
        assert!(!body.contains("Your turn"));
        assert!(!body.contains("Example 2"));
        assert!(!body.contains("Mirror horizontally"));
        assert!(!body.contains("Mirror vertically"));
        assert!(body.contains("event.target.name === 'transform_1'"));
        assert!(body.contains("event.target.name === 'transform_2'"));
        let after_pos = body.find("After").unwrap();
        let legend_pos = body.find("Which 2 transforms were applied?").unwrap();
        assert!(after_pos < legend_pos);
    }

    #[test]
    fn render_challenge_hides_debug_transform_text() {
        let req = Request::builder()
            .method(Method::Get)
            .uri("/challenge/puzzle")
            .body(Vec::new())
            .build();
        let resp = render_challenge(&req, 6);
        let body = String::from_utf8(resp.into_body()).unwrap();
        assert!(!body.contains("Debug transforms:"));
    }

    #[test]
    fn serve_challenge_page_requires_test_mode() {
        let req = Request::builder()
            .method(Method::Get)
            .uri("/challenge/puzzle")
            .body(Vec::new())
            .build();
        let resp = serve_challenge_page(&req, false, 6);
        assert_eq!(*resp.status(), 404u16);
        let resp_ok = serve_challenge_page(&req, true, 6);
        assert_eq!(*resp_ok.status(), 200u16);
    }

    #[test]
    fn serve_challenge_page_sets_no_store_cache_control() {
        let req = Request::builder()
            .method(Method::Get)
            .uri("/challenge/puzzle")
            .body(Vec::new())
            .build();
        let resp = serve_challenge_page(&req, true, 6);
        assert_eq!(
            header_value(&resp, "Cache-Control").as_deref(),
            Some("no-store")
        );
    }

    #[test]
    fn handle_challenge_submit_accepts_correct_solution() {
        let store = TestStore::default();
        let now = crate::admin::now_ts();
        let seed = ChallengeSeed {
            seed_id: "seed-2".to_string(),
            issued_at: now,
            expires_at: now + 300,
            ip_bucket: crate::signals::ip_identity::bucket_ip("unknown"),
            grid_size: 4,
            active_cells: 4,
            transforms: vec![Transform::RotateCw90, Transform::ShiftDown],
            training_count: 2,
            seed: 9999,
        };
        let puzzle = build_puzzle(&seed);
        let output = grid_to_tritstring(&puzzle.test_output);
        let seed_token = make_seed_token(&seed);
        let body = format!("seed={}&output={}", seed_token, output);
        let req = Request::builder()
            .method(Method::Post)
            .uri("/challenge/puzzle")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(body.as_bytes().to_vec())
            .build();
        let resp = handle_challenge_submit(&store, &req);
        assert_eq!(*resp.status(), 200u16);
        assert_eq!(
            header_value(&resp, "Cache-Control").as_deref(),
            Some("no-store")
        );
    }

    #[test]
    fn handle_challenge_submit_rejects_incorrect_solution() {
        let store = TestStore::default();
        let now = crate::admin::now_ts();
        let seed = ChallengeSeed {
            seed_id: "seed-3".to_string(),
            issued_at: now,
            expires_at: now + 300,
            ip_bucket: crate::signals::ip_identity::bucket_ip("unknown"),
            grid_size: 4,
            active_cells: 4,
            transforms: vec![Transform::RotateCw90, Transform::ShiftDown],
            training_count: 2,
            seed: 4242,
        };
        let puzzle = build_puzzle(&seed);
        let mut output = grid_to_tritstring(&puzzle.test_output);
        output.replace_range(0..1, if &output[0..1] == "1" { "0" } else { "1" });
        let seed_token = make_seed_token(&seed);
        let body = format!("seed={}&output={}", seed_token, output);
        let req = Request::builder()
            .method(Method::Post)
            .uri("/challenge/puzzle")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(body.as_bytes().to_vec())
            .build();
        let resp = handle_challenge_submit(&store, &req);
        assert_eq!(*resp.status(), 403u16);
    }

    #[test]
    fn handle_challenge_submit_is_single_attempt() {
        let store = TestStore::default();
        let now = crate::admin::now_ts();
        let seed = ChallengeSeed {
            seed_id: "seed-once".to_string(),
            issued_at: now,
            expires_at: now + 300,
            ip_bucket: crate::signals::ip_identity::bucket_ip("unknown"),
            grid_size: 4,
            active_cells: 7,
            transforms: vec![Transform::RotateCw90, Transform::ShiftDown],
            training_count: 1,
            seed: 424242,
        };
        let puzzle = build_puzzle(&seed);
        let mut wrong = grid_to_tritstring(&puzzle.test_output);
        wrong.replace_range(0..1, if &wrong[0..1] == "1" { "0" } else { "1" });
        let seed_token = make_seed_token(&seed);
        let body_wrong = format!("seed={}&output={}", seed_token, wrong);
        let req_wrong = Request::builder()
            .method(Method::Post)
            .uri("/challenge/puzzle")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(body_wrong.as_bytes().to_vec())
            .build();
        let resp_wrong = handle_challenge_submit(&store, &req_wrong);
        assert_eq!(*resp_wrong.status(), 403u16);
        assert_eq!(
            header_value(&resp_wrong, "Cache-Control").as_deref(),
            Some("no-store")
        );
        let wrong_body = String::from_utf8(resp_wrong.into_body()).unwrap();
        assert!(wrong_body.contains("Incorrect."));
        assert!(wrong_body.contains("Request new challenge."));

        let correct = grid_to_tritstring(&puzzle.test_output);
        let body_correct = format!("seed={}&output={}", seed_token, correct);
        let req_correct = Request::builder()
            .method(Method::Post)
            .uri("/challenge/puzzle")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(body_correct.as_bytes().to_vec())
            .build();
        let resp_correct = handle_challenge_submit(&store, &req_correct);
        assert_eq!(*resp_correct.status(), 403u16);
        let correct_body = String::from_utf8(resp_correct.into_body()).unwrap();
        assert!(correct_body.contains("Expired"));
        assert!(!correct_body.contains("Seed already used"));
        assert!(correct_body.contains("Request new challenge."));
    }

    #[test]
    fn handle_challenge_submit_expired_seed_shows_expired_message() {
        let store = TestStore::default();
        let now = crate::admin::now_ts();
        let seed = ChallengeSeed {
            seed_id: "seed-expired".to_string(),
            issued_at: now - 1000,
            expires_at: now - 1,
            ip_bucket: crate::signals::ip_identity::bucket_ip("unknown"),
            grid_size: 4,
            active_cells: 7,
            transforms: vec![Transform::RotateCw90, Transform::ShiftDown],
            training_count: 1,
            seed: 424243,
        };
        let seed_token = make_seed_token(&seed);
        let req = Request::builder()
            .method(Method::Post)
            .uri("/challenge/puzzle")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(
                format!("seed={}&output=0000000000000000", seed_token)
                    .as_bytes()
                    .to_vec(),
            )
            .build();
        let resp = handle_challenge_submit(&store, &req);
        assert_eq!(*resp.status(), 403u16);
        let body = String::from_utf8(resp.into_body()).unwrap();
        assert!(body.contains("Expired"));
        assert!(body.contains("Request new challenge."));
    }

    #[test]
    fn handle_challenge_submit_invalid_seed_uses_generic_forbidden_message() {
        let store = TestStore::default();
        let req = Request::builder()
            .method(Method::Post)
            .uri("/challenge/puzzle")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(b"seed=not-a-valid-token&output=0000000000000000".to_vec())
            .build();
        let resp = handle_challenge_submit(&store, &req);
        assert_eq!(*resp.status(), 403u16);
        assert_eq!(
            header_value(&resp, "Cache-Control").as_deref(),
            Some("no-store")
        );
        let body = String::from_utf8(resp.into_body()).unwrap();
        assert!(body.contains("Forbidden. Please request a new challenge."));
        assert!(body.contains("Request new challenge."));
    }

    #[test]
    fn handle_challenge_submit_rejects_oversized_form_body() {
        let store = TestStore::default();
        let oversized = "a".repeat(crate::request_validation::MAX_CHALLENGE_FORM_BYTES + 1);
        let req = Request::builder()
            .method(Method::Post)
            .uri("/challenge/puzzle")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(oversized.into_bytes())
            .build();
        let resp = handle_challenge_submit(&store, &req);
        assert_eq!(*resp.status(), 403u16);
        let body = String::from_utf8(resp.into_body()).unwrap();
        assert!(body.contains("Forbidden. Please request a new challenge."));
    }

    #[test]
    fn transform_shift_left_preserves_alt_cell() {
        let mut grid = vec![0u8; 16];
        grid[1] = 2;
        let out = apply_transform(&grid, 4, Transform::ShiftLeft);
        assert_eq!(out[0], 2);
    }
}
