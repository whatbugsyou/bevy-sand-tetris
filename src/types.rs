use crate::constants::PIECE_SUBDIVISION;
use bevy::prelude::*;
use rand::{Rng, RngExt};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GrainColor {
    Cyan,
    Yellow,
    Purple,
    Green,
    Red,
}

impl GrainColor {
    const ALL: [GrainColor; 5] = [
        GrainColor::Cyan,
        GrainColor::Yellow,
        GrainColor::Purple,
        GrainColor::Green,
        GrainColor::Red,
    ];

    pub fn random(rng: &mut impl Rng) -> Self {
        Self::ALL[rng.random_range(0..Self::ALL.len())]
    }

    pub fn to_bevy_color(self) -> Color {
        match self {
            GrainColor::Cyan => Color::srgb(0.30, 0.85, 0.95),
            GrainColor::Yellow => Color::srgb(0.96, 0.88, 0.25),
            GrainColor::Purple => Color::srgb(0.72, 0.42, 0.95),
            GrainColor::Green => Color::srgb(0.35, 0.88, 0.50),
            GrainColor::Red => Color::srgb(0.95, 0.35, 0.35),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum TetrominoShape {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

impl TetrominoShape {
    const ALL: [TetrominoShape; 7] = [
        TetrominoShape::I,
        TetrominoShape::O,
        TetrominoShape::T,
        TetrominoShape::S,
        TetrominoShape::Z,
        TetrominoShape::J,
        TetrominoShape::L,
    ];

    pub fn random(rng: &mut impl Rng) -> Self {
        Self::ALL[rng.random_range(0..Self::ALL.len())]
    }

    fn mino_offsets(self) -> [IVec2; 4] {
        match self {
            TetrominoShape::I => [
                IVec2::new(-1, 0),
                IVec2::new(0, 0),
                IVec2::new(1, 0),
                IVec2::new(2, 0),
            ],
            TetrominoShape::O => [
                IVec2::new(0, 0),
                IVec2::new(1, 0),
                IVec2::new(0, 1),
                IVec2::new(1, 1),
            ],
            TetrominoShape::T => [
                IVec2::new(-1, 0),
                IVec2::new(0, 0),
                IVec2::new(1, 0),
                IVec2::new(0, 1),
            ],
            TetrominoShape::S => [
                IVec2::new(0, 0),
                IVec2::new(1, 0),
                IVec2::new(-1, 1),
                IVec2::new(0, 1),
            ],
            TetrominoShape::Z => [
                IVec2::new(-1, 0),
                IVec2::new(0, 0),
                IVec2::new(0, 1),
                IVec2::new(1, 1),
            ],
            TetrominoShape::J => [
                IVec2::new(-1, 0),
                IVec2::new(0, 0),
                IVec2::new(1, 0),
                IVec2::new(-1, 1),
            ],
            TetrominoShape::L => [
                IVec2::new(-1, 0),
                IVec2::new(0, 0),
                IVec2::new(1, 0),
                IVec2::new(1, 1),
            ],
        }
    }

    pub fn offsets(self) -> Vec<IVec2> {
        let mut result = Vec::with_capacity((4 * PIECE_SUBDIVISION * PIECE_SUBDIVISION) as usize);
        for mino in self.mino_offsets() {
            let base_x = mino.x * PIECE_SUBDIVISION;
            let base_y = mino.y * PIECE_SUBDIVISION;
            for dy in 0..PIECE_SUBDIVISION {
                for dx in 0..PIECE_SUBDIVISION {
                    result.push(IVec2::new(base_x + dx, base_y + dy));
                }
            }
        }
        result
    }
}
