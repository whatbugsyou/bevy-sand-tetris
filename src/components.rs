use crate::types::GrainColor;
use bevy::prelude::*;

#[derive(Component, Clone, Copy)]
pub struct Grain {
    pub color: GrainColor,
    pub settled: bool,
    pub stable: bool, // true if grain cannot move (has support below and blocked diagonals)
}

#[derive(Component)]
pub struct ActivePiece;

/// Marker + animation data for grains that have been cleared and are flying off the board.
#[derive(Component)]
pub struct PopOutGrain {
    pub launch_delay: f32, // seconds before this grain launches (left→right stagger)
    pub elapsed: f32,      // total time since the clear animation started
    pub origin_x: f32,     // world X at the moment of clear
    pub origin_y: f32,     // world Y at the moment of clear
}
