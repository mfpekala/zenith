use crate::drawing::hollow::ShrinkingCircleBundle;
use bevy::prelude::*;

#[derive(Component)]
pub struct Goal {
    pub radius: f32,
    pub strength: f32,
}

#[derive(Bundle)]
pub struct GoalBundle {
    pub goal: Goal,
    pub shrink: ShrinkingCircleBundle,
    pub spatial: SpatialBundle,
}
impl GoalBundle {
    pub fn new(pos: Vec2, radius: f32, strength: f32) -> Self {
        Self {
            goal: Goal { radius, strength },
            shrink: ShrinkingCircleBundle::new(radius, strength * 2.0),
            spatial: SpatialBundle::from_transform(Transform::from_translation(pos.extend(0.0))),
        }
    }
}
