pub mod menu_asset;
pub mod title_screen;

use self::menu_asset::MenuAssetPlugin;
use bevy::prelude::*;

pub fn register_menus(app: &mut App) {
    app.add_plugins(MenuAssetPlugin);

    title_screen::register_title_screen(app);
}
