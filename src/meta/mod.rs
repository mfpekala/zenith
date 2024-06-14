use bevy::prelude::*;

use self::level_data::{crystallize_level_data, spawn_level, LevelDataOneshots};

pub mod galaxy;
pub mod consts;
pub mod game_state;
pub mod level_data;

pub struct MetaPlugin;
impl Plugin for MetaPlugin {
    fn build(&self, app: &mut App) {
        let crystallize_level_data_id = app.world.register_system(crystallize_level_data);
        let spawn_level_id = app.world.register_system(spawn_level);
        app.insert_resource(LevelDataOneshots {
            crystallize_level_data_id,
            spawn_level_id,
        });
    }
}
