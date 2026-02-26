use bevy::prelude::*;
use crate::components::{ActivePiece, Grain};
use crate::constants::*;
use crate::resources::{BoardGrid, FallTimer, GameStatus};
const HORIZONTAL_HOLD_INITIAL_DELAY: f32 = 0.16;
const HORIZONTAL_HOLD_REPEAT_INTERVAL: f32 = 0.05;

#[derive(Default)]
pub(crate) struct HorizontalHoldState {
    direction: i32,
    next_move_time: f64,
}

/// Move the active piece left/right, rotate, soft-drop, or hard-drop.
pub fn input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut active_query: Query<(Entity, &mut Transform, &mut Grain), With<ActivePiece>>,
    mut board: ResMut<BoardGrid>,
    mut fall_timer: ResMut<FallTimer>,
    time: Res<Time>,
    mut horizontal_hold: Local<HorizontalHoldState>,
    mut commands: Commands,
    mut game_status: ResMut<GameStatus>,
) {
    if game_status.is_game_over || active_query.is_empty() {
        return;
    }

    // --- Horizontal movement ---
    let left_pressed = keyboard.pressed(KeyCode::ArrowLeft);
    let right_pressed = keyboard.pressed(KeyCode::ArrowRight);
    let desired_dir: i32 = if keyboard.just_pressed(KeyCode::ArrowLeft) {
        -1
    } else if keyboard.just_pressed(KeyCode::ArrowRight) {
        1
    } else if left_pressed && !right_pressed {
        -1
    } else if right_pressed && !left_pressed {
        1
    } else {
        0
    };
    if desired_dir == 0 {
        horizontal_hold.direction = 0;
    } else {
        let now = time.elapsed_secs_f64();
        let should_move_now = if desired_dir != horizontal_hold.direction {
            horizontal_hold.direction = desired_dir;
            horizontal_hold.next_move_time = now + HORIZONTAL_HOLD_INITIAL_DELAY as f64;
            true
        } else if now >= horizontal_hold.next_move_time {
            horizontal_hold.next_move_time = now + HORIZONTAL_HOLD_REPEAT_INTERVAL as f64;
            true
        } else {
            false
        };

        if should_move_now {
            let pieces: Vec<(Entity, i32, i32)> = active_query
                .iter()
                .map(|(e, t, _)| {
                    let (col, row) =
                        BoardGrid::world_to_grid_unclamped(t.translation.x, t.translation.y);
                    (e, col, row)
                })
                .collect();
            let can_move = pieces.iter().all(|&(_, col, row)| {
                let nc = col + desired_dir;
                board.is_free(nc, row)
                    || pieces.iter().any(|&(_, c2, r2)| c2 == nc && r2 == row)
            });

            if can_move {
                for (_, mut transform, _) in &mut active_query {
                    transform.translation.x += desired_dir as f32 * GRAIN_SIZE;
                }
            }
        }
    }

    // --- Rotation (Up arrow or Z key) ---
    if keyboard.just_pressed(KeyCode::ArrowUp) || keyboard.just_pressed(KeyCode::KeyZ) {
        let pieces: Vec<(Entity, i32, i32)> = active_query
            .iter()
            .map(|(e, t, _)| {
                let (col, row) = BoardGrid::world_to_grid_unclamped(t.translation.x, t.translation.y);
                (e, col, row)
            })
            .collect();
        if !pieces.is_empty() {
            // Pivot = center of mass (rounded)
            let count = pieces.len() as f32;
            let cx: f32 = pieces.iter().map(|p| p.1 as f32).sum::<f32>() / count;
            let cy: f32 = pieces.iter().map(|p| p.2 as f32).sum::<f32>() / count;
            let pcol = cx.round() as i32;
            let prow = cy.round() as i32;

            // 90° clockwise: (dc, dr) -> (dr, -dc)
            let rotated: Vec<(Entity, i32, i32)> = pieces
                .iter()
                .map(|&(e, col, row)| {
                    let dc = col - pcol;
                    let dr = row - prow;
                    (e, pcol + dr, prow - dc)
                })
                .collect();

            let can_rotate = rotated.iter().all(|&(_, nc, nr)| {
                board.is_free(nc, nr)
                    || pieces.iter().any(|&(_, c2, r2)| c2 == nc && r2 == nr)
            });

            if can_rotate {
                for (entity, mut transform, _) in &mut active_query {
                    if let Some(&(_, nc, nr)) = rotated.iter().find(|&&(e, _, _)| e == entity) {
                        let (wx, wy) = BoardGrid::grid_to_world_i32(nc, nr);
                        transform.translation.x = wx;
                        transform.translation.y = wy;
                    }
                }
            }
        }
    }

    // --- Soft drop (Down arrow held) ---
    if keyboard.pressed(KeyCode::ArrowDown) {
        fall_timer.0.set_duration(std::time::Duration::from_secs_f32(0.05));
    } else {
        fall_timer.0.set_duration(std::time::Duration::from_secs_f32(FALL_INTERVAL));
    }

    // --- Hard drop (Space) ---
    if keyboard.just_pressed(KeyCode::Space) {
        // Re-collect positions (may have changed from horizontal/rotation above)
        let pieces: Vec<(Entity, i32, i32)> = active_query
            .iter()
            .map(|(e, t, _)| {
                let (col, row) = BoardGrid::world_to_grid_unclamped(t.translation.x, t.translation.y);
                (e, col, row)
            })
            .collect();

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
        for (entity, mut transform, mut grain) in &mut active_query {
            transform.translation.y -= drop_rows as f32 * GRAIN_SIZE;
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
            let (wx, wy) = BoardGrid::grid_to_world_i32(col, row);
            transform.translation.x = wx;
            transform.translation.y = wy;
            grain.settled = true;
            board.set(col as usize, row as usize, entity);
            commands.entity(entity).remove::<ActivePiece>();
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
