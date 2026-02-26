use bevy::prelude::*;

mod components;
mod constants;
mod resources;
mod systems;
mod types;

use constants::{IPHONE14_HEIGHT, IPHONE14_WIDTH, PREVIEW_AREA_HEIGHT, SAND_STEP_INTERVAL};
use resources::{
    BoardDirty, BoardGrid, ClearEffect, ClearScratch, GameStatus, NextPieceQueue, SandTimer,
};
use systems::{
    clear::{clear_system, pop_out_system},
    game_over::game_over_check_system,
    ghost::ghost_system,
    input::input_system,
    sand::sand_physics_system,
    setup::setup_scene,
    spawn::init_queue_system,
    ui::{game_over_ui, preview_interaction_system, setup_ui, update_preview_ui, update_score_ui},
};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.08)))
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
        .add_systems(Startup, (setup_scene, setup_ui, init_queue_system))
        .add_systems(
            Update,
            (
                preview_interaction_system,
                input_system,
                ghost_system,
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
