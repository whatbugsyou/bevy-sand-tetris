# AGENTS.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Project Overview

A Tetris game with sand physics built in Rust using Bevy 0.18.0. Tetromino pieces spawn and fall as in classic Tetris, but each mino is subdivided into a 6×6 grid of "grains" that simulate sand behavior (falling, diagonal sliding) after settling. Lines are cleared when a same-color connected component (8-connectivity BFS) spans from the left wall to the right wall.

## Build & Run

```sh
cargo build          # compile
cargo run            # run the game
cargo clippy         # lint
cargo fmt --check    # check formatting
```

No test suite exists yet. The project has no CI configuration.

## Architecture

Bevy ECS architecture — all game logic lives in systems that run each frame.

### Coordinate System

The board is a fixed grid defined by constants in `src/constants.rs`. Key relationship:

- **Base board**: 10 columns × 22 rows (classic Tetris dimensions)
- **Grain board**: 60 columns × 132 rows (`PIECE_SUBDIVISION = 6` multiplier)
- **Window**: sized to iPhone 14 (390×844), with `GRAIN_SIZE` derived from height / rows

`BoardGrid` (in `src/resources.rs`) maps between world coordinates and grid `(col, row)` where row 0 is the bottom (floor). It owns a 2D array of `Option<Entity>` tracking every settled grain.

### System Pipeline

All Update systems run in a strict chain (order matters):

1. **spawn** — spawns a random `TetrominoShape` as individual `Grain` entities tagged with `ActivePiece`, each grain is a separate Bevy entity with a `Sprite`
2. **input** — handles horizontal movement (with key-repeat), rotation (pivot = center of mass), soft drop (hold down), hard drop (space)
3. **physics** — active piece falls one grain-row per `FallTimer` tick; on collision, grains are settled into `BoardGrid` and `ActivePiece` is removed
4. **clear** — BFS over `BoardGrid` to find same-color connected components spanning both walls; triggers a flash animation via `ClearEffect`/`PendingClear`, then despawns grains and awards score
5. **sand** — settled grains simulate sand: fall straight down, or slide diagonally (random left/right bias); paused during clear animations
6. **game_over** — checks if any grain reached the top row
7. **update_score_ui / game_over_ui** — UI updates

### Key Design Patterns

- **Active vs Settled**: `ActivePiece` component marks grains that are still falling as a group. Once locked, grains lose this component and are tracked in `BoardGrid.cells`.
- **ClearEffect state machine**: `ClearEffect.pending` being `Some` pauses sand physics and triggers the flash animation. When elapsed time exceeds duration, grains are despawned.
- **Collision checks** exclude sibling grains: movement/rotation validity checks allow a target cell if it's occupied by another grain in the same active piece.

### Controls

- Arrow Left/Right: move piece (with hold-repeat)
- Arrow Up / Z: rotate 90° clockwise
- Arrow Down (hold): soft drop
- Space: hard drop (instant lock)
