// src/challenge_tests.rs
// Unit tests for the ARC-style challenge logic

#[cfg(test)]
mod tests {
    use super::super::challenge::{
        apply_transform,
        build_puzzle,
        parse_submission,
        ChallengeSeed,
        Transform,
    };

    fn grid_4x4() -> Vec<u8> {
        (1u8..=16u8).collect()
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
    fn parse_submission_accepts_bitstring_and_csv() {
        let bitstring = "1000000000000001";
        let parsed = parse_submission(bitstring, 4).unwrap();
        assert_eq!(parsed[0], 1);
        assert_eq!(parsed[15], 1);

        let csv = "0,15";
        let parsed_csv = parse_submission(csv, 4).unwrap();
        assert_eq!(parsed_csv[0], 1);
        assert_eq!(parsed_csv[15], 1);
    }

    #[test]
    fn parse_submission_rejects_invalid_length() {
        let err = parse_submission("10", 4).unwrap_err();
        assert_eq!(err, "invalid length");
    }
}
