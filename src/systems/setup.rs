use bevy::prelude::*;
use crate::constants::*;

pub fn setup_scene(mut commands: Commands) {
    commands.spawn(Camera2d);

    let board_width = BOARD_WIDTH as f32 * GRAIN_SIZE;
    let board_height = BOARD_HEIGHT as f32 * GRAIN_SIZE;
    let half_w = board_width * 0.5;

    // Floor
    commands.spawn((
        Sprite::from_color(Color::srgb(0.35, 0.35, 0.40), Vec2::new(board_width + GRAIN_SIZE, 4.0)),
        Transform::from_xyz(0.0, FLOOR_Y - GRAIN_SIZE * 0.5, -1.0),
    ));

    // Left wall
    commands.spawn((
        Sprite::from_color(Color::srgb(0.30, 0.30, 0.35), Vec2::new(4.0, board_height + GRAIN_SIZE)),
        Transform::from_xyz(-half_w - GRAIN_SIZE * 0.5, FLOOR_Y + board_height * 0.5, -1.0),
    ));

    // Right wall
    commands.spawn((
        Sprite::from_color(Color::srgb(0.30, 0.30, 0.35), Vec2::new(4.0, board_height + GRAIN_SIZE)),
        Transform::from_xyz(half_w + GRAIN_SIZE * 0.5, FLOOR_Y + board_height * 0.5, -1.0),
    ));
}
