use super::{
    draggable::Draggable,
    editable_point::{EditablePoint, EditablePointBundle},
};
use crate::drawing::CircleMarker;
use bevy::prelude::*;

#[derive(Component)]
pub struct EditableRock {
    pub closed: bool,
    pub gravity_strength: Option<f32>,
    pub gravity_reach: Option<f32>,
    pub points: Vec<Entity>,
}

#[derive(Bundle)]
pub struct EditableRockBundle {
    pub erock: EditableRock,
    pub editable_point: EditablePoint,
    pub draggable: Draggable,
    pub circle: CircleMarker,
    pub spatial: SpatialBundle,
}
impl EditableRockBundle {
    pub fn from_single_point(id: Entity, pos: Vec2) -> Self {
        Self {
            erock: EditableRock {
                closed: false,
                gravity_reach: None,
                gravity_strength: None,
                points: vec![id],
            },
            editable_point: EditablePoint { is_focused: true },
            draggable: Draggable::new(10.0),
            circle: CircleMarker::new(10.0, Color::SEA_GREEN),
            spatial: SpatialBundle::from_transform(Transform::from_translation(pos.extend(0.0))),
        }
    }
}
