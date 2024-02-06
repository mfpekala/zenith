pub mod main_menu;

use bevy::prelude::*;

pub fn register_menus(app: &mut App) {
    main_menu::register_main_menu(app);
}
