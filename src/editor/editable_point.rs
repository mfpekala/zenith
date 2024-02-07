use super::{draggable::Draggable, is_editing};
use crate::{drawing::CircleMarker, input::MouseState};
use bevy::prelude::*;

#[derive(Component)]
pub struct EditablePoint;

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
            editable_point: EditablePoint,
            draggable: Draggable {
                is_dragging: false,
                knob_size,
            },
            circle: CircleMarker {
                radius: knob_size,
                color: Color::ANTIQUE_WHITE,
                shown: true,
            },
            spatial: SpatialBundle::from_transform(Transform::from_translation(pos.extend(0.0))),
        }
    }
}

pub fn create_and_destroy_points(
    mouse_state: Res<MouseState>,
    mouse_buttons: Res<Input<MouseButton>>,
    mut commands: Commands,
    existing_points: Query<(Entity, &Transform, &Draggable), With<EditablePoint>>,
) {
    // Create new points
    if mouse_buttons.just_pressed(MouseButton::Right) {
        commands.spawn(EditablePointBundle::new(mouse_state.world_pos));
    }
    // Destroy points
    if mouse_buttons.just_pressed(MouseButton::Middle) {
        for (id, tran, draggable) in existing_points.iter() {
            if tran.translation.truncate().distance(mouse_state.world_pos) < draggable.knob_size {
                commands.entity(id).despawn_recursive();
            }
        }
    }
}

pub fn register_editable_points(app: &mut App) {
    app.add_systems(Update, create_and_destroy_points.run_if(is_editing));
}
