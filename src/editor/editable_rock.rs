use super::draggable::Draggable;
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
    pub draggable: Draggable,
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
            draggable: Draggable::new(10.0),
            spatial: SpatialBundle::from_transform(Transform::from_translation(pos.extend(0.0))),
        }
    }
}
