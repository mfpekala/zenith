use bevy::prelude::*;
use load::{actively_load, did_level_change, is_actively_loading_level, start_load};

mod load;

pub struct LevelerPlugin;
impl Plugin for LevelerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, start_load.run_if(did_level_change));
        app.add_systems(Update, actively_load.run_if(is_actively_loading_level));
    }
}
