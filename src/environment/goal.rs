use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    drawing::{
        animated::{AnimationStub, AnimationStubs},
        layering::sprite_layer_u8,
    },
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

    pub fn to_path(&self) -> String {
        match *self {
            Self::Medium => "sprites/start_goal/goal18.png".to_string(),
        }
    }

    pub fn length(&self) -> u8 {
        match *self {
            Self::Medium => 10,
        }
    }

    pub fn to_animation_bundle_stub(&self) -> AnimationStub {
        let size = UVec2::new(self.to_diameter(), self.to_diameter());
        AnimationStub::single_repeating(
            "shrinking",
            &self.to_path(),
            size,
            self.length(),
            None,
            sprite_layer_u8(),
        )
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
    animation: AnimationStubs,
    mv: IntMoveable,
    spatial: SpatialBundle,
}
impl GoalBundle {
    pub fn new(size: GoalSize, pos: IVec2) -> Self {
        Self {
            goal: GoalMarker,
            animation: AnimationStubs(vec![size.to_animation_bundle_stub()]),
            mv: IntMoveable::new(pos.extend(-1)),
            spatial: SpatialBundle::from_transform(Transform::from_translation(
                pos.as_vec2().extend(-1.0),
            )),
        }
    }
}
