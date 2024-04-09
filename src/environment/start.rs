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

#[derive(Component)]
pub struct StartMarker;

#[derive(Bundle)]
pub struct StartBundle {
    start: StartMarker,
    animation: AnimationStubs,
    mv: IntMoveable,
    spatial: SpatialBundle,
}
impl StartBundle {
    pub fn new(size: StartSize, pos: IVec2) -> Self {
        Self {
            start: StartMarker,
            animation: AnimationStubs(vec![size.to_animation_bundle_stub()]),
            mv: IntMoveable::new(pos.extend(0)),
            spatial: SpatialBundle::from_transform(Transform::from_translation(
                pos.as_vec2().extend(0.0),
            )),
        }
    }
}
