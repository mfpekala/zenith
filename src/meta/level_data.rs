use bevy::prelude::*;

/// All the data that exists about a level.
/// Just the data that needs to be used to load/play the level
#[derive(
    serde::Serialize,
    serde::Deserialize,
    bevy::asset::Asset,
    bevy::reflect::TypePath,
    Debug,
    PartialEq,
    Clone,
    Resource,
    Default,
)]
pub struct LevelData {
    
}
impl LevelData {}
