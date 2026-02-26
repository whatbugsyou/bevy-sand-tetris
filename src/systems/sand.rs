use crate::components::{ActivePiece, ClearingGrain, Grain};
use crate::constants::*;
use crate::resources::{BoardDirty, BoardGrid, ClearEffect, GameStatus, SandTimer};
use bevy::prelude::*;
use rand::RngExt;

/// Simulate sand physics for settled grains: fall down or slide diagonally.
/// Scan from bottom to top for stability.
pub fn sand_physics_system(
    mut board: ResMut<BoardGrid>,
    mut grain_query: Query<
        (&mut Transform, &mut Grain),
        (Without<ActivePiece>, Without<ClearingGrain>),
    >,
    mut board_dirty: ResMut<BoardDirty>,
    mut sand_timer: ResMut<SandTimer>,
    time: Res<Time>,
    clear_effect: Res<ClearEffect>,
    game_status: Res<GameStatus>,
) {
    if game_status.is_game_over || clear_effect.pending.is_some() {
        return;
    }
    if !sand_timer.0.tick(time.delta()).just_finished() {
        return;
    }

    let mut rng = rand::rng();
    let mut moved_any = false;

    // Process rows from bottom (row 0) to top
    for row in 0..(BOARD_HEIGHT as usize) {
        for col in 0..(BOARD_WIDTH as usize) {
            let entity = match board.cells[row][col] {
                Some(e) => e,
                None => continue,
            };

            // Only process settled, unstable grains
            if let Ok((_, grain)) = grain_query.get(entity) {
                if !grain.settled || grain.stable {
                    continue;
                }
            } else {
                continue;
            }

            let r = row as i32;
            let c = col as i32;

            // Try to fall straight down
            if r > 0 && board.is_free(c, r - 1) {
                move_grain(&mut board, &mut grain_query, entity, col, row, col, row - 1);
                moved_any = true;
                // Mark neighbors above as potentially unstable
                mark_neighbors_unstable(&board, &mut grain_query, col, row);
                continue;
            }

            // Try diagonal: randomly pick left-down or right-down first
            if r > 0 {
                let (first_dc, second_dc) = if rng.random_bool(0.5) {
                    (-1i32, 1i32)
                } else {
                    (1, -1)
                };

                if board.is_free(c + first_dc, r - 1) {
                    move_grain(
                        &mut board,
                        &mut grain_query,
                        entity,
                        col,
                        row,
                        (c + first_dc) as usize,
                        row - 1,
                    );
                    moved_any = true;
                    mark_neighbors_unstable(&board, &mut grain_query, col, row);
                    continue;
                }
                if board.is_free(c + second_dc, r - 1) {
                    move_grain(
                        &mut board,
                        &mut grain_query,
                        entity,
                        col,
                        row,
                        (c + second_dc) as usize,
                        row - 1,
                    );
                    moved_any = true;
                    mark_neighbors_unstable(&board, &mut grain_query, col, row);
                    continue;
                }
            }

            // If we reach here, grain cannot move - mark as stable
            if let Ok((_, mut grain)) = grain_query.get_mut(entity) {
                grain.stable = true;
            }
        }
    }
    if moved_any {
        board_dirty.0 = true;
    }
}

fn move_grain(
    board: &mut BoardGrid,
    grain_query: &mut Query<
        (&mut Transform, &mut Grain),
        (Without<ActivePiece>, Without<ClearingGrain>),
    >,
    entity: Entity,
    from_col: usize,
    from_row: usize,
    to_col: usize,
    to_row: usize,
) {
    board.clear_cell(from_col, from_row);
    board.set(to_col, to_row, entity);
    if let Ok((mut transform, _)) = grain_query.get_mut(entity) {
        let (wx, wy) = BoardGrid::grid_to_world(to_col, to_row);
        transform.translation.x = wx;
        transform.translation.y = wy;
    }
}

/// Mark grains in the surrounding area as unstable when a grain moves
fn mark_neighbors_unstable(
    board: &BoardGrid,
    grain_query: &mut Query<
        (&mut Transform, &mut Grain),
        (Without<ActivePiece>, Without<ClearingGrain>),
    >,
    col: usize,
    row: usize,
) {
    // Check grains above and diagonally above
    let check_positions = [
        (col as i32, row as i32 + 1),     // directly above
        (col as i32 - 1, row as i32 + 1), // upper-left
        (col as i32 + 1, row as i32 + 1), // upper-right
    ];

    for (c, r) in check_positions {
        if c >= 0 && c < BOARD_WIDTH && r >= 0 && r < BOARD_HEIGHT {
            if let Some(entity) = board.cells[r as usize][c as usize] {
                if let Ok((_, mut grain)) = grain_query.get_mut(entity) {
                    if grain.settled {
                        grain.stable = false;
                    }
                }
            }
        }
    }
}
