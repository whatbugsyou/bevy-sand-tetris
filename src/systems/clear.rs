use crate::components::{ActivePiece, ClearingGrain, Grain};
use crate::constants::*;
use crate::resources::{
    BoardDirty, BoardGrid, ClearEffect, ClearScratch, GameStatus, PendingClear,
};
use bevy::prelude::*;
use std::collections::HashSet;

const CLEAR_FLASH_DURATION: f32 = 0.6;
const CLEAR_FLASH_INTERVAL: f32 = 0.2;
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

/// For each color, find connected components (8-connectivity, including diagonals) among settled grains.
/// If a component touches both the left wall (col 0) and right wall (col BOARD_WIDTH-1),
/// mark all grains in that component for white flashing, then eliminate after a short delay.
pub fn clear_system(
    mut commands: Commands,
    mut board: ResMut<BoardGrid>,
    mut board_dirty: ResMut<BoardDirty>,
    mut clear_scratch: ResMut<ClearScratch>,
    mut clear_effect: ResMut<ClearEffect>,
    mut game_status: ResMut<GameStatus>,
    mut grain_query: Query<&mut Grain, Without<ActivePiece>>,
    mut sprite_query: Query<&mut Sprite>,
    time: Res<Time>,
) {
    let mut should_finalize = false;
    if let Some(pending) = clear_effect.pending.as_mut() {
        pending.elapsed += time.delta_secs();
        let flash_on = ((pending.elapsed / pending.flash_interval).floor() as i32) % 2 == 0;
        for &(_, _, entity) in &pending.targets {
            let grain_color = grain_query
                .get(entity)
                .map(|g| g.color.to_bevy_color())
                .unwrap_or(Color::WHITE);
            if let Ok(mut sprite) = sprite_query.get_mut(entity) {
                sprite.color = if flash_on { Color::WHITE } else { grain_color };
            }
        }
        if pending.elapsed >= pending.duration {
            should_finalize = true;
        }
    }

    if should_finalize {
        if let Some(pending) = clear_effect.pending.take() {
            let cleared_positions: HashSet<(usize, usize)> = pending
                .targets
                .iter()
                .map(|&(col, row, _)| (col, row))
                .collect();
            mark_boundary_unstable(&board, &mut grain_query, &cleared_positions);

            for (col, row, entity) in pending.targets {
                board.clear_cell(col, row);
                commands.entity(entity).despawn();
            }
            board_dirty.0 = true;
            game_status.score += pending.points;
            info!(
                "Cleared {} grains after flash. Score: {}",
                pending.points, game_status.score
            );
        }
        return;
    }

    if clear_effect.pending.is_some() || game_status.is_game_over || !board_dirty.0 {
        return;
    }
    board_dirty.0 = false;

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

    for &(_, _, entity) in &targets_to_clear {
        commands.entity(entity).insert(ClearingGrain);
        if let Ok(mut sprite) = sprite_query.get_mut(entity) {
            sprite.color = Color::WHITE;
        }
    }

    clear_effect.pending = Some(PendingClear {
        targets: targets_to_clear,
        points,
        elapsed: 0.0,
        duration: CLEAR_FLASH_DURATION,
        flash_interval: CLEAR_FLASH_INTERVAL,
    });
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
