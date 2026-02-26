use crate::constants::*;
use crate::types::{GrainColor, TetrominoShape};
use bevy::prelude::*;
use std::collections::VecDeque;

#[derive(Resource, Default)]
pub struct NextPieceQueue {
    pub pieces: VecDeque<(TetrominoShape, GrainColor)>,
}

#[derive(Resource, Default)]
pub struct GameStatus {
    pub score: u32,
    pub is_game_over: bool,
}

#[derive(Resource)]
pub struct SpawnClock(pub Timer);

#[derive(Resource)]
pub struct FallTimer(pub Timer);

#[derive(Resource)]
pub struct SandTimer(pub Timer);

#[derive(Resource, Default)]
pub struct ClearEffect {
    pub pending: Option<PendingClear>,
}
#[derive(Resource, Default)]
pub struct BoardDirty(pub bool);

#[derive(Resource)]
pub struct ClearScratch {
    pub visited_stamp: Vec<u32>,
    pub current_stamp: u32,
    pub queue: VecDeque<(usize, usize)>,
    pub component: Vec<(usize, usize)>,
}

impl Default for ClearScratch {
    fn default() -> Self {
        let cell_count = (BOARD_WIDTH as usize) * (BOARD_HEIGHT as usize);
        Self {
            visited_stamp: vec![0; cell_count],
            current_stamp: 1,
            queue: VecDeque::new(),
            component: Vec::new(),
        }
    }
}

pub struct PendingClear {
    pub points: u32,
    pub elapsed: f32,
    pub duration: f32,
}

/// Grid-based board tracking settled grains.
/// Row 0 = bottom (FLOOR_Y), col 0 = left edge.
#[derive(Resource)]
pub struct BoardGrid {
    pub cells: [[Option<Entity>; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize],
}

impl Default for BoardGrid {
    fn default() -> Self {
        Self {
            cells: [[None; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize],
        }
    }
}

impl BoardGrid {
    fn left_x() -> f32 {
        -((BOARD_WIDTH - 1) as f32) * 0.5 * GRAIN_SIZE
    }
    /// Convert world coordinates to grid (col, row). Returns None if out of bounds.
    #[allow(dead_code)]
    pub fn world_to_grid(x: f32, y: f32) -> Option<(usize, usize)> {
        let col = ((x - Self::left_x()) / GRAIN_SIZE).round() as i32;
        let row = ((y - FLOOR_Y) / GRAIN_SIZE).round() as i32;
        if col >= 0 && col < BOARD_WIDTH && row >= 0 && row < BOARD_HEIGHT {
            Some((col as usize, row as usize))
        } else {
            None
        }
    }

    /// Convert world coordinates to grid (col, row) without bounds clamping.
    /// Used for active pieces that may be above the board.
    pub fn world_to_grid_unclamped(x: f32, y: f32) -> (i32, i32) {
        let col = ((x - Self::left_x()) / GRAIN_SIZE).round() as i32;
        let row = ((y - FLOOR_Y) / GRAIN_SIZE).round() as i32;
        (col, row)
    }

    /// Convert grid (col, row) to world (x, y).
    pub fn grid_to_world(col: usize, row: usize) -> (f32, f32) {
        Self::grid_to_world_i32(col as i32, row as i32)
    }

    /// Convert unclamped grid (col, row) to world (x, y).
    pub fn grid_to_world_i32(col: i32, row: i32) -> (f32, f32) {
        let x = Self::left_x() + col as f32 * GRAIN_SIZE;
        let y = FLOOR_Y + row as f32 * GRAIN_SIZE;
        (x, y)
    }

    pub fn is_free(&self, col: i32, row: i32) -> bool {
        if col < 0 || col >= BOARD_WIDTH || row < 0 {
            return false;
        }
        if row >= BOARD_HEIGHT {
            return true; // above board is free
        }
        self.cells[row as usize][col as usize].is_none()
    }

    pub fn set(&mut self, col: usize, row: usize, entity: Entity) {
        self.cells[row][col] = Some(entity);
    }

    pub fn clear_cell(&mut self, col: usize, row: usize) {
        self.cells[row][col] = None;
    }
}
