use super::{editable_rock::EditableRock, is_editing, state_machine::editing_state_machine};
use crate::{drawing::hollow::CircleMarker, input::MouseState};
use bevy::prelude::*;

#[derive(Component)]
pub struct Draggable {
    pub enabled: bool,
    pub is_dragging: bool,
    pub knob_size: f32,
}
impl Draggable {
    pub fn new(knob_size: f32) -> Self {
        Self {
            enabled: true,
            is_dragging: false,
            knob_size,
        }
    }

    pub fn is_mouse_over(&self, pos: Vec2, mouse_state: &MouseState) -> bool {
        mouse_state.world_pos.distance_squared(pos) < self.knob_size * self.knob_size
    }
}

pub fn handle_draggables(
    mouse_state: Res<MouseState>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut draggables: Query<(
        &mut Draggable,
        &mut Transform,
        Option<&mut CircleMarker>,
        Option<&EditableRock>,
    )>,
) {
    for (mut draggable, mut tran, cm, erock) in draggables.iter_mut() {
        // First set the circle size
        if let Some(mut cm) = cm {
            cm.radius = if draggable.enabled {
                draggable.knob_size
            } else {
                draggable.knob_size / 2.0
            };
        }
        // If it's not draggable we're done
        if !draggable.enabled {
            continue;
        }
        // Do the dragging if it's enabled
        if draggable.is_dragging {
            if mouse_state.left_pressed {
                // Dragging for editable rock centers is handled elsewhere
                if erock.is_some() {
                    return;
                }
                tran.translation = mouse_state.world_pos.extend(0.0).round();
                // We only want to drag one thing at a time, so return early
                return;
            }
            draggable.is_dragging = false;
        } else {
            if mouse_buttons.just_pressed(MouseButton::Left)
                && draggable.is_mouse_over(tran.translation.truncate(), &mouse_state)
            {
                draggable.is_dragging = true;
                // We only want to drag one thing at a time, so return early
                return;
            }
        }
    }
}

pub fn register_draggables(app: &mut App) {
    app.add_systems(
        Update,
        handle_draggables
            .run_if(is_editing)
            .after(editing_state_machine),
    );
}
