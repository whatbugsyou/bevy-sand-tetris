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

/// Marker component for ghost (preview) grains showing where the active piece will land.
#[derive(Component)]
pub struct GhostGrain;

/// Marks a clickable slot in the bottom candidate area. `slot` is the 0-based index.
#[derive(Component)]
pub struct PreviewSlotButton {
    pub slot: usize,
}

/// Marker + animation data for grains that have been cleared and are flying off the board.
#[derive(Component)]
pub struct PopOutGrain {
    pub launch_delay: f32,  // seconds before this grain launches (random short stagger)
    pub elapsed: f32,       // total time since the clear animation started
    pub origin_x: f32,      // world X at the moment of clear
    pub origin_y: f32,      // world Y at the moment of clear
    pub vel_x: f32,         // horizontal velocity (px/s)
    pub vel_y: f32,         // initial vertical velocity (px/s)
    pub rot_speed: f32,     // Z-axis spin speed (rad/s)
    pub flip_speed: f32,    // pseudo-3D Y-axis rotation speed (rad/s)
    pub flip_phase: f32,    // initial phase offset for Y-axis flip
    pub base_color: Color,  // original grain color for flash lerp
}
