use std::collections::VecDeque;
use bevy::prelude::*;
use crate::components::{ClearingGrain, Grain};
use crate::constants::*;
use crate::resources::{BoardGrid, ClearEffect, GameStatus, PendingClear};
use crate::types::GrainColor;

const CLEAR_FLASH_DURATION: f32 = 0.6;
const CLEAR_FLASH_INTERVAL: f32 = 0.2;

/// For each color, find connected components (8-connectivity, including diagonals) among settled grains.
/// If a component touches both the left wall (col 0) and right wall (col BOARD_WIDTH-1),
/// mark all grains in that component for white flashing, then eliminate after a short delay.
pub fn clear_system(
    mut commands: Commands,
    mut board: ResMut<BoardGrid>,
    mut clear_effect: ResMut<ClearEffect>,
    mut game_status: ResMut<GameStatus>,
    grain_query: Query<&Grain>,
    mut sprite_query: Query<(&Grain, &mut Sprite)>,
    time: Res<Time>,
) {
    let mut should_finalize = false;
    if let Some(pending) = clear_effect.pending.as_mut() {
        pending.elapsed += time.delta_secs();
        let flash_on = ((pending.elapsed / pending.flash_interval).floor() as i32) % 2 == 0;
        for &(_, _, entity) in &pending.targets {
            if let Ok((grain, mut sprite)) = sprite_query.get_mut(entity) {
                sprite.color = if flash_on {
                    Color::WHITE
                } else {
                    grain.color.to_bevy_color()
                };
            }
        }
        if pending.elapsed >= pending.duration {
            should_finalize = true;
        }
    }

    if should_finalize {
        if let Some(pending) = clear_effect.pending.take() {
            for (col, row, entity) in pending.targets {
                board.clear_cell(col, row);
                commands.entity(entity).despawn();
            }
            game_status.score += pending.points;
            info!(
                "Cleared {} grains after flash. Score: {}",
                pending.points, game_status.score
            );
        }
        return;
    }

    if clear_effect.pending.is_some() || game_status.is_game_over {
        return;
    }

    let w = BOARD_WIDTH as usize;
    let h = BOARD_HEIGHT as usize;

    // Build a color grid from the board.
    let mut color_grid: Vec<Vec<Option<GrainColor>>> = vec![vec![None; w]; h];
    for row in 0..h {
        for col in 0..w {
            if let Some(entity) = board.cells[row][col] {
                if let Ok(grain) = grain_query.get(entity) {
                    if grain.settled {
                        color_grid[row][col] = Some(grain.color);
                    }
                }
            }
        }
    }

    // BFS to find connected components (8-neighbor).
    let mut visited = vec![vec![false; w]; h];
    let directions: [(i32, i32); 8] = [
        (0, 1),
        (0, -1),
        (1, 0),
        (-1, 0),
        (1, 1),
        (1, -1),
        (-1, 1),
        (-1, -1),
    ];

    let mut targets_to_clear: Vec<(usize, usize, Entity)> = Vec::new();

    for start_row in 0..h {
        for start_col in 0..w {
            if visited[start_row][start_col] || color_grid[start_row][start_col].is_none() {
                continue;
            }

            let target_color = color_grid[start_row][start_col].unwrap();
            let mut component: Vec<(usize, usize)> = Vec::new();
            let mut queue = VecDeque::new();
            let mut touches_left = false;
            let mut touches_right = false;

            queue.push_back((start_row, start_col));
            visited[start_row][start_col] = true;

            while let Some((r, c)) = queue.pop_front() {
                component.push((r, c));
                if c == 0 {
                    touches_left = true;
                }
                if c == w - 1 {
                    touches_right = true;
                }

                for &(dr, dc) in &directions {
                    let nr = r as i32 + dr;
                    let nc = c as i32 + dc;
                    if nr >= 0 && nr < h as i32 && nc >= 0 && nc < w as i32 {
                        let nr = nr as usize;
                        let nc = nc as usize;
                        if !visited[nr][nc] {
                            if let Some(color) = color_grid[nr][nc] {
                                if color == target_color {
                                    visited[nr][nc] = true;
                                    queue.push_back((nr, nc));
                                }
                            }
                        }
                    }
                }
            }

            if touches_left && touches_right {
                for (r, c) in component {
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
        if let Ok((_, mut sprite)) = sprite_query.get_mut(entity) {
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
