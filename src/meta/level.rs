use crate::environment::rock::RockType;
use bevy::prelude::*;
use std::path::{Path, PathBuf};

pub fn get_level_folder() -> PathBuf {
    Path::new("assets/levels").into()
}

/// All the data that exists about a level.
/// Just the data that needs to be used to load/play the level
#[derive(serde::Serialize)]
pub struct Level {
    rocks: Vec<(RockType, Vec<Vec2>)>,
}
