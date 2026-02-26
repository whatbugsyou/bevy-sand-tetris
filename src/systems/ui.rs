use bevy::prelude::*;
use crate::resources::GameStatus;

#[derive(Component)]
pub struct ScoreText;

#[derive(Component)]
pub struct GameOverOverlay;

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
}

pub fn update_score_ui(
    game_status: Res<GameStatus>,
    mut query: Query<&mut Text, With<ScoreText>>,
) {
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
