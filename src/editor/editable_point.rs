use super::{draggable::Draggable, editable_rock::EditableRock, is_editing};
use crate::{drawing::hollow::CircleMarker, input::MouseState};
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
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    existing_points: Query<(Entity, &Transform, &Draggable), With<EditablePoint>>,
    mut erocks: Query<(Entity, &mut EditableRock)>,
) {
    if mouse_buttons.just_pressed(MouseButton::Middle) {
        for (id, tran, draggable) in existing_points.iter() {
            if !draggable.enabled {
                continue;
            }
            if draggable.is_mouse_over(tran.translation.truncate(), &mouse_state) {
                // If we are deleting a rock, despawn it
                if let Ok((eid, mut erock)) = erocks.get_mut(id) {
                    erock.despawn(eid, &mut commands);
                    return;
                }
                // Otherwise, find the rock that this is associated with and clean it up
                for (eid, mut erock) in erocks.iter_mut() {
                    if erock.points.iter().any(|pid| *pid == id)
                        || erock.gravity_reach_point == Some(id)
                    {
                        erock.points.retain(|pid| *pid != id);
                        if erock.gravity_reach_point == Some(id) {
                            erock.gravity_reach_point = None;
                        }
                        if erock.points.len() < 3 {
                            erock.despawn(eid, &mut commands);
                            commands.entity(id).despawn_recursive();
                            return;
                        }
                    }
                }
                commands.entity(id).despawn_recursive();
            }
        }
    }
}

pub fn register_editable_points(app: &mut App) {
    app.add_systems(Update, destroy_points.run_if(is_editing));
}
