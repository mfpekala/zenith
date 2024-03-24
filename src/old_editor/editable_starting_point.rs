use super::draggable::Draggable;
use crate::drawing::hollow::EquiTriangleMarker;
use bevy::prelude::*;

#[derive(Component)]
pub struct EditableStartingPoint;

#[derive(Bundle)]
pub struct EditableStartingPointBundle {
    pub estart: EditableStartingPoint,
    pub tri: EquiTriangleMarker,
    pub draggable: Draggable,
    pub spatial: SpatialBundle,
}
impl EditableStartingPointBundle {
    pub fn new(pos: Vec2) -> Self {
        Self {
            estart: EditableStartingPoint,
            tri: EquiTriangleMarker::new(10.0, Color::PINK),
            draggable: Draggable::new(10.0),
            spatial: SpatialBundle::from_transform(Transform::from_translation(pos.extend(0.0))),
        }
    }
}
