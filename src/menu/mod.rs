pub mod button;
pub mod constellation_screen;
pub mod galaxy_screen;
pub mod paused;
pub mod placement;
pub mod title_screen;

use bevy::prelude::*;
use button::{
    materialize_button_backgrounds, materialize_buttons, update_button_fill_colors,
    update_button_state, MenuButton, MenuButtonBundle, MenuButtonPressed,
};
use paused::{
    destroy_any_pause, did_any_pause_end, did_any_pause_start, did_specific_pause_start, is_paused,
    is_unpaused, setup_any_pause, setup_specific_pause, start_pause, stop_pause, update_pause,
};
use placement::GameRelativePlacement;

use crate::{
    camera::{CameraMarker, CameraMode, CameraScale},
    environment::background::{BgKind, BgManager},
    meta::game_state::{entered_menu, in_menu, left_menu, GameState, PauseState, SetPaused},
    physics::dyno::IntMoveable,
    sound::music::{MusicKind, MusicManager},
};

#[derive(Component)]
struct MenuSettingsButton;

fn setup_any_menu(
    mut cam: Query<(&mut IntMoveable, &mut CameraMarker)>,
    mut bg_manager: ResMut<BgManager>,
    mut music_manager: ResMut<MusicManager>,
    mut commands: Commands,
) {
    bg_manager.set_kind(BgKind::ParallaxStars(500));
    for (mut mv, mut cam) in cam.iter_mut() {
        mv.pos = IVec3::ZERO;
        cam.mode = CameraMode::Controlled;
        cam.scale = CameraScale::One;
    }
    music_manager.fade_to_song(Some(MusicKind::EyeOfTheStorm));
    commands.spawn((
        MenuButtonBundle::new(
            MenuButton::basic("go_settings", "S"),
            GameRelativePlacement::new(IVec3::new(-151, -78, 0), 1.0),
        ),
        MenuSettingsButton,
    ));
}

/// Update logic common to any menu. This is basically just used so that we
/// can watch for settings logic
fn update_any_menu(
    gs: Res<GameState>,
    mut button_pressed: EventReader<MenuButtonPressed>,
    mut pause_writer: EventWriter<SetPaused>,
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
) {
    // See if the settings button was pressed
    let last_button = button_pressed.read().last();
    if let Some(button_press) = last_button {
        match button_press.0.as_str() {
            "go_settings" => {
                pause_writer.send(SetPaused(Some(PauseState::Settings {
                    prev_level: false,
                    prev_menu: true,
                })));
            }
            _ => (),
        }
    }
    // When there is a pause, capture the escape press and unpause
    if let Some(_pause) = &gs.pause {
        if keyboard.just_pressed(KeyCode::Escape) {
            pause_writer.send(SetPaused(None));
        }
        keyboard.reset_all();
    }
}

fn destroy_any_menu(
    mut cam: Query<&mut CameraMarker>,
    menu_settings_buttons: Query<Entity, With<MenuSettingsButton>>,
    mut commands: Commands,
) {
    for mut cam in cam.iter_mut() {
        cam.mode = CameraMode::Follow { dislodgement: None };
    }
    for msb in menu_settings_buttons.iter() {
        commands.entity(msb).despawn_recursive();
    }
}

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_any_menu.run_if(in_menu));

        title_screen::register_title_screen(app);
        constellation_screen::register_constellation_screen(app);
        galaxy_screen::register_galaxy_screen(app);

        app.add_systems(
            Update,
            placement::update_game_relative_placements.after(setup_specific_pause),
        );

        app.add_systems(Update, start_pause.run_if(is_unpaused));
        app.add_systems(Update, stop_pause.run_if(is_paused));

        app.add_systems(Update, setup_any_pause.run_if(did_any_pause_start));
        app.add_systems(Update, destroy_any_pause.run_if(did_any_pause_end));

        app.add_systems(
            Update,
            setup_specific_pause.run_if(did_specific_pause_start),
        );

        app.add_systems(Update, update_pause.run_if(is_paused));

        app.add_systems(Update, setup_any_menu.run_if(entered_menu));
        app.add_systems(Update, destroy_any_menu.run_if(left_menu));

        app.add_event::<MenuButtonPressed>();
        app.add_systems(Update, materialize_buttons.after(setup_specific_pause));
        app.add_systems(FixedUpdate, materialize_button_backgrounds);
        app.add_systems(
            Update,
            (update_button_state, update_button_fill_colors).chain(),
        );
    }
}
