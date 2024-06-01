pub mod constellation_screen;
pub mod title_screen;

use bevy::prelude::*;

pub fn register_menus(app: &mut App) {
    title_screen::register_title_screen(app);
    constellation_screen::register_constellation_screen(app);
}
