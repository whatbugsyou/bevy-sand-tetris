use bevy::prelude::*;
use crate::types::GrainColor;

#[derive(Component, Clone, Copy)]
pub struct Grain {
    pub color: GrainColor,
    pub settled: bool,
}

#[derive(Component)]
pub struct ActivePiece;

#[derive(Component)]
pub struct ClearingGrain;
