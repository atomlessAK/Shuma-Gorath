use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};

use super::types::{ChallengePuzzle, ChallengeSeed, Transform};

pub(crate) fn build_puzzle(seed: &ChallengeSeed) -> ChallengePuzzle {
    let size = seed.grid_size as usize;
    let active = seed.active_cells as usize;
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed.seed);
    let mut training_pairs = Vec::new();
    for _ in 0..seed.training_count {
        let (input, output) = generate_pair(&mut rng, size, active, &seed.transforms);
        training_pairs.push((input, output));
    }
    let (test_input, test_output) = if let Some((input, output)) = training_pairs.first() {
        (input.clone(), output.clone())
    } else {
        generate_pair(&mut rng, size, active, &seed.transforms)
    };
    ChallengePuzzle {
        training_pairs,
        test_input,
        test_output,
        grid_size: size,
    }
}

fn generate_grid(rng: &mut impl Rng, size: usize, active: usize) -> Vec<u8> {
    let mut grid = vec![0u8; size * size];
    let mut indices: Vec<usize> = (0..grid.len()).collect();
    indices.shuffle(rng);
    let active_indices: Vec<usize> = indices.into_iter().take(active).collect();
    let mut has_one = false;
    let mut has_two = false;
    for idx in &active_indices {
        let val = if rng.random::<bool>() { 1 } else { 2 };
        if val == 1 {
            has_one = true;
        } else {
            has_two = true;
        }
        grid[*idx] = val;
    }
    if active >= 2 && (!has_one || !has_two) {
        let idx = active_indices[0];
        grid[idx] = if has_one { 2 } else { 1 };
    }
    grid
}

fn inverse_transform(transform: Transform) -> Option<Transform> {
    match transform {
        Transform::ShiftLeft => Some(Transform::ShiftRight),
        Transform::ShiftRight => Some(Transform::ShiftLeft),
        Transform::ShiftUp => Some(Transform::ShiftDown),
        Transform::ShiftDown => Some(Transform::ShiftUp),
        Transform::RotateCw90 => Some(Transform::RotateCcw90),
        Transform::RotateCcw90 => Some(Transform::RotateCw90),
        _ => None,
    }
}

const MIN_TRANSFORM_COUNT: usize = 4;
const MAX_TRANSFORM_COUNT: usize = 8;
#[cfg(test)]
const DEFAULT_TRANSFORM_COUNT: usize = 6;

fn all_transforms() -> Vec<Transform> {
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
}

#[cfg(test)]
pub(crate) fn parse_transform_count(value: Option<&str>) -> usize {
    let parsed = value
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(DEFAULT_TRANSFORM_COUNT);
    parsed.clamp(MIN_TRANSFORM_COUNT, MAX_TRANSFORM_COUNT)
}

pub(crate) fn transforms_for_count(count: usize) -> Vec<Transform> {
    let capped = count.clamp(MIN_TRANSFORM_COUNT, MAX_TRANSFORM_COUNT);
    all_transforms().into_iter().take(capped).collect()
}

pub(crate) fn select_transform_pair(rng: &mut impl Rng, available: &[Transform]) -> Vec<Transform> {
    let mut options = available.to_vec();
    options.shuffle(rng);
    let first = options[0];
    let inverse = inverse_transform(first);
    let second_choices: Vec<Transform> = options
        .into_iter()
        .skip(1)
        .filter(|candidate| Some(*candidate) != inverse)
        .collect();
    let second_idx = rng.random_range(0..second_choices.len());
    let second = second_choices[second_idx];
    vec![first, second]
}

fn apply_transforms(grid: &[u8], size: usize, transforms: &[Transform]) -> Vec<u8> {
    let mut current = grid.to_vec();
    for t in transforms {
        current = apply_transform(&current, size, *t);
    }
    current
}

pub(crate) fn generate_pair(
    rng: &mut impl Rng,
    size: usize,
    active: usize,
    transforms: &[Transform],
) -> (Vec<u8>, Vec<u8>) {
    const MAX_PAIR_ATTEMPTS: usize = 64;
    let mut last_input = Vec::new();
    let mut last_output = Vec::new();
    for _ in 0..MAX_PAIR_ATTEMPTS {
        let input = generate_grid(rng, size, active);
        let output = apply_transforms(&input, size, transforms);
        if output != input {
            return (input, output);
        }
        last_input = input;
        last_output = output;
    }
    (last_input, last_output)
}

pub(crate) fn apply_transform(grid: &[u8], size: usize, transform: Transform) -> Vec<u8> {
    let mut out = vec![0u8; size * size];
    match transform {
        Transform::RotateCw90 => {
            for r in 0..size {
                for c in 0..size {
                    let src = idx(r, c, size);
                    let dst = idx(c, size - 1 - r, size);
                    out[dst] = grid[src];
                }
            }
        }
        Transform::RotateCcw90 => {
            for r in 0..size {
                for c in 0..size {
                    let src = idx(r, c, size);
                    let dst = idx(size - 1 - c, r, size);
                    out[dst] = grid[src];
                }
            }
        }
        Transform::MirrorHorizontal => {
            for r in 0..size {
                for c in 0..size {
                    let src = idx(r, c, size);
                    let dst = idx(size - 1 - r, c, size);
                    out[dst] = grid[src];
                }
            }
        }
        Transform::MirrorVertical => {
            for r in 0..size {
                for c in 0..size {
                    let src = idx(r, c, size);
                    let dst = idx(r, size - 1 - c, size);
                    out[dst] = grid[src];
                }
            }
        }
        Transform::ShiftUp | Transform::DropTop => {
            for r in 1..size {
                for c in 0..size {
                    let src = idx(r, c, size);
                    let dst = idx(r - 1, c, size);
                    out[dst] = grid[src];
                }
            }
        }
        Transform::ShiftDown | Transform::DropBottom => {
            for r in 0..size - 1 {
                for c in 0..size {
                    let src = idx(r, c, size);
                    let dst = idx(r + 1, c, size);
                    out[dst] = grid[src];
                }
            }
        }
        Transform::ShiftLeft | Transform::DropLeft => {
            for r in 0..size {
                for c in 1..size {
                    let src = idx(r, c, size);
                    let dst = idx(r, c - 1, size);
                    out[dst] = grid[src];
                }
            }
        }
        Transform::ShiftRight | Transform::DropRight => {
            for r in 0..size {
                for c in 0..size - 1 {
                    let src = idx(r, c, size);
                    let dst = idx(r, c + 1, size);
                    out[dst] = grid[src];
                }
            }
        }
    }
    out
}

pub(crate) fn parse_submission(input: &str, size: usize) -> Result<Vec<u8>, &'static str> {
    let trimmed = input.trim();
    let expected = size * size;
    if trimmed.is_empty() {
        return Err("invalid length");
    }
    if !trimmed.chars().all(|c| c == '0' || c == '1' || c == '2') {
        return Err("invalid format");
    }
    if trimmed.len() != expected {
        return Err("invalid length");
    }
    let out = trimmed
        .chars()
        .map(|c| match c {
            '0' => 0,
            '1' => 1,
            _ => 2,
        })
        .collect();
    Ok(out)
}

fn idx(row: usize, col: usize, size: usize) -> usize {
    row * size + col
}
