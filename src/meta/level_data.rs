use bevy::{prelude::*, utils::HashMap};

use crate::environment::{
    field::{FieldDrag, FieldStrength},
    rock::RockKind,
};

#[derive(
    serde::Serialize, serde::Deserialize, bevy::reflect::TypePath, Debug, PartialEq, Clone, Default,
)]
pub struct ExportedRock {
    pub points: Vec<u64>,
    pub kind: RockKind,
}

#[derive(
    serde::Serialize, serde::Deserialize, bevy::reflect::TypePath, Debug, PartialEq, Clone, Default,
)]
pub struct ExportedField {
    pub points: Vec<u64>,
    pub dir: Vec2,
    pub strength: FieldStrength,
    pub drag: FieldDrag,
}

/// A representation of a level. Note that this functions both as saves of
/// editor state, and what is used when loading a level to play
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
    pub points: HashMap<u64, IVec2>,
    pub start: u64,
    pub goal: u64,
    pub rocks: Vec<ExportedRock>,
    pub fields: Vec<ExportedField>,
    pub replenishes: Vec<u64>,
}
