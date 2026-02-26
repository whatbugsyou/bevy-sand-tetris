use crate::components::{ActivePiece, Grain, PopOutGrain};
use crate::constants::*;
use crate::resources::{
    BoardDirty, BoardGrid, ClearEffect, ClearScratch, GameStatus, PendingClear,
};
use bevy::prelude::*;
use std::collections::HashSet;

/// Total time (seconds) for the stagger wave to sweep from the leftmost to rightmost column.
const POP_STAGGER_TOTAL: f32 = 0.5;
/// Initial upward velocity (world-space pixels/s) when a grain launches.
const POP_INITIAL_VEL: f32 = 600.0;
/// Downward acceleration (world-space pixels/s²).
const POP_GRAVITY: f32 = 1400.0;
/// Extra time budget after the last grain launches to let it exit the screen.
const POP_FLIGHT_DURATION: f32 = 1.5;

const CLEAR_DIRECTIONS: [(i32, i32); 8] = [
    (0, 1),
    (0, -1),
    (1, 0),
    (-1, 0),
    (1, 1),
    (1, -1),
    (-1, 1),
    (-1, -1),
];

/// For each color, find connected components (8-connectivity) among settled grains.
/// If a component touches both walls, immediately remove those grains from the board,
/// insert `PopOutGrain` with a left→right staggered launch delay, and start the pending timer.
pub fn clear_system(
    mut commands: Commands,
    mut board: ResMut<BoardGrid>,
    mut board_dirty: ResMut<BoardDirty>,
    mut clear_scratch: ResMut<ClearScratch>,
    mut clear_effect: ResMut<ClearEffect>,
    mut game_status: ResMut<GameStatus>,
    mut grain_query: Query<&mut Grain, Without<ActivePiece>>,
    time: Res<Time>,
) {
    // ---- Tick the pending timer and finalize when done ----
    let mut should_finalize = false;
    if let Some(pending) = clear_effect.pending.as_mut() {
        pending.elapsed += time.delta_secs();
        if pending.elapsed >= pending.duration {
            should_finalize = true;
        }
    }

    if should_finalize {
        if let Some(pending) = clear_effect.pending.take() {
            game_status.score += pending.points;
            board_dirty.0 = true; // re-check for newly clearable components
            info!("Clear animation done. +{} pts  total: {}", pending.points, game_status.score);
        }
        return;
    }

    // ---- Gate: skip BFS while animating, game over, or board unchanged ----
    if clear_effect.pending.is_some() || game_status.is_game_over || !board_dirty.0 {
        return;
    }
    board_dirty.0 = false;

    // ---- BFS ----
    let w = BOARD_WIDTH as usize;
    let h = BOARD_HEIGHT as usize;
    let cell_count = w * h;
    if clear_scratch.visited_stamp.len() != cell_count {
        clear_scratch.visited_stamp.resize(cell_count, 0);
    }
    if clear_scratch.current_stamp == u32::MAX {
        clear_scratch.visited_stamp.fill(0);
        clear_scratch.current_stamp = 1;
    } else {
        clear_scratch.current_stamp += 1;
    }
    let stamp = clear_scratch.current_stamp;
    let ClearScratch {
        visited_stamp,
        queue,
        component,
        ..
    } = &mut *clear_scratch;

    let mut targets_to_clear: Vec<(usize, usize, Entity)> = Vec::new();

    for start_row in 0..h {
        for start_col in 0..w {
            let start_idx = start_row * w + start_col;
            if visited_stamp[start_idx] == stamp {
                continue;
            }
            let start_entity = match board.cells[start_row][start_col] {
                Some(entity) => entity,
                None => continue,
            };
            let target_color = match grain_query.get(start_entity) {
                Ok(grain) if grain.settled => grain.color,
                _ => continue,
            };

            let mut touches_left = false;
            let mut touches_right = false;
            visited_stamp[start_idx] = stamp;
            component.clear();
            queue.clear();

            queue.push_back((start_row, start_col));

            while let Some((r, c)) = queue.pop_front() {
                component.push((r, c));
                if c == 0 {
                    touches_left = true;
                }
                if c == w - 1 {
                    touches_right = true;
                }

                for &(dr, dc) in &CLEAR_DIRECTIONS {
                    let nr = r as i32 + dr;
                    let nc = c as i32 + dc;
                    if nr >= 0 && nr < h as i32 && nc >= 0 && nc < w as i32 {
                        let nr = nr as usize;
                        let nc = nc as usize;
                        let nidx = nr * w + nc;
                        if visited_stamp[nidx] == stamp {
                            continue;
                        }
                        let neighbor_entity = match board.cells[nr][nc] {
                            Some(entity) => entity,
                            None => continue,
                        };
                        let same_component = match grain_query.get(neighbor_entity) {
                            Ok(grain) => grain.settled && grain.color == target_color,
                            Err(_) => false,
                        };
                        if same_component {
                            visited_stamp[nidx] = stamp;
                            queue.push_back((nr, nc));
                        }
                    }
                }
            }

            if touches_left && touches_right {
                for &(r, c) in component.iter() {
                    if let Some(entity) = board.cells[r][c] {
                        targets_to_clear.push((c, r, entity));
                    }
                }
            }
        }
    }

    if targets_to_clear.is_empty() {
        return;
    }

    targets_to_clear.sort_unstable_by_key(|&(col, row, _)| (row, col));
    targets_to_clear.dedup_by_key(|entry| (entry.0, entry.1));
    let points = targets_to_clear.len() as u32;

    // ---- Immediately remove from board and mark adjacent grains unstable ----
    let cleared_positions: HashSet<(usize, usize)> = targets_to_clear
        .iter()
        .map(|&(col, row, _)| (col, row))
        .collect();

    for &(col, row, _) in &targets_to_clear {
        board.clear_cell(col, row);
    }
    mark_boundary_unstable(&board, &mut grain_query, &cleared_positions);

    // ---- Insert PopOutGrain with staggered launch delays (left → right) ----
    let col_max = (BOARD_WIDTH - 1) as f32;
    for &(col, row, entity) in &targets_to_clear {
        let (origin_x, origin_y) = BoardGrid::grid_to_world(col, row);
        let launch_delay = (col as f32 / col_max) * POP_STAGGER_TOTAL;
        commands.entity(entity).insert(PopOutGrain {
            launch_delay,
            elapsed: 0.0,
            origin_x,
            origin_y,
        });
    }

    clear_effect.pending = Some(PendingClear {
        points,
        elapsed: 0.0,
        duration: POP_STAGGER_TOTAL + POP_FLIGHT_DURATION,
    });
    info!("Clearing {} grains (pop-out animation started)", points);
}

/// Drive projectile motion for each `PopOutGrain` and despawn it once it exits the screen.
pub fn pop_out_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut PopOutGrain)>,
) {
    let dt = time.delta_secs();
    // Despawn threshold: well below the visible board
    let despawn_y = FLOOR_Y - 200.0;

    for (entity, mut transform, mut pop) in query.iter_mut() {
        pop.elapsed += dt;
        let t = pop.elapsed - pop.launch_delay;
        if t <= 0.0 {
            continue; // not launched yet — stay in original position
        }
        // Projectile: y = origin_y + v0*t - 0.5*g*t²
        let y = pop.origin_y + POP_INITIAL_VEL * t - 0.5 * POP_GRAVITY * t * t;
        transform.translation.x = pop.origin_x;
        transform.translation.y = y;

        if y < despawn_y {
            commands.entity(entity).despawn();
        }
    }
}

/// Mark grains adjacent to the cleared region boundary as unstable.
fn mark_boundary_unstable(
    board: &BoardGrid,
    grain_query: &mut Query<&mut Grain, Without<ActivePiece>>,
    cleared_positions: &HashSet<(usize, usize)>,
) {
    for &(col, row) in cleared_positions {
        for dr in -1..=1 {
            for dc in -1..=1 {
                if dr == 0 && dc == 0 {
                    continue;
                }

                let nc = col as i32 + dc;
                let nr = row as i32 + dr;
                if nc < 0 || nc >= BOARD_WIDTH || nr < 0 || nr >= BOARD_HEIGHT {
                    continue;
                }

                let nc_usize = nc as usize;
                let nr_usize = nr as usize;
                if cleared_positions.contains(&(nc_usize, nr_usize)) {
                    continue;
                }

                if let Some(entity) = board.cells[nr_usize][nc_usize] {
                    if let Ok(mut grain) = grain_query.get_mut(entity) {
                        if grain.settled {
                            grain.stable = false;
                        }
                    }
                }
            }
        }
    }
}
