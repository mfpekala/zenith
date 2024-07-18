use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use progress::{
    continue_initializing_game_progress, initialize_game_progress, is_progress_initializing,
    save_game_progress, GameProgress,
};

pub mod consts;
pub mod game_state;
pub mod level_data;
pub mod progress;

pub struct MetaPlugin;
impl Plugin for MetaPlugin {
    fn build(&self, app: &mut App) {
        level_data::register_level_data_oneshots(app);

        app.register_type::<GameProgress>();
        app.add_plugins(RonAssetPlugin::<GameProgress>::new(&["progress.ron"]));
        app.add_systems(Startup, initialize_game_progress);
        app.add_systems(
            Update,
            continue_initializing_game_progress.run_if(is_progress_initializing),
        );
        app.add_systems(Update, save_game_progress);
    }
}
