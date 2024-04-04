use bevy::prelude::*;

#[derive(Component)]
pub struct StartingPoint;

#[derive(Bundle)]
pub struct StartingPointBundle {
    pub spatial: SpatialBundle,
}
impl StartingPointBundle {
    pub fn new(pos: Vec2) -> Self {
        Self {
            spatial: SpatialBundle::from_transform(Transform::from_translation(pos.extend(0.0))),
        }
    }
}
