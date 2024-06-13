use bevy::prelude::*;

use crate::{
    drawing::{
        effects::{ScreenEffect, ScreenEffectManager},
        layering::menu_layer,
    },
    meta::{
        consts::{MENU_HEIGHT, MENU_WIDTH},
        game_state::{GameState, MenuState, MetaState, PrevGameState, SetPaused},
    },
};

use super::{
    button::{MenuButton, MenuButtonBundle, MenuButtonPressed},
    placement::GameRelativePlacement,
};

pub(super) fn did_pause_start(old_state: Res<PrevGameState>, new_state: Res<GameState>) -> bool {
    !old_state.paused && new_state.paused
}

pub(super) fn did_pause_end(old_state: Res<PrevGameState>, new_state: Res<GameState>) -> bool {
    old_state.paused && !new_state.paused
}

pub fn is_paused(state: Res<GameState>) -> bool {
    state.paused
}

pub fn is_unpaused(state: Res<GameState>) -> bool {
    !state.paused
}

pub(super) fn start_pause(
    mut pause_writer: EventWriter<SetPaused>,
    gs: Res<GameState>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) && !gs.paused && !gs.is_in_menu() {
        pause_writer.send(SetPaused(true));
    }
}

pub(super) fn stop_pause(
    mut pause_writer: EventWriter<SetPaused>,
    gs: Res<GameState>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) && gs.paused {
        pause_writer.send(SetPaused(false));
    }
}

#[derive(Component)]
pub(super) struct PauseRoot;
impl PauseRoot {
    fn new_root(name: &str) -> impl Bundle {
        (
            PauseRoot,
            SpatialBundle::default(),
            Name::new(format!("pause_root_{name}")),
        )
    }
}

pub(super) fn setup_pause(
    gs: Res<GameState>,
    mut commands: Commands,
    mut set_paused_writer: EventWriter<SetPaused>,
) {
    if !gs.paused {
        // Shouldn't happen
        return;
    }
    let spawn_background = |parent: &mut ChildBuilder, color: Color| {
        parent.spawn((
            Name::new("paused_dark_box"),
            SpriteBundle {
                sprite: Sprite { color, ..default() },
                transform: Transform {
                    scale: Vec3::new(MENU_WIDTH as f32, MENU_HEIGHT as f32, 1.0),
                    translation: Vec3::new(0.0, 0.0, 10.0),
                    ..default()
                },
                ..default()
            },
            menu_layer(),
        ));
    };
    match &gs.meta {
        MetaState::Menu(_) => {
            set_paused_writer.send(SetPaused(false));
            return;
        }
        MetaState::Level(_) => {
            commands
                .spawn(PauseRoot::new_root("level"))
                .with_children(|parent| {
                    spawn_background(parent, Color::rgba(0.2, 0.2, 0.2, 0.8));
                    parent.spawn(MenuButtonBundle::new(
                        MenuButton::basic("exit_menu", "Exit to main menu"),
                        GameRelativePlacement::new(IVec3::new(0, 0, 12), 1.0),
                    ));
                });
        }
        MetaState::Editor(_) => {}
    }
}

pub(super) fn update_pause(
    mut button_pressed: EventReader<MenuButtonPressed>,
    gs: Res<GameState>,
    mut screen_effects: ResMut<ScreenEffectManager>,
) {
    let last_button = button_pressed.read().last();
    match &gs.meta {
        MetaState::Menu(_) => (),
        MetaState::Level(_) => {
            let Some(last_button) = last_button else {
                return;
            };
            match last_button.0.as_str() {
                "exit_menu" => {
                    screen_effects.queue_effect(ScreenEffect::FadeToBlack(Some(GameState {
                        meta: MetaState::Menu(MenuState::Title),
                        paused: false,
                    })));
                    screen_effects.queue_effect(ScreenEffect::UnfadeToBlack);
                }
                _ => panic!("Bad button press on menu"),
            }
        }
        MetaState::Editor(_) => {}
    }
}

pub(super) fn destroy_pause(pause_root: Query<Entity, With<PauseRoot>>, mut commands: Commands) {
    for eid in pause_root.iter() {
        commands.entity(eid).despawn_recursive();
    }
}
