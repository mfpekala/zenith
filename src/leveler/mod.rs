use bevy::prelude::*;
use load::{actively_load, destroy_level, did_level_change, is_actively_loading_level, start_load};

use crate::meta::game_state::left_level;

mod load;

pub struct LevelerPlugin;
impl Plugin for LevelerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, start_load.run_if(did_level_change));
        app.add_systems(Update, actively_load.run_if(is_actively_loading_level));
        app.add_systems(Update, destroy_level.run_if(left_level));
    }
}
