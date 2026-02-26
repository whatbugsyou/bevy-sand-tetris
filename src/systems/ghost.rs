use crate::components::{ActivePiece, Grain, GhostGrain};
use crate::constants::*;
use crate::resources::BoardGrid;
use bevy::prelude::*;

/// Renders a dim ghost preview of where the active piece would land on hard drop.
/// Ghost grains are despawned and re-spawned every frame to stay in sync with piece movement.
pub fn ghost_system(
    mut commands: Commands,
    ghost_query: Query<Entity, With<GhostGrain>>,
    active_query: Query<(&Transform, &Grain), With<ActivePiece>>,
    board: Res<BoardGrid>,
) {
    // Always despawn stale ghosts first
    for entity in &ghost_query {
        commands.entity(entity).despawn();
    }

    if active_query.is_empty() {
        return;
    }

    // Collect current active piece positions and colors
    let pieces: Vec<(i32, i32, Color)> = active_query
        .iter()
        .map(|(t, g)| {
            let (col, row) =
                BoardGrid::world_to_grid_unclamped(t.translation.x, t.translation.y);
            (col, row, g.color.to_bevy_color())
        })
        .collect();

    // Compute hard-drop distance (same logic as Space in input.rs)
    let mut drop_rows = 0i32;
    loop {
        let next_drop = drop_rows + 1;
        let can_drop = pieces.iter().all(|&(col, row, _)| {
            let nr = row - next_drop;
            nr >= 0
                && (board.is_free(col, nr)
                    || pieces.iter().any(|&(c2, r2, _)| c2 == col && r2 == nr))
        });
        if can_drop {
            drop_rows = next_drop;
        } else {
            break;
        }
    }

    // Spawn ghost grains at drop destination
    for &(col, row, color) in &pieces {
        let ghost_row = row - drop_rows;
        if ghost_row < 0 || ghost_row >= BOARD_HEIGHT {
            continue;
        }
        let (wx, wy) = BoardGrid::grid_to_world_i32(col, ghost_row);

        // Dim the color to ~25% brightness to visually distinguish ghost from active piece
        let ghost_color = dim_color(color);

        commands.spawn((
            GhostGrain,
            Sprite::from_color(ghost_color, Vec2::splat(GRAIN_SIZE)),
            Transform::from_xyz(wx, wy, -0.5),
            GlobalTransform::default(),
        ));
    }
}

/// Returns a darkened version of the given color (25% of original brightness).
fn dim_color(color: Color) -> Color {
    let factor = 0.25;
    let s = color.to_srgba();
    Color::srgba(s.red * factor, s.green * factor, s.blue * factor, s.alpha)
}
