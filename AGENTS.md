# AGENTS.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Project Overview

A Tetris game with sand physics built in Rust using Bevy 0.18.0. Each tetromino mino is subdivided into an 8×8 grid of "grains" that simulate sand behavior (falling, diagonal sliding) after settling. Instead of auto-spawning, the player selects from 3 candidate pieces shown in a bottom UI panel. Lines are cleared when a same-color connected component (8-connectivity BFS) spans from the left wall to the right wall.

## Build & Run

```sh
cargo build          # compile
cargo run            # run the game
cargo clippy         # lint
cargo fmt --check    # check formatting
```

No test suite exists. The project has no CI configuration.

## Architecture

Bevy ECS architecture — all game logic lives in systems that run each frame.

### Coordinate System

The board is a fixed grid defined by constants in `src/constants.rs`:

- **Base board**: 10 columns × 22 rows (classic Tetris dimensions)
- **Grain board**: 80 columns × 176 rows (`PIECE_SUBDIVISION = 8` multiplier)
- **Window**: sized to iPhone 14 (390×844) plus `PREVIEW_AREA_HEIGHT = 100px` at the bottom for the piece-selection UI; `GRAIN_SIZE` is derived from `IPHONE14_HEIGHT / BASE_BOARD_HEIGHT / PIECE_SUBDIVISION`

`BoardGrid` (in `src/resources.rs`) maps between world coordinates and grid `(col, row)` where row 0 is the bottom (floor). It owns a 2D array of `Option<Entity>` tracking every settled grain. `FLOOR_Y` is offset upward so the bottom 100px of the window is free for the UI.

### System Pipeline

Startup systems: `setup_scene`, `setup_ui`, `init_queue_system` (fills `NextPieceQueue` with `NUM_CANDIDATES = 3` random pieces).

All Update systems run in a strict chain (order matters):

1. **preview_interaction_system** — handles clicks on the candidate piece slots; spawns the chosen `TetrominoShape` as `Grain` entities tagged `ActivePiece` and replenishes the queue; triggers game-over if the spawn area is blocked
2. **input_system** — moves the active piece horizontally to follow the mouse cursor; Space hard-drops and locks grains into `BoardGrid`, removing `ActivePiece`
3. **ghost_system** — despawns and re-spawns `GhostGrain` entities each frame to show where the piece would land
4. **clear_system** — gates on `BoardDirty`; BFS over settled grains per color; if a component spans both walls, inserts `PopOutGrain` with burst physics onto cleared grains and starts `ClearEffect.pending` timer
5. **pop_out_system** — drives projectile motion + pseudo-3D tumble for `PopOutGrain` entities; despawns them when off-screen
6. **sand_physics_system** — ticks `SandTimer`; scans board bottom-to-top, moves unstable settled grains down or diagonally; sets `grain.stable = true` when blocked; paused while `ClearEffect.pending.is_some()`
7. **game_over_check_system** — checks if any settled grain reached the top row
8. **update_score_ui / update_preview_ui / game_over_ui** — UI updates

### Key Resources

- **`BoardGrid`**: 2D `[[Option<Entity>; 80]; 176]` — the authoritative source of grain positions. Must be kept in sync with `Transform` components.
- **`BoardDirty`**: boolean flag set whenever grains settle or sand moves; gates the BFS in `clear_system` to avoid re-scanning an unchanged board.
- **`ClearScratch`**: pre-allocated BFS scratch buffers (stamp-based visited array, queue, component list) to avoid per-frame allocations.
- **`NextPieceQueue`**: `VecDeque` of `(TetrominoShape, GrainColor)` pairs shown in the UI; always maintained at `NUM_CANDIDATES` entries.
- **`ClearEffect`**: holds `Option<PendingClear>` with elapsed/duration; its presence pauses sand physics.

### Key Design Patterns

- **Active vs Settled**: `ActivePiece` marks grains still being positioned. On hard-drop (Space), `ActivePiece` is removed, `grain.settled = true`, and the entity is registered in `BoardGrid`.
- **Stable optimisation**: `grain.stable = true` means the grain cannot move (blocked below and diagonally). Sand physics skips stable grains; neighbors are marked `stable = false` whenever an adjacent grain moves or is cleared.
- **Collision checks exclude siblings**: movement validity checks treat cells occupied by another grain of the same active piece as free.
- **Ghost piece**: `GhostGrain` entities are despawned and fully re-spawned every frame — they are never mutated in place.
- **PopOutGrain animation**: on clear, grains are immediately removed from `BoardGrid` and receive `PopOutGrain` with per-grain ballistic + tumble parameters; `pop_out_system` drives their `Transform` and `Sprite` color (white-flash lerp to base color) until they leave the screen.

### Controls

- **Mouse move**: horizontally positions the active piece (snaps to nearest grain column)
- **Space**: hard drop — instantly locks the piece at the lowest valid position
