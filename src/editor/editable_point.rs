use super::{draggable::Draggable, is_editing};
use crate::{drawing::CircleMarker, input::MouseState};
use bevy::prelude::*;

#[derive(Component)]
pub struct EditablePoint {
    pub is_focused: bool,
}

#[derive(Bundle)]
pub struct EditablePointBundle {
    pub editable_point: EditablePoint,
    pub draggable: Draggable,
    pub circle: CircleMarker,
    pub spatial: SpatialBundle,
}
impl EditablePointBundle {
    pub fn new(pos: Vec2) -> Self {
        let knob_size = 10.0;
        Self {
            editable_point: EditablePoint { is_focused: true },
            draggable: Draggable::new(knob_size),
            circle: CircleMarker {
                radius: knob_size,
                color: Color::ANTIQUE_WHITE,
                shown: true,
            },
            spatial: SpatialBundle::from_transform(Transform::from_translation(pos.extend(0.0))),
        }
    }
}

pub fn destroy_points(
    mouse_state: Res<MouseState>,
    mouse_buttons: Res<Input<MouseButton>>,
    mut commands: Commands,
    existing_points: Query<(Entity, &Transform, &Draggable), With<EditablePoint>>,
) {
    if mouse_buttons.just_pressed(MouseButton::Middle) {
        for (id, tran, draggable) in existing_points.iter() {
            if !draggable.enabled {
                continue;
            }
            if draggable.is_mouse_over(tran.translation.truncate(), &mouse_state) {
                commands.entity(id).despawn_recursive();
            }
        }
    }
}

pub fn register_editable_points(app: &mut App) {
    app.add_systems(Update, destroy_points.run_if(is_editing));
}
