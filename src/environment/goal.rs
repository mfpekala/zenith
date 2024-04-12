use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    drawing::animation::{AnimationManager, SpriteInfo},
    physics::dyno::IntMoveable,
};

#[derive(Clone, Debug, PartialEq, Default, Reflect, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize)]
pub enum GoalSize {
    #[default]
    Medium,
}
impl GoalSize {
    pub fn to_diameter(&self) -> u32 {
        match *self {
            Self::Medium => 18,
        }
    }

    pub fn to_sprite_info(&self) -> SpriteInfo {
        match *self {
            Self::Medium => SpriteInfo {
                path: "sprites/start_goal/goal18.png".to_string(),
                size: UVec2::new(self.to_diameter(), self.to_diameter()),
            },
        }
    }

    pub fn to_anim_length(&self) -> u32 {
        match *self {
            Self::Medium => 10,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default, Reflect, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize)]
pub enum GoalStrength {
    #[default]
    Medium,
}
impl GoalStrength {
    pub fn to_f32(&self) -> f32 {
        match *self {
            Self::Medium => 1.0,
        }
    }
}

#[derive(Component)]
pub struct GoalMarker;

#[derive(Bundle)]
pub struct GoalBundle {
    goal: GoalMarker,
    anim: AnimationManager,
    mv: IntMoveable,
    spatial: SpatialBundle,
}
impl GoalBundle {
    pub fn new(size: GoalSize, pos: IVec2) -> Self {
        Self {
            goal: GoalMarker,
            anim: AnimationManager::single_repeating(size.to_sprite_info(), size.to_anim_length()),
            mv: IntMoveable::new(pos.extend(-1)),
            spatial: SpatialBundle::from_transform(Transform::from_translation(
                pos.as_vec2().extend(-1.0),
            )),
        }
    }
}
