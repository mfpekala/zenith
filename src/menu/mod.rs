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

        app.add_event::<MenuButtonPressed>();
        app.add_systems(Update, materialize_buttons.after(setup_specific_pause));
        app.add_systems(FixedUpdate, materialize_button_backgrounds);
        app.add_systems(
            Update,
            (update_button_state, update_button_fill_colors).chain(),
        );
    }
}
