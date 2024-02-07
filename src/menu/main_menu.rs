use bevy::prelude::*;

use crate::meta::game_state::{
    entered_menu, left_menu, EditingState, EditorState, GameState, MetaState, SetGameState,
};

#[derive(Component)]
struct MainMenuMarker;

fn setup_main_menu(mut commands: Commands) {
    commands
        .spawn((
            MainMenuMarker,
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: BackgroundColor(Color::rgb(0.01, 0.03, 0.01)),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(150.0),
                        height: Val::Px(65.0),
                        border: UiRect::all(Val::Px(5.0)),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    border_color: BorderColor(Color::BLACK),
                    background_color: Color::WHITE.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Button",
                        TextStyle {
                            font_size: 40.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ));
                });
        });
}

fn destroy_main_menu(mut commands: Commands, ids: Query<Entity, With<MainMenuMarker>>) {
    for id in ids.iter() {
        commands.entity(id).despawn_recursive();
    }
}

fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut state_changer: EventWriter<SetGameState>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                state_changer.send(SetGameState(GameState {
                    meta: MetaState::Editor(EditorState::Editing(EditingState {
                        mode: crate::meta::game_state::EditingMode::Free,
                        paused: false,
                    })),
                }));
            }
            Interaction::Hovered => {
                *color = Color::RED.into();
            }
            Interaction::None => {
                *color = Color::ORANGE.into();
            }
        }
    }
}

pub fn register_main_menu(app: &mut App) {
    app.add_systems(Update, setup_main_menu.run_if(entered_menu));
    app.add_systems(Update, destroy_main_menu.run_if(left_menu));
    app.add_systems(Update, button_system);
}
