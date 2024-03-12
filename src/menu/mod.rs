pub mod main_menu;
pub mod menu_asset;

use self::menu_asset::MenuAssetPlugin;
use bevy::prelude::*;

pub fn register_menus(app: &mut App) {
    app.add_plugins(MenuAssetPlugin);

    main_menu::register_main_menu(app);
}
