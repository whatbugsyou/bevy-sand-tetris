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

#[derive(Component)]
pub struct ClearingGrain;
