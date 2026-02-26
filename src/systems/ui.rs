use crate::components::{ActivePiece, Grain, PreviewSlotButton};
use crate::constants::{NUM_CANDIDATES, PREVIEW_AREA_HEIGHT, *};
use crate::resources::{BoardGrid, ClearEffect, GameStatus, NextPieceQueue};
use crate::systems::spawn::col_to_world_x;
use crate::types::{GrainColor, TetrominoShape};
use bevy::prelude::*;

const SLOT_NORMAL_COLOR: Color = Color::srgba(0.10, 0.10, 0.14, 1.0);
const SLOT_HOVERED_COLOR: Color = Color::srgba(0.18, 0.18, 0.26, 1.0);
const SLOT_PRESSED_COLOR: Color = Color::srgba(0.26, 0.26, 0.38, 1.0);

#[derive(Component)]
pub struct ScoreText;

#[derive(Component)]
pub struct GameOverOverlay;

/// Marks a single cell in a next-piece preview grid.
#[derive(Component)]
pub struct PreviewCell {
    /// 0 = first upcoming piece, 1 = second upcoming piece
    pub slot: usize,
    /// Column in the 4×2 normalised display grid (0–3)
    pub col: i32,
    /// Row in the 4×2 normalised display grid (0–1)
    pub row: i32,
}

const CELL_SIZE: f32 = 20.0;
const CELL_GAP: f32 = 2.0;
const PREVIEW_COLS: i32 = 4;
const PREVIEW_ROWS: i32 = 2;
const EMPTY_CELL_COLOR: Color = Color::srgb(0.15, 0.15, 0.18);

pub fn setup_ui(mut commands: Commands) {
    // Score text at top-left
    commands.spawn((
        ScoreText,
        Text::new("Score: 0"),
        TextFont {
            font_size: 28.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            top: Val::Px(20.0),
            ..default()
        },
    ));

    // Bottom panel: full-width solid background + "NEXT" label + 2 piece preview grids.
    // The solid background covers the entire PREVIEW_AREA_HEIGHT strip, ensuring
    // no world-space grain sprites are visible beneath the UI.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.0),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                height: Val::Px(PREVIEW_AREA_HEIGHT),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(6.0),
                border: UiRect::top(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.07, 0.07, 0.10)),
            BorderColor::all(Color::srgb(0.35, 0.35, 0.40)),
        ))
        .with_children(|panel| {
            // "NEXT" label
            panel.spawn((
                Text::new("NEXT"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.6)),
            ));

            // Row of candidate slots (NUM_CANDIDATES)
            panel
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(10.0),
                    ..default()
                })
                .with_children(|row| {
                    for slot in 0..NUM_CANDIDATES {
                        // Each slot is a clickable button container
                        row.spawn((
                            Button,
                            PreviewSlotButton { slot },
                            Node {
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(CELL_GAP),
                                padding: UiRect::all(Val::Px(5.0)),
                                border: UiRect::all(Val::Px(2.0)),
                                ..default()
                            },
                            BackgroundColor(SLOT_NORMAL_COLOR),
                            BorderColor::all(Color::srgba(0.45, 0.45, 0.60, 0.5)),
                        ))
                        .with_children(|slot_col| {
                            for r in (0..PREVIEW_ROWS).rev() {
                                // one grid row
                                slot_col
                                    .spawn(Node {
                                        flex_direction: FlexDirection::Row,
                                        column_gap: Val::Px(CELL_GAP),
                                        ..default()
                                    })
                                    .with_children(|grid_row| {
                                        for c in 0..PREVIEW_COLS {
                                            grid_row.spawn((
                                                PreviewCell {
                                                    slot,
                                                    col: c,
                                                    row: r,
                                                },
                                                Node {
                                                    width: Val::Px(CELL_SIZE),
                                                    height: Val::Px(CELL_SIZE),
                                                    ..default()
                                                },
                                                BackgroundColor(EMPTY_CELL_COLOR),
                                            ));
                                        }
                                    });
                            }
                        });
                    }
                });
        });
}

pub fn update_score_ui(game_status: Res<GameStatus>, mut query: Query<&mut Text, With<ScoreText>>) {
    if game_status.is_changed() {
        for mut text in &mut query {
            **text = format!("Score: {}", game_status.score);
        }
    }
}

pub fn game_over_ui(
    game_status: Res<GameStatus>,
    mut commands: Commands,
    existing: Query<Entity, With<GameOverOverlay>>,
) {
    if game_status.is_game_over && existing.is_empty() {
        commands
            .spawn((
                GameOverOverlay,
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new(format!("GAME OVER\nScore: {}", game_status.score)),
                    TextFont {
                        font_size: 48.0,
                        ..default()
                    },
                    TextColor(Color::srgba(1.0, 0.3, 0.3, 1.0)),
                    TextLayout::new_with_justify(Justify::Center),
                ));
            });
    }
}

/// Handles hover/press interactions on candidate piece slots.
/// On press: spawns the selected piece as the active piece and replenishes the queue.
pub fn preview_interaction_system(
    mut commands: Commands,
    mut interaction_query: Query<
        (&PreviewSlotButton, &Interaction, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut piece_queue: ResMut<NextPieceQueue>,
    board: Res<BoardGrid>,
    active_query: Query<(), With<ActivePiece>>,
    mut game_status: ResMut<GameStatus>,
    clear_effect: Res<ClearEffect>,
) {
    for (slot_btn, interaction, mut bg) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                bg.0 = SLOT_PRESSED_COLOR;
                // Spawn only when no active piece, game is running, and no clear in progress
                if !game_status.is_game_over
                    && active_query.is_empty()
                    && clear_effect.pending.is_none()
                {
                    let slot = slot_btn.slot;
                    if let Some(&(shape, color)) = piece_queue.pieces.get(slot) {
                        // Remove selected piece and replenish queue
                        piece_queue.pieces.remove(slot);
                        let mut rng = rand::rng();
                        piece_queue.pieces.push_back((
                            TetrominoShape::random(&mut rng),
                            GrainColor::random(&mut rng),
                        ));

                        // Compute centered anchor column
                        let offsets = shape.offsets();
                        let min_offset_x =
                            offsets.iter().map(|o| o.x).min().unwrap_or(0);
                        let max_offset_x =
                            offsets.iter().map(|o| o.x).max().unwrap_or(0);
                        let min_anchor = -min_offset_x;
                        let max_anchor = (BOARD_WIDTH - 1) - max_offset_x;
                        let anchor_col = (min_anchor + max_anchor) / 2;

                        // Check if spawn area is clear
                        let spawn_blocked = offsets.iter().any(|offset| {
                            let col = anchor_col + offset.x;
                            let x = col_to_world_x(col);
                            let y = SPAWN_Y + offset.y as f32 * GRAIN_SIZE;
                            let (_, row) = BoardGrid::world_to_grid_unclamped(x, y);
                            !board.is_free(col, row)
                        });

                        if spawn_blocked {
                            game_status.is_game_over = true;
                            info!(
                                "Game Over! Spawn area blocked. Final score: {}",
                                game_status.score
                            );
                        } else {
                            for offset in offsets {
                                let col = anchor_col + offset.x;
                                let x = col_to_world_x(col);
                                let y = SPAWN_Y + offset.y as f32 * GRAIN_SIZE;
                                commands.spawn((
                                    ActivePiece,
                                    Grain {
                                        color,
                                        settled: false,
                                        stable: false,
                                    },
                                    Sprite::from_color(
                                        color.to_bevy_color(),
                                        Vec2::splat(GRAIN_SIZE),
                                    ),
                                    Transform::from_xyz(x, y, 0.0),
                                    GlobalTransform::default(),
                                ));
                            }
                            info!("Player selected {:?} with color {:?}", shape, color);
                        }
                    }
                }
            }
            Interaction::Hovered => {
                bg.0 = SLOT_HOVERED_COLOR;
            }
            Interaction::None => {
                bg.0 = SLOT_NORMAL_COLOR;
            }
        }
    }
}

/// Updates the colour of every `PreviewCell` to reflect the current `NextPieceQueue`.
pub fn update_preview_ui(
    piece_queue: Res<NextPieceQueue>,
    mut cells: Query<(&PreviewCell, &mut BackgroundColor)>,
) {
    if !piece_queue.is_changed() {
        return;
    }

    for (cell, mut bg) in &mut cells {
        let color = piece_queue
            .pieces
            .get(cell.slot)
            .and_then(|(shape, grain_color)| {
                // Normalise mino offsets so the leftmost column maps to 0
                let offsets = shape.mino_offsets();
                let min_x = offsets.iter().map(|o| o.x).min().unwrap_or(0);
                let occupied = offsets
                    .iter()
                    .any(|o| o.x - min_x == cell.col && o.y == cell.row);
                if occupied {
                    Some(grain_color.to_bevy_color())
                } else {
                    None
                }
            })
            .unwrap_or(EMPTY_CELL_COLOR);

        bg.0 = color;
    }
}
