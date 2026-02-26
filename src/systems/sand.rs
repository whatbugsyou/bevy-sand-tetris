use bevy::prelude::*;
use rand::RngExt;
use crate::components::{ActivePiece, ClearingGrain, Grain};
use crate::constants::*;
use crate::resources::{BoardGrid, ClearEffect, GameStatus, SandTimer};

/// Simulate sand physics for settled grains: fall down or slide diagonally.
/// Scan from bottom to top for stability.
pub fn sand_physics_system(
    mut board: ResMut<BoardGrid>,
    mut grain_query: Query<(&mut Transform, &Grain), (Without<ActivePiece>, Without<ClearingGrain>)>,
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

    // Process rows from bottom (row 0) to top
    for row in 0..(BOARD_HEIGHT as usize) {
        for col in 0..(BOARD_WIDTH as usize) {
            let entity = match board.cells[row][col] {
                Some(e) => e,
                None => continue,
            };

            // Only process settled grains
            if let Ok((_, grain)) = grain_query.get(entity) {
                if !grain.settled {
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
                    continue;
                }
            }
        }
    }
}

fn move_grain(
    board: &mut BoardGrid,
    grain_query: &mut Query<(&mut Transform, &Grain), (Without<ActivePiece>, Without<ClearingGrain>)>,
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
