use crate::components::{ActivePiece, Grain};
use crate::constants::*;
use crate::resources::{BoardDirty, BoardGrid, GameStatus, NextPieceQueue};
use crate::types::{GrainColor, TetrominoShape};
use bevy::prelude::*;

/// Move the active piece horizontally to follow the mouse cursor.
/// Press Space to hard-drop (confirm placement).
pub fn input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut active_query: Query<(Entity, &mut Transform, &mut Grain, &ActivePiece)>,
    mut board: ResMut<BoardGrid>,
    mut board_dirty: ResMut<BoardDirty>,
    mut commands: Commands,
    mut game_status: ResMut<GameStatus>,
    mut piece_queue: ResMut<NextPieceQueue>,
) {
    if game_status.is_game_over || active_query.is_empty() {
        return;
    }

    // --- Mouse-based horizontal positioning ---
    if let Ok(window) = windows.single() {
        if let Some(cursor_pos) = window.cursor_position() {
            if let Ok((camera, camera_transform)) = camera_query.single() {
                if let Ok(world_pos) =
                    camera.viewport_to_world_2d(camera_transform, cursor_pos)
                {
                    let (target_col, _) =
                        BoardGrid::world_to_grid_unclamped(world_pos.x, 0.0);

                    let pieces: Vec<(Entity, i32, i32)> = active_query
                        .iter()
                        .map(|(e, t, _, _)| {
                            let (col, row) = BoardGrid::world_to_grid_unclamped(
                                t.translation.x,
                                t.translation.y,
                            );
                            (e, col, row)
                        })
                        .collect();

                    // Use center-of-mass column as reference
                    let count = pieces.len() as f32;
                    let current_center = (pieces
                        .iter()
                        .map(|(_, c, _)| *c as f32)
                        .sum::<f32>()
                        / count)
                        .round() as i32;
                    let mut delta = target_col - current_center;

                    if delta != 0 {
                        // Clamp so no grain leaves board bounds
                        let min_col =
                            pieces.iter().map(|(_, c, _)| *c).min().unwrap_or(0);
                        let max_col =
                            pieces.iter().map(|(_, c, _)| *c).max().unwrap_or(0);
                        let min_delta = -min_col;
                        let max_delta = (BOARD_WIDTH - 1) - max_col;
                        delta = delta.clamp(min_delta, max_delta);

                        if delta != 0 {
                            let can_move = pieces.iter().all(|&(_, col, row)| {
                                let nc = col + delta;
                                board.is_free(nc, row)
                                    || pieces
                                        .iter()
                                        .any(|&(_, c2, r2)| c2 == nc && r2 == row)
                            });

                            if can_move {
                                for (_, mut transform, _, _) in &mut active_query {
                                    transform.translation.x += delta as f32 * GRAIN_SIZE;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // --- Space: hard drop (confirm placement) ---
    if keyboard.just_pressed(KeyCode::Space) {
        let pieces: Vec<(Entity, i32, i32)> = active_query
            .iter()
            .map(|(e, t, _, _)| {
                let (col, row) =
                    BoardGrid::world_to_grid_unclamped(t.translation.x, t.translation.y);
                (e, col, row)
            })
            .collect();
        // All grains share the same slot; read it once before the mutable loop.
        let active_slot = active_query.iter().next().map(|(_, _, _, ap)| ap.slot);

        // Find max drop distance
        let mut drop_rows = 0i32;
        loop {
            let next_drop = drop_rows + 1;
            let can_drop = pieces.iter().all(|&(_, col, row)| {
                let nr = row - next_drop;
                if nr < 0 {
                    return false;
                }
                board.is_free(col, nr)
                    || pieces.iter().any(|&(_, c2, r2)| c2 == col && r2 == nr)
            });
            if can_drop {
                drop_rows = next_drop;
            } else {
                break;
            }
        }

        // Apply drop and lock
        let mut overflowed_top = false;
        let mut locked_any = false;
        for (entity, mut transform, mut grain, _) in &mut active_query {
            transform.translation.y -= drop_rows as f32 * GRAIN_SIZE;
            let (col, row) = BoardGrid::world_to_grid_unclamped(
                transform.translation.x,
                transform.translation.y,
            );
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
            let (wx, wy) = BoardGrid::grid_to_world_i32(col, row);
            transform.translation.x = wx;
            transform.translation.y = wy;
            grain.settled = true;
            grain.stable = false;
            board.set(col as usize, row as usize, entity);
            locked_any = true;
            commands.entity(entity).remove::<ActivePiece>();
        }
        if locked_any {
            board_dirty.0 = true;
            // Piece was placed: now consume its queue slot and replenish.
            if let Some(slot) = active_slot {
                if slot < piece_queue.pieces.len() {
                    piece_queue.pieces.remove(slot);
                    let mut rng = rand::rng();
                    piece_queue.pieces.push_back((
                        TetrominoShape::random(&mut rng),
                        GrainColor::random(&mut rng),
                    ));
                }
            }
        }
        if overflowed_top {
            game_status.is_game_over = true;
            info!(
                "Game Over! Piece hard-dropped while above top. Final score: {}",
                game_status.score
            );
        }
    }
}
