use crate::drawing::hollow::EquiTriangleMarker;
use bevy::prelude::*;

#[derive(Component)]
pub struct StartingPoint;

#[derive(Bundle)]
pub struct StartingPointBundle {
    pub tri: EquiTriangleMarker,
    pub spatial: SpatialBundle,
}
impl StartingPointBundle {
    pub fn new(pos: Vec2) -> Self {
        Self {
            tri: EquiTriangleMarker::new(10.0, Color::PINK),
            spatial: SpatialBundle::from_transform(Transform::from_translation(pos.extend(0.0))),
        }
    }
}
