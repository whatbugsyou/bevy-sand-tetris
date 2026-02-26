pub const IPHONE14_WIDTH: f32 = 390.0;
pub const IPHONE14_HEIGHT: f32 = 844.0;
pub const BASE_BOARD_WIDTH: i32 = 10;
pub const BASE_BOARD_HEIGHT: i32 = 22;
pub const BASE_MINO_SIZE: f32 = IPHONE14_HEIGHT / BASE_BOARD_HEIGHT as f32;
pub const PIECE_SUBDIVISION: i32 = 8; // each original mino becomes 8x8 grains
pub const BOARD_WIDTH: i32 = BASE_BOARD_WIDTH * PIECE_SUBDIVISION;
pub const BOARD_HEIGHT: i32 = BASE_BOARD_HEIGHT * PIECE_SUBDIVISION;
pub const GRAIN_SIZE: f32 = BASE_MINO_SIZE / PIECE_SUBDIVISION as f32;
pub const FALL_INTERVAL: f32 = 0.5 / PIECE_SUBDIVISION as f32; // keep similar world-space falling speed
pub const SAND_STEP_INTERVAL: f32 = 0.03;
/// Extra pixels below the board reserved for the next-piece preview UI.
pub const PREVIEW_AREA_HEIGHT: f32 = 100.0;
/// Shift the board up so the bottom 100px of the window is free for the UI.
pub const FLOOR_Y: f32 =
    -(BOARD_HEIGHT as f32 * GRAIN_SIZE) * 0.5 + PREVIEW_AREA_HEIGHT * 0.5;
pub const SPAWN_Y: f32 = FLOOR_Y + (BOARD_HEIGHT as f32 - 2.0) * GRAIN_SIZE;
