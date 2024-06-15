use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    drawing::animation::{AnimationManager, SpriteInfo},
    physics::dyno::IntMoveable,
};

#[derive(Clone, Debug, PartialEq, Default, Reflect, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize)]
pub enum StartSize {
    #[default]
    Medium,
}
impl StartSize {
    pub fn to_diameter(&self) -> u32 {
        match *self {
            Self::Medium => 18,
        }
    }

    pub fn to_path(&self) -> String {
        match *self {
            Self::Medium => "sprites/start_goal/start18.png".to_string(),
        }
    }

    pub fn to_sprite_info(&self) -> SpriteInfo {
        match *self {
            Self::Medium => SpriteInfo {
                path: "sprites/start_goal/start18.png".to_string(),
                size: UVec2::new(self.to_diameter(), self.to_diameter()),
                ..default()
            },
        }
    }

    pub fn to_anim_length(&self) -> u32 {
        match *self {
            Self::Medium => 10,
        }
    }
}

#[derive(Component)]
pub struct StartMarker;

#[derive(Bundle)]
pub struct StartBundle {
    start: StartMarker,
    anim: AnimationManager,
    mv: IntMoveable,
    spatial: SpatialBundle,
}
impl StartBundle {
    pub fn new(size: StartSize, pos: IVec2) -> Self {
        Self {
            start: StartMarker,
            anim: AnimationManager::single_repeating(size.to_sprite_info(), size.to_anim_length()),
            mv: IntMoveable::new(pos.extend(-1)),
            spatial: SpatialBundle::from_transform(Transform::from_translation(
                pos.as_vec2().extend(-1.0),
            )),
        }
    }
}
