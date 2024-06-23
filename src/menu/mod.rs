pub mod button;
pub mod constellation_screen;
pub mod galaxy_screen;
pub mod paused;
pub mod placement;
pub mod title_screen;

use bevy::prelude::*;
use button::{
    materialize_button_backgrounds, materialize_buttons, update_button_fill_colors,
    update_button_state, MenuButtonPressed,
};
use paused::{
    destroy_any_pause, did_any_pause_end, did_any_pause_start, did_specific_pause_start, is_paused,
    is_unpaused, setup_any_pause, setup_specific_pause, start_pause, stop_pause, update_pause,
};

use crate::{
    camera::{CameraMarker, CameraMode},
    environment::background::{BgKind, BgManager},
    meta::game_state::{entered_menu, left_menu},
    physics::dyno::IntMoveable,
    sound::music::{MusicKind, MusicManager},
};

fn setup_any_menu(
    mut cam: Query<(&mut IntMoveable, &mut CameraMarker)>,
    mut bg_manager: ResMut<BgManager>,
    mut music_manager: ResMut<MusicManager>,
) {
    bg_manager.set_kind(BgKind::ParallaxStars(500));
    for (mut mv, mut cam) in cam.iter_mut() {
        mv.pos = IVec3::ZERO;
        cam.mode = CameraMode::Controlled;
    }
    music_manager.fade_to_song(Some(MusicKind::EyeOfTheStorm));
}

fn destroy_any_menu(mut cam: Query<&mut CameraMarker>) {
    for mut cam in cam.iter_mut() {
        cam.mode = CameraMode::Follow;
    }
}

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
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
