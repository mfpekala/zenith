use bevy::prelude::*;

use self::level_data::{export_level, LevelDataOneshots};

pub mod consts;
pub mod game_state;
pub mod level_data;

pub struct MetaPlugin;
impl Plugin for MetaPlugin {
    fn build(&self, app: &mut App) {
        let export_level_id = app.world.register_system(export_level);
        app.insert_resource(LevelDataOneshots { export_level_id });
    }
}
