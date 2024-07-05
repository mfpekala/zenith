use bevy::{prelude::*, utils::HashSet};

use crate::{
    drawing::{
        effects::{ScreenEffect, ScreenEffectManager},
        layering::menu_layer,
        text::{TextAlign, TextBoxBundle, TextWeight},
    },
    environment::background::{BgEffect, BgManager},
    meta::{
        consts::{MENU_HEIGHT, MENU_WIDTH},
        game_state::{GameState, MenuState, MetaState, PauseState, PrevGameState, SetPaused},
    },
    sound::SoundSettings,
};

use super::{
    button::{MenuButton, MenuButtonBundle, MenuButtonFill, MenuButtonPressed},
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
                if bg_manager.has_stateful_effect() {
                    None
                } else {
                    match specific_menu {
                        MenuState::Studio => None,
                        MenuState::Title => None,
                        MenuState::ConstellationSelect => {
                            bg_manager.clear_effects();
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
                            bg_manager.clear_effects();
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
    if keyboard.just_pressed(KeyCode::Escape) {
        match gs.pause {
            Some(PauseState::Settings { prev_level, .. }) => {
                if prev_level {
                    pause_writer.send(SetPaused(Some(PauseState::Level)));
                } else {
                    pause_writer.send(SetPaused(None));
                }
            }
            Some(_) => {
                pause_writer.send(SetPaused(None));
            }
            None => (),
        }
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
                        MenuButton::basic("go_settings", "Settings"),
                        GameRelativePlacement::new(IVec3::new(0, 24, 12), 1.0),
                    ));
                    parent.spawn(MenuButtonBundle::new(
                        MenuButton::basic("back_galaxy", "Back to Galaxy Select"),
                        GameRelativePlacement::new(IVec3::new(0, 0, 12), 1.0),
                    ));
                    parent.spawn(MenuButtonBundle::new(
                        MenuButton::basic("exit_menu", "Exit to Main Menu"),
                        GameRelativePlacement::new(IVec3::new(0, -24, 12), 1.0),
                    ));
                });
        }
        PauseState::Editor => {
            commands
                .spawn(PauseRoot::new_root("editor"))
                .with_children(|parent| {
                    parent.spawn(MenuButtonBundle::new(
                        MenuButton::basic("exit_menu", "Exit to main menu"),
                        GameRelativePlacement::new(IVec3::new(0, 0, 12), 1.0),
                    ));
                });
        }
        PauseState::Settings { .. } => {
            commands
                .spawn(PauseRoot::new_root("settings"))
                .with_children(|parent| {
                    // Main volume "slider"
                    let main_bund = TextBoxBundle::new_menu_text(
                        "Main Volume",
                        24.0,
                        GameRelativePlacement::new(IVec3::new(0, 48, 12), 0.5),
                        Color::WHITE,
                        TextWeight::default(),
                        TextAlign::Center,
                    );
                    parent.spawn(main_bund);
                    for discrete in (0..5).into_iter() {
                        let x = (discrete - 2) * 12;
                        let id = format!("set_main_volume{discrete}");
                        parent.spawn(MenuButtonBundle::new(
                            MenuButton::basic(&id, " "),
                            GameRelativePlacement::new(IVec3::new(x, 32, 12), 1.0),
                        ));
                    }

                    // Music volume "slider"
                    let main_bund = TextBoxBundle::new_menu_text(
                        "Music Volume",
                        24.0,
                        GameRelativePlacement::new(IVec3::new(0, 12, 12), 0.5),
                        Color::WHITE,
                        TextWeight::default(),
                        TextAlign::Center,
                    );
                    parent.spawn(main_bund);
                    for discrete in (0..5).into_iter() {
                        let x = (discrete - 2) * 12;
                        let id = format!("set_music_volume{discrete}");
                        parent.spawn(MenuButtonBundle::new(
                            MenuButton::basic(&id, " "),
                            GameRelativePlacement::new(IVec3::new(x, -4, 12), 1.0),
                        ));
                    }

                    // Effect volume "slider"
                    let main_bund = TextBoxBundle::new_menu_text(
                        "Effect Volume",
                        24.0,
                        GameRelativePlacement::new(IVec3::new(0, -26, 12), 0.5),
                        Color::WHITE,
                        TextWeight::default(),
                        TextAlign::Center,
                    );
                    parent.spawn(main_bund);
                    for discrete in (0..5).into_iter() {
                        let x = (discrete - 2) * 12;
                        let id = format!("set_effect_volume{discrete}");
                        parent.spawn(MenuButtonBundle::new(
                            MenuButton::basic(&id, " "),
                            GameRelativePlacement::new(IVec3::new(x, -42, 12), 1.0),
                        ));
                    }
                });
        }
    }
}

pub(super) fn update_pause(
    gs: Res<GameState>,
    mut sound_settings: ResMut<SoundSettings>,
    mut buttons: Query<&mut MenuButtonFill>,
    mut button_pressed: EventReader<MenuButtonPressed>,
    mut screen_effects: ResMut<ScreenEffectManager>,
    mut pause_writer: EventWriter<SetPaused>,
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
                "go_settings" => {
                    pause_writer.send(SetPaused(Some(PauseState::Settings {
                        prev_level: true,
                        prev_menu: false,
                    })));
                }
                "back_galaxy" => {
                    screen_effects.queue_effect(ScreenEffect::FadeToBlack(Some(GameState {
                        meta: MetaState::Menu(MenuState::GalaxyOverworld),
                        pause: None,
                    })));
                    screen_effects.queue_effect(ScreenEffect::UnfadeToBlack);
                }
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
        PauseState::Settings { .. } => {
            if let Some(last_button) = last_button {
                if last_button.0.starts_with("set_main_volume") {
                    let last_char_int = last_button
                        .0
                        .chars()
                        .last()
                        .unwrap()
                        .to_string()
                        .parse::<i32>()
                        .unwrap();
                    sound_settings.main_volume = last_char_int as f32 / 5.0;
                }
                if last_button.0.starts_with("set_music_volume") {
                    let last_char_int = last_button
                        .0
                        .chars()
                        .last()
                        .unwrap()
                        .to_string()
                        .parse::<i32>()
                        .unwrap();
                    sound_settings.music_volume = last_char_int as f32 / 5.0;
                }
                if last_button.0.starts_with("set_effect_volume") {
                    let last_char_int = last_button
                        .0
                        .chars()
                        .last()
                        .unwrap()
                        .to_string()
                        .parse::<i32>()
                        .unwrap();
                    sound_settings.effect_volume = last_char_int as f32 / 5.0;
                }
            }

            // Whether each fill should be marked as selected
            let mut selected_set = HashSet::<String>::new();

            let main_selected = (sound_settings.main_volume * 5.0).round() as i32;
            let main_id = format!("set_main_volume{main_selected}");
            selected_set.insert(main_id);

            let music_selected = (sound_settings.music_volume * 5.0).round() as i32;
            let music_id = format!("set_music_volume{music_selected}");
            selected_set.insert(music_id);

            let effect_selected = (sound_settings.effect_volume * 5.0).round() as i32;
            let effect_id = format!("set_effect_volume{effect_selected}");
            selected_set.insert(effect_id);

            for mut fill in buttons.iter_mut() {
                let id = fill.id.clone();
                fill.is_selected = selected_set.contains(&id);
            }
        }
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
