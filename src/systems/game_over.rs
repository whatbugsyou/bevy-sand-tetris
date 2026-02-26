use crate::constants::*;
use crate::resources::{BoardGrid, ClearEffect, GameStatus};
use bevy::prelude::*;

pub fn game_over_check_system(
    mut game_status: ResMut<GameStatus>,
    board: Res<BoardGrid>,
    clear_effect: Res<ClearEffect>,
) {
    if game_status.is_game_over || clear_effect.pending.is_some() {
        return;
    }

    // Check top row (BOARD_HEIGHT - 1) for any settled grain
    let top_row = (BOARD_HEIGHT - 1) as usize;
    let reached_top = board.cells[top_row].iter().any(|cell| cell.is_some());

    if reached_top {
        game_status.is_game_over = true;
        info!("Game Over! Final score: {}", game_status.score);
    }
}
