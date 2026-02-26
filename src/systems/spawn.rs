use crate::components::{ActivePiece, Grain};
use crate::constants::*;
use crate::resources::{BoardGrid, GameStatus, SpawnClock};
use crate::types::{GrainColor, TetrominoShape};
use bevy::prelude::*;
use rand::RngExt;

pub fn col_to_world_x(col: i32) -> f32 {
    let left_col_x = -((BOARD_WIDTH - 1) as f32) * 0.5 * GRAIN_SIZE;
    left_col_x + col as f32 * GRAIN_SIZE
}

pub fn spawn_piece_system(
    mut commands: Commands,
    mut spawn_clock: ResMut<SpawnClock>,
    mut game_status: ResMut<GameStatus>,
    board: Res<BoardGrid>,
    time: Res<Time>,
    active_query: Query<(), With<ActivePiece>>,
) {
    if game_status.is_game_over || !active_query.is_empty() {
        return;
    }

    if !spawn_clock.0.tick(time.delta()).just_finished() {
        return;
    }

    let mut rng = rand::rng();
    let shape = TetrominoShape::random(&mut rng);
    let color = GrainColor::random(&mut rng);
    let offsets = shape.offsets();
    let min_offset_x = offsets.iter().map(|o| o.x).min().unwrap_or(0);
    let max_offset_x = offsets.iter().map(|o| o.x).max().unwrap_or(0);
    let min_anchor_col = -min_offset_x;
    let max_anchor_col = (BOARD_WIDTH - 1) - max_offset_x;
    let anchor_col = rng.random_range(min_anchor_col..(max_anchor_col + 1));
    let spawn_blocked = offsets.iter().any(|offset| {
        let col = anchor_col + offset.x;
        let x = col_to_world_x(col);
        let y = SPAWN_Y + offset.y as f32 * GRAIN_SIZE;
        let (_, row) = BoardGrid::world_to_grid_unclamped(x, y);
        !board.is_free(col, row)
    });

    if spawn_blocked {
        game_status.is_game_over = true;
        info!(
            "Game Over! Spawn area blocked. Final score: {}",
            game_status.score
        );
        return;
    }

    for offset in offsets {
        let col = anchor_col + offset.x;
        let x = col_to_world_x(col);
        let y = SPAWN_Y + offset.y as f32 * GRAIN_SIZE;
        commands.spawn((
            ActivePiece,
            Grain {
                color,
                settled: false,
                stable: false,
            },
            Sprite::from_color(color.to_bevy_color(), Vec2::splat(GRAIN_SIZE)),
            Transform::from_xyz(x, y, 0.0),
            GlobalTransform::default(),
        ));
    }

    info!("Spawned shape {:?} with color {:?}", shape, color);
}
