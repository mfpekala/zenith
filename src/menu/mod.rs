pub mod button;
pub mod constellation_screen;
pub mod paused;
pub mod placement;
pub mod title_screen;

use bevy::prelude::*;
use button::{materialize_button_backgrounds, materialize_buttons};
use paused::{
    destroy_pause, did_pause_end, did_pause_start, is_paused, setup_pause, start_pause,
    update_pause,
};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        title_screen::register_title_screen(app);
        constellation_screen::register_constellation_screen(app);
        app.add_systems(FixedUpdate, placement::update_game_relative_placements);
        app.add_systems(Update, start_pause);
        app.add_systems(Update, setup_pause.run_if(did_pause_start));
        app.add_systems(Update, update_pause.run_if(is_paused));
        app.add_systems(Update, destroy_pause.run_if(did_pause_end));
        app.add_systems(Update, materialize_buttons.after(setup_pause));
        app.add_systems(FixedUpdate, materialize_button_backgrounds);
    }
}
