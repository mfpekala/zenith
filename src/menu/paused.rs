use bevy::prelude::*;

use crate::{
    drawing::{
        effects::{ScreenEffect, ScreenEffectManager},
        layering::menu_layer,
    },
    environment::background::{BgEffect, BgManager},
    meta::{
        consts::{MENU_HEIGHT, MENU_WIDTH},
        game_state::{GameState, MenuState, MetaState, PauseState, PrevGameState, SetPaused},
    },
};

use super::{
    button::{MenuButton, MenuButtonBundle, MenuButtonPressed},
    placement::GameRelativePlacement,
};

pub(super) fn did_any_pause_start(
    old_state: Res<PrevGameState>,
    new_state: Res<GameState>,
) -> bool {
    old_state.pause.is_none() && new_state.pause.is_some()
}

pub(super) fn did_any_pause_end(old_state: Res<PrevGameState>, new_state: Res<GameState>) -> bool {
    old_state.pause.is_some() && new_state.pause.is_none()
}

pub(super) fn did_specific_pause_start(
    old_state: Res<PrevGameState>,
    new_state: Res<GameState>,
) -> bool {
    match (old_state.pause, new_state.pause) {
        (None, None) => false,
        (None, Some(_)) => true,
        (Some(_), None) => false,
        (Some(old), Some(new)) => old != new,
    }
}

pub fn is_paused(state: Res<GameState>) -> bool {
    state.pause.is_some()
}

pub fn is_unpaused(state: Res<GameState>) -> bool {
    state.pause.is_none()
}

pub fn start_pause(
    mut pause_writer: EventWriter<SetPaused>,
    gs: Res<GameState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut bg_manager: ResMut<BgManager>,
) {
    if keyboard.just_pressed(KeyCode::Escape) && gs.pause.is_none() {
        let pause_state = match &gs.meta {
            MetaState::Menu(specific_menu) => {
                if bg_manager.has_active_effect() {
                    None
                } else {
                    match specific_menu {
                        MenuState::Title => None,
                        MenuState::ConstellationSelect => {
                            bg_manager.queue_effect(BgEffect::default_menu_scroll(
                                false,
                                true,
                                Some(MetaState::Menu(MenuState::Title)),
                            ));
                            bg_manager
                                .queue_effect(BgEffect::default_menu_scroll(false, false, None));
                            None
                        }
                        MenuState::GalaxyOverworld => {
                            bg_manager.queue_effect(BgEffect::default_menu_scroll(
                                false,
                                true,
                                Some(MetaState::Menu(MenuState::ConstellationSelect)),
                            ));
                            bg_manager
                                .queue_effect(BgEffect::default_menu_scroll(false, false, None));
                            None
                        }
                    }
                }
            }
            MetaState::Level(_) => Some(PauseState::Level),
            MetaState::Editor(_) => Some(PauseState::Editor),
        };
        if pause_state.is_some() {
            pause_writer.send(SetPaused(pause_state));
        }
    }
}

pub(super) fn stop_pause(
    mut pause_writer: EventWriter<SetPaused>,
    gs: Res<GameState>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) && gs.pause.is_some() {
        pause_writer.send(SetPaused(None));
    }
}

#[derive(Component)]
pub(super) struct PauseBackground;

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

pub(super) fn setup_any_pause(gs: Res<GameState>, mut commands: Commands) {
    let Some(_pause_state) = gs.pause else {
        // Shouldn't happen
        return;
    };
    commands.spawn((
        PauseBackground,
        Name::new("paused_dark_box"),
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgba(0.2, 0.2, 0.2, 0.8),
                ..default()
            },
            transform: Transform {
                scale: Vec3::new(MENU_WIDTH as f32, MENU_HEIGHT as f32, 1.0),
                translation: Vec3::new(0.0, 0.0, 10.0),
                ..default()
            },
            ..default()
        },
        menu_layer(),
    ));
}

pub(super) fn setup_specific_pause(
    gs: Res<GameState>,
    roots: Query<Entity, With<PauseRoot>>,
    mut commands: Commands,
) {
    let Some(pause) = gs.pause else {
        // Shouldn't happen
        return;
    };
    for root in roots.iter() {
        commands.entity(root).despawn_recursive();
    }
    match pause {
        PauseState::Level => {
            commands
                .spawn(PauseRoot::new_root("level"))
                .with_children(|parent| {
                    parent.spawn(MenuButtonBundle::new(
                        MenuButton::basic("exit_menu", "Exit to main menu"),
                        GameRelativePlacement::new(IVec3::new(0, 0, 12), 1.0),
                    ));
                });
        }
        PauseState::Editor => {
            commands
                .spawn(PauseRoot::new_root("level"))
                .with_children(|parent| {
                    parent.spawn(MenuButtonBundle::new(
                        MenuButton::basic("exit_menu", "Exit to main menu"),
                        GameRelativePlacement::new(IVec3::new(0, 0, 12), 1.0),
                    ));
                });
        }
        _ => todo!("setup_specific_pause"),
    }
}

pub(super) fn update_pause(
    gs: Res<GameState>,
    mut button_pressed: EventReader<MenuButtonPressed>,
    mut screen_effects: ResMut<ScreenEffectManager>,
) {
    let Some(pause_state) = gs.pause else {
        // Shouldn't happen
        return;
    };
    let last_button = button_pressed.read().last();
    match pause_state {
        PauseState::Level => {
            let Some(last_button) = last_button else {
                return;
            };
            match last_button.0.as_str() {
                "exit_menu" => {
                    screen_effects.queue_effect(ScreenEffect::FadeToBlack(Some(GameState {
                        meta: MetaState::Menu(MenuState::Title),
                        pause: None,
                    })));
                    screen_effects.queue_effect(ScreenEffect::UnfadeToBlack);
                }
                _ => panic!("Bad button press on level pause menu"),
            }
        }
        PauseState::Editor => {
            let Some(last_button) = last_button else {
                return;
            };
            match last_button.0.as_str() {
                "exit_menu" => {
                    screen_effects.queue_effect(ScreenEffect::FadeToBlack(Some(GameState {
                        meta: MetaState::Menu(MenuState::Title),
                        pause: None,
                    })));
                    screen_effects.queue_effect(ScreenEffect::UnfadeToBlack);
                }
                _ => panic!("Bad button press on editor pause menu"),
            }
        }
        _ => todo!("pause_state update"),
    }
}

pub(super) fn destroy_any_pause(
    pause_root: Query<Entity, With<PauseRoot>>,
    pause_background: Query<Entity, With<PauseBackground>>,
    mut commands: Commands,
) {
    for eid in pause_root.iter().chain(pause_background.iter()) {
        commands.entity(eid).despawn_recursive();
    }
}
