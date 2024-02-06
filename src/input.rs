use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::meta::game_state::{in_editor, in_level};

#[derive(Resource, Debug)]
pub struct MouseState {
    pub pos: Option<Vec2>,
    pub pending_launch_start: Option<Vec2>,
    pub pending_launch_vel: Option<Vec2>,
}
impl MouseState {
    pub fn empty() -> Self {
        Self {
            pos: None,
            pending_launch_start: None,
            pending_launch_vel: None,
        }
    }
}

#[derive(Event, Debug)]
pub struct LaunchEvent {
    pub vel: Vec2,
}

pub fn watch_mouse(
    buttons: Res<Input<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    mut mouse_state: ResMut<MouseState>,
    mut launch_event: EventWriter<LaunchEvent>,
) {
    let Some(mouse_pos) = q_windows.single().cursor_position() else {
        // Mouse is not in the window, don't do anything
        return;
    };
    mouse_state.pos = Some(mouse_pos);
    if buttons.just_pressed(MouseButton::Left) {
        // Beginning launch
        mouse_state.pending_launch_start = Some(mouse_pos);
    }
    if buttons.pressed(MouseButton::Left) {
        if let Some(start_pos) = mouse_state.pending_launch_start {
            let mut almost_vel = (mouse_pos - start_pos) * 0.03;
            almost_vel.x *= -1.0;
            mouse_state.pending_launch_vel = Some(almost_vel);
        }
    } else {
        match mouse_state.pending_launch_vel {
            Some(vel) => {
                launch_event.send(LaunchEvent { vel });
                mouse_state.pending_launch_start = None;
                mouse_state.pending_launch_vel = None;
            }
            None => {
                // Nothing to do
            }
        }
    }
}

#[derive(Resource, Debug)]
pub struct CameraControlState {
    pub wasd_dir: Vec2,
    pub zoom: f32,
}
impl CameraControlState {
    pub fn new() -> Self {
        Self {
            wasd_dir: Vec2::ZERO,
            zoom: 0.0,
        }
    }
}

#[derive(Event, Debug)]
pub struct SwitchCameraModeEvent;

pub fn watch_camera_input(
    mut camera_control_state: ResMut<CameraControlState>,
    keys: Res<Input<KeyCode>>,
    mut switch_event: EventWriter<SwitchCameraModeEvent>,
) {
    // Movement
    let mut hor = 0.0;
    let mut ver = 0.0;
    if keys.pressed(KeyCode::A) {
        hor -= 1.0;
    }
    if keys.pressed(KeyCode::D) {
        hor += 1.0;
    }
    if keys.pressed(KeyCode::W) {
        ver += 1.0;
    }
    if keys.pressed(KeyCode::S) {
        ver -= 1.0;
    }
    let raw_dir = Vec2 { x: hor, y: ver };
    camera_control_state.wasd_dir = if raw_dir.length_squared() > 0.1 {
        raw_dir.normalize()
    } else {
        Vec2::ZERO
    };
    // Zoom
    let mut zoom = 0.0;
    if keys.pressed(KeyCode::Q) {
        zoom -= 1.0;
    }
    if keys.pressed(KeyCode::E) {
        zoom += 1.0;
    }
    camera_control_state.zoom = zoom;
    // Switch event
    if keys.just_pressed(KeyCode::Space) {
        switch_event.send(SwitchCameraModeEvent);
    }
}

pub fn register_input(app: &mut App) {
    app.insert_resource(MouseState::empty());
    app.add_event::<LaunchEvent>();
    app.add_systems(Update, watch_mouse);
    app.insert_resource(CameraControlState::new());
    app.add_event::<SwitchCameraModeEvent>();
    app.add_systems(
        Update,
        watch_camera_input.run_if(in_editor.or_else(in_level)),
    );
}
