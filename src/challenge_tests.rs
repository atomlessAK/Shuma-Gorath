// src/challenge_tests.rs
// Unit tests for the ARC-style challenge logic

#[cfg(test)]
mod tests {
    use super::super::challenge::{
        apply_transform,
        build_puzzle,
        generate_pair,
        handle_challenge_submit,
        make_seed_token,
        parse_submission,
        render_challenge,
        select_transform_pair,
        serve_challenge_page,
        ChallengeSeed,
        Transform,
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

    #[derive(Default)]
    struct TestStore {
        map: RefCell<HashMap<String, Vec<u8>>>,
    }

    impl super::super::challenge::KeyValueStore for TestStore {
        fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ()> {
            Ok(self.map.borrow().get(key).cloned())
        }
        fn set(&self, key: &str, value: &[u8]) -> Result<(), ()> {
            self.map.borrow_mut().insert(key.to_string(), value.to_vec());
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
    fn transform_pair_avoids_inverse_rotation() {
        let mut rng = StdRng::seed_from_u64(123);
        for _ in 0..50 {
            let pair = select_transform_pair(&mut rng);
            let a = pair[0];
            let b = pair[1];
            let inverse = matches!(
                (a, b),
                (Transform::RotateCw90, Transform::RotateCcw90) | (Transform::RotateCcw90, Transform::RotateCw90)
            );
            assert!(!inverse);
        }
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
        let expected = vec![
            13, 9, 5, 1,
            14, 10, 6, 2,
            15, 11, 7, 3,
            16, 12, 8, 4,
        ];
        assert_eq!(out, expected);
    }

    #[test]
    fn transform_mirror_horizontal_works() {
        let grid = grid_4x4();
        let out = apply_transform(&grid, 4, Transform::MirrorHorizontal);
        let expected = vec![
            4, 3, 2, 1,
            8, 7, 6, 5,
            12, 11, 10, 9,
            16, 15, 14, 13,
        ];
        assert_eq!(out, expected);
    }

    #[test]
    fn transform_shift_left_no_wrap() {
        let grid = grid_4x4();
        let out = apply_transform(&grid, 4, Transform::ShiftLeft);
        let expected = vec![
            2, 3, 4, 0,
            6, 7, 8, 0,
            10, 11, 12, 0,
            14, 15, 16, 0,
        ];
        assert_eq!(out, expected);
    }

    #[test]
    fn transform_drop_right_matches_spec() {
        let grid = grid_4x4();
        let out = apply_transform(&grid, 4, Transform::DropRight);
        let expected = vec![
            0, 1, 2, 3,
            0, 5, 6, 7,
            0, 9, 10, 11,
            0, 13, 14, 15,
        ];
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
            .uri("/challenge")
            .body(Vec::new())
            .build();
        let resp = render_challenge(&req);
        let body = String::from_utf8(resp.into_body()).unwrap();
        assert!(body.contains("id=\"challenge-output-grid\""));
    }

    #[test]
    fn render_challenge_uses_hidden_output_field() {
        let req = Request::builder()
            .method(Method::Get)
            .uri("/challenge")
            .body(Vec::new())
            .build();
        let resp = render_challenge(&req);
        let body = String::from_utf8(resp.into_body()).unwrap();
        assert!(body.contains("id=\"challenge-output\""));
        assert!(body.contains("type=\"hidden\""));
    }

    #[test]
    fn serve_challenge_page_requires_test_mode() {
        let req = Request::builder()
            .method(Method::Get)
            .uri("/challenge")
            .body(Vec::new())
            .build();
        let resp = serve_challenge_page(&req, false);
        assert_eq!(*resp.status(), 404u16);
        let resp_ok = serve_challenge_page(&req, true);
        assert_eq!(*resp_ok.status(), 200u16);
    }

    #[test]
    fn handle_challenge_submit_accepts_correct_solution() {
        let store = TestStore::default();
        let now = crate::admin::now_ts();
        let seed = ChallengeSeed {
            seed_id: "seed-2".to_string(),
            issued_at: now,
            expires_at: now + 300,
            ip_bucket: crate::ip::bucket_ip("unknown"),
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
            .uri("/challenge")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(body.as_bytes().to_vec())
            .build();
        let resp = handle_challenge_submit(&store, &req);
        assert_eq!(*resp.status(), 200u16);
    }

    #[test]
    fn handle_challenge_submit_rejects_incorrect_solution() {
        let store = TestStore::default();
        let now = crate::admin::now_ts();
        let seed = ChallengeSeed {
            seed_id: "seed-3".to_string(),
            issued_at: now,
            expires_at: now + 300,
            ip_bucket: crate::ip::bucket_ip("unknown"),
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
            .uri("/challenge")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(body.as_bytes().to_vec())
            .build();
        let resp = handle_challenge_submit(&store, &req);
        assert_eq!(*resp.status(), 403u16);
    }

    #[test]
    fn transform_shift_left_preserves_alt_cell() {
        let mut grid = vec![0u8; 16];
        grid[1] = 2;
        let out = apply_transform(&grid, 4, Transform::ShiftLeft);
        assert_eq!(out[0], 2);
    }
}
