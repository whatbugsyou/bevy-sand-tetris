use bevy::prelude::*;
use crate::components::{ActivePiece, Grain};
use crate::constants::*;
use crate::resources::{BoardGrid, FallTimer, GameStatus};

/// Active piece falls as a group, one grid row per tick.
pub fn falling_system(
    mut commands: Commands,
    mut active_query: Query<(Entity, &mut Transform, &mut Grain), With<ActivePiece>>,
    mut board: ResMut<BoardGrid>,
    mut fall_timer: ResMut<FallTimer>,
    time: Res<Time>,
    mut game_status: ResMut<GameStatus>,
) {
    if game_status.is_game_over {
        return;
    }

    if active_query.is_empty() {
        return;
    }

    if !fall_timer.0.tick(time.delta()).just_finished() {
        return;
    }

    // Collect current grid positions of all active grains (may be above board)
    let positions: Vec<(Entity, i32, i32)> = active_query
        .iter()
        .map(|(e, t, _)| {
            let (col, row) = BoardGrid::world_to_grid_unclamped(t.translation.x, t.translation.y);
            (e, col, row)
        })
        .collect();

    // Check if ALL grains can move down one row
    let can_fall = positions.iter().all(|&(_, col, row)| {
        let target_row = row - 1;
        if target_row < 0 {
            return false;
        }
        // OK if the target cell is free or occupied by another active piece grain
        board.is_free(col, target_row)
            || positions.iter().any(|&(_, c2, r2)| c2 == col && r2 == target_row)
    });

    if can_fall {
        // Move all grains down one row
        for (_, mut transform, _) in &mut active_query {
            transform.translation.y -= GRAIN_SIZE;
        }
    } else {
        // Lock: settle all active grains into the board
        let mut overflowed_top = false;
        for (entity, mut transform, mut grain) in &mut active_query {
            let (col, row) =
                BoardGrid::world_to_grid_unclamped(transform.translation.x, transform.translation.y);
            if col < 0 || col >= BOARD_WIDTH || row < 0 {
                overflowed_top = true;
                commands.entity(entity).despawn();
                continue;
            }
            if row >= BOARD_HEIGHT {
                overflowed_top = true;
                commands.entity(entity).despawn();
                continue;
            }

            // Snap to exact grid position
            let (wx, wy) = BoardGrid::grid_to_world_i32(col, row);
            transform.translation.x = wx;
            transform.translation.y = wy;
            grain.settled = true;
            board.set(col as usize, row as usize, entity);
            commands.entity(entity).remove::<ActivePiece>();
        }
        if overflowed_top {
            game_status.is_game_over = true;
            info!("Game Over! Piece locked above top. Final score: {}", game_status.score);
        }
    }
}
