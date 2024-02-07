use super::is_editing;
use crate::input::MouseState;
use bevy::prelude::*;

#[derive(Component)]
pub struct Draggable {
    pub is_dragging: bool,
    pub knob_size: f32,
}
impl Draggable {
    pub fn new(knob_size: f32) -> Self {
        Self {
            is_dragging: false,
            knob_size,
        }
    }
}

fn handle_draggables(
    mouse_state: Res<MouseState>,
    mouse_buttons: Res<Input<MouseButton>>,
    mut draggables: Query<(&mut Draggable, &mut Transform)>,
) {
    for (mut draggable, mut tran) in draggables.iter_mut() {
        if draggable.is_dragging {
            if mouse_state.left_pressed {
                tran.translation = mouse_state.world_pos.extend(0.0);
            } else {
                draggable.is_dragging = false;
            }
        } else {
            if mouse_buttons.just_pressed(MouseButton::Left)
                && mouse_state
                    .world_pos
                    .distance_squared(tran.translation.truncate())
                    < draggable.knob_size * draggable.knob_size
            {
                draggable.is_dragging = true;
            }
        }
    }
}

pub fn register_draggables(app: &mut App) {
    app.add_systems(Update, handle_draggables.run_if(is_editing));
}
