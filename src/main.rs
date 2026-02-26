use bevy::prelude::*;

mod components;
mod constants;
mod resources;
mod systems;
mod types;

use constants::{
    FALL_INTERVAL, IPHONE14_HEIGHT, IPHONE14_WIDTH, PREVIEW_AREA_HEIGHT, SAND_STEP_INTERVAL,
};
use resources::{
    BoardDirty, BoardGrid, ClearEffect, ClearScratch, FallTimer, GameStatus, NextPieceQueue,
    SandTimer, SpawnClock,
};
use systems::{
    clear::{clear_system, pop_out_system},
    game_over::game_over_check_system,
    input::input_system,
    physics::falling_system,
    sand::sand_physics_system,
    setup::setup_scene,
    spawn::spawn_piece_system,
    ui::{game_over_ui, setup_ui, update_preview_ui, update_score_ui},
};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.08)))
        .insert_resource(SpawnClock(Timer::from_seconds(0.3, TimerMode::Repeating)))
        .insert_resource(FallTimer(Timer::from_seconds(
            FALL_INTERVAL,
            TimerMode::Repeating,
        )))
        .insert_resource(SandTimer(Timer::from_seconds(
            SAND_STEP_INTERVAL,
            TimerMode::Repeating,
        )))
        .insert_resource(ClearEffect::default())
        .insert_resource(BoardDirty::default())
        .insert_resource(ClearScratch::default())
        .insert_resource(GameStatus::default())
        .insert_resource(NextPieceQueue::default())
        .insert_resource(BoardGrid::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy Sand Tetris".into(),
                resolution: (
                    IPHONE14_WIDTH as u32,
                    (IPHONE14_HEIGHT + PREVIEW_AREA_HEIGHT) as u32,
                )
                .into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, (setup_scene, setup_ui))
        .add_systems(
            Update,
            (
                spawn_piece_system,
                input_system,
                falling_system,
                clear_system,
                pop_out_system,
                sand_physics_system,
                game_over_check_system,
                update_score_ui,
                update_preview_ui,
                game_over_ui,
            )
                .chain(),
        )
        .run();
}
