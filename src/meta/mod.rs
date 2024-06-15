use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use progress::{
    continue_initializing_game_progress, initialize_game_progress, is_progress_initializing,
    GameProgress,
};

use self::level_data::{crystallize_level_data, spawn_level, LevelDataOneshots};

pub mod consts;
pub mod game_state;
pub mod level_data;
pub mod progress;

pub struct MetaPlugin;
impl Plugin for MetaPlugin {
    fn build(&self, app: &mut App) {
        let crystallize_level_data_id = app.world.register_system(crystallize_level_data);
        let spawn_level_id = app.world.register_system(spawn_level);
        app.insert_resource(LevelDataOneshots {
            crystallize_level_data_id,
            spawn_level_id,
        });

        app.register_type::<GameProgress>();
        app.add_plugins(RonAssetPlugin::<GameProgress>::new(&["progress.ron"]));
        app.add_systems(Startup, initialize_game_progress);
        app.add_systems(
            Update,
            continue_initializing_game_progress.run_if(is_progress_initializing),
        );
    }
}
