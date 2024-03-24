use super::draggable::Draggable;
use crate::drawing::hollow::EquiTriangleMarker;
use bevy::prelude::*;

#[derive(Component)]
pub struct EditableGoal;

#[derive(Bundle)]
pub struct EditableGoalBundle {
    pub egoal: EditableGoal,
    pub tri: EquiTriangleMarker,
    pub draggable: Draggable,
    pub spatial: SpatialBundle,
}
impl EditableGoalBundle {
    pub fn new(pos: Vec2) -> Self {
        Self {
            egoal: EditableGoal,
            tri: EquiTriangleMarker::new(10.0, Color::PURPLE),
            draggable: Draggable::new(10.0),
            spatial: SpatialBundle::from_transform(Transform::from_translation(pos.extend(0.0))),
        }
    }
}
