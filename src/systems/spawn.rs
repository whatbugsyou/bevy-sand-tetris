use crate::constants::*;
use crate::resources::NextPieceQueue;
use crate::types::{GrainColor, TetrominoShape};
use bevy::prelude::*;

/// Convert a grain column index to world-space X.
pub fn col_to_world_x(col: i32) -> f32 {
    let left_col_x = -((BOARD_WIDTH - 1) as f32) * 0.5 * GRAIN_SIZE;
    left_col_x + col as f32 * GRAIN_SIZE
}

/// Startup system: fill the candidate queue to NUM_CANDIDATES pieces.
pub fn init_queue_system(mut piece_queue: ResMut<NextPieceQueue>) {
    let mut rng = rand::rng();
    while piece_queue.pieces.len() < NUM_CANDIDATES {
        piece_queue.pieces.push_back((
            TetrominoShape::random(&mut rng),
            GrainColor::random(&mut rng),
        ));
    }
}
