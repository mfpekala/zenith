pub mod constellation_screen;
mod placement;
pub mod title_screen;

use bevy::prelude::*;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        title_screen::register_title_screen(app);
        constellation_screen::register_constellation_screen(app);
        app.add_systems(FixedUpdate, placement::update_game_relative_placements);
    }
}
