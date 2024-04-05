use bevy::prelude::*;

use crate::environment::{
    field::{FieldDrag, FieldStrength},
    rock::RockKind,
};

/// For rehydrating exported level data (saves, intermediate editor output) into spawnable things
/// NOTE: _Spawnable_ things. This means you usually implement Rehydrate<SomeBundle>
pub trait Rehydrate<T> {
    fn rehydrate(self) -> T;
}

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
pub struct ExportedRock {
    pub kind: RockKind,
    pub points: Vec<IVec2>,
    pub z: i32,
}

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
pub struct ExportedField {
    pub points: Vec<IVec2>,
    pub dir: Vec2,
    pub strength: FieldStrength,
    pub drag: FieldDrag,
}

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
    start: IVec2,
    goal: IVec2,
    rocks: Vec<ExportedRock>,
    fields: Vec<ExportedField>,
}
impl LevelData {}
