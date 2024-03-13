pub mod constellation_screen;
pub mod menu_asset;
pub mod title_screen;

use self::menu_asset::MenuAssetPlugin;
use bevy::prelude::*;

pub fn register_menus(app: &mut App) {
    app.add_plugins(MenuAssetPlugin);

    title_screen::register_title_screen(app);
    constellation_screen::register_constellation_screen(app);
}
