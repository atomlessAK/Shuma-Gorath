# Challenge Legibility Improvements

## Context
The ARC-style human verification challenge is functional but too difficult for human users. The goal is to improve clarity without reducing bot resistance.

## Goals
- Make the transforms easier to infer.
- Increase visual anchors in each grid.
- Keep the challenge edge-friendly and deterministic.
- Preserve a single challenge flow (no alternate transport).

## Non-Goals
- Change the transform set or add new transform types.
- Add third-party CAPTCHA dependencies.
- Add ML-based verification.

## Behavioral Changes
- Increase active cells from 3-6 to 5-7.
- Add a second tone for active cells:
  - Black for value 1
  - Pink (rgb(255,205,235)) for value 2
- Add a legend panel above the examples showing all available transforms and the instruction: "Two of these are being applied to the input grids."
- Output grid interaction cycles through empty -> black -> pink -> empty.

## Data Model
- Grid values become ternary: 0 (empty), 1 (black), 2 (pink).
- Submission format is a base-3 string with length grid_size * grid_size.
- Transforms operate on u8 values and preserve color values.

## Legend Rendering
- Each transform is rendered with a small 4x4 grid icon and overlays:
  - Rotations: clockwise or counter-clockwise arrow.
  - Mirrors: dashed reflection line.
  - Shifts: two dashed lines plus direction arrow.
  - Drops: dashed line plus direction arrow.

## Testing
- Update parse_submission tests to accept trit strings and reject CSV.
- Add transform test to ensure alt cells are preserved.
- Keep existing transform tests for correctness.

## Rollout Notes
- Debug transform list remains visible on the page for now to validate behavior.
