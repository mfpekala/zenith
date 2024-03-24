use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::{
    camera::{CameraMarker, CameraMode},
    cutscenes::{is_not_in_cutscene, Cutscene},
    meta::{
        consts::{SCREEN_HEIGHT, SCREEN_WIDTH, WINDOW_WIDTH},
        game_state::{in_editor, in_level, GameState},
    },
    physics::should_apply_physics,
};

#[derive(Resource, Debug)]
pub struct MouseState {
    pub pos: IVec2,
    pub world_pos: IVec2,
    pub left_pressed: bool,
    pub pending_launch_start: Option<IVec2>,
    pub pending_launch_vel: Option<Vec2>,
}
impl MouseState {
    pub fn empty() -> Self {
        Self {
            pos: IVec2::ZERO,
            world_pos: IVec2::ZERO,
            left_pressed: false,
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
    buttons: Res<ButtonInput<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    mut mouse_state: ResMut<MouseState>,
    mut launch_event: EventWriter<LaunchEvent>,
    camera_n_tran: Query<(&Transform, &CameraMarker)>,
    gs: Res<GameState>,
    cutscene: Res<Cutscene>,
) {
    let Some(mut mouse_pos) = q_windows.single().cursor_position() else {
        // Mouse is not in the window, don't do anything
        return;
    };
    let Some((camera_tran, camera_marker)) = camera_n_tran.iter().next() else {
        // Camera not found, don't do anything
        return;
    };
    let scale_down_to_screen =
        (SCREEN_WIDTH as f32) / (WINDOW_WIDTH as f32) * camera_marker.scale.to_f32();
    mouse_state.pos = IVec2::new(mouse_pos.x.round() as i32, mouse_pos.y.round() as i32);
    mouse_pos *= scale_down_to_screen;
    let fworld_pos = camera_tran.translation.truncate()
        - Vec2 {
            x: camera_marker.scale.to_f32() * (SCREEN_WIDTH as f32 / 2.0 - mouse_pos.x),
            y: -camera_marker.scale.to_f32() * (SCREEN_HEIGHT as f32 / 2.0 - mouse_pos.y),
        };
    mouse_state.world_pos = IVec2::new(fworld_pos.x.round() as i32, fworld_pos.y.round() as i32);

    mouse_state.left_pressed = buttons.pressed(MouseButton::Left);
    if buttons.just_pressed(MouseButton::Left) {
        // Beginning launch
        mouse_state.pending_launch_start = Some(mouse_state.pos);
    }
    if buttons.pressed(MouseButton::Left) {
        if let Some(start_pos) = mouse_state.pending_launch_start {
            let mut almost_vel = (mouse_state.pos - start_pos).as_vec2();
            almost_vel.x *= -1.0;
            mouse_state.pending_launch_vel = Some(almost_vel);
        }
    } else {
        match mouse_state.pending_launch_vel {
            Some(vel) => {
                if should_apply_physics(gs) && *cutscene == Cutscene::None {
                    launch_event.send(LaunchEvent { vel });
                }
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
    pub zoom: i8,
}
impl CameraControlState {
    pub fn new() -> Self {
        Self {
            wasd_dir: Vec2::ZERO,
            zoom: 0,
        }
    }
}

#[derive(Event, Debug)]
pub struct SwitchCameraModeEvent;

#[derive(Event, Debug)]
pub struct SetCameraModeEvent {
    pub mode: CameraMode,
}

pub fn watch_camera_input(
    mut camera_control_state: ResMut<CameraControlState>,
    keys: Res<ButtonInput<KeyCode>>,
    mut switch_event: EventWriter<SwitchCameraModeEvent>,
) {
    // Movement
    let mut hor = 0.0;
    let mut ver = 0.0;
    if keys.pressed(KeyCode::KeyA) {
        hor -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) {
        hor += 1.0;
    }
    if keys.pressed(KeyCode::KeyW) {
        ver += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) {
        ver -= 1.0;
    }
    let raw_dir = Vec2 { x: hor, y: ver };
    camera_control_state.wasd_dir = if raw_dir.length_squared() > 0.1 {
        raw_dir.normalize()
    } else {
        Vec2::ZERO
    };
    // Zoom
    let mut zoom = 0;
    if keys.just_pressed(KeyCode::KeyQ) {
        zoom += 1;
    }
    if keys.just_pressed(KeyCode::KeyE) {
        zoom -= 1;
    }
    camera_control_state.zoom = zoom;
    // Switch event
    if keys.just_pressed(KeyCode::Space) {
        switch_event.send(SwitchCameraModeEvent);
    }
}

#[derive(Component, Debug)]
pub struct LongKeyPress {
    ticks: u32,
    ticks_held: u32,
    pub key_code: KeyCode,
}
impl LongKeyPress {
    pub fn new(key_code: KeyCode, length: u32) -> Self {
        Self {
            key_code,
            ticks: length,
            ticks_held: 0,
        }
    }

    /// NOTE: Consumes the state
    pub fn was_activated(&mut self) -> bool {
        if self.ticks_held > self.ticks {
            self.ticks_held = 0;
            return true;
        }
        false
    }
}

fn update_long_presses(mut lps: Query<&mut LongKeyPress>, keys: Res<ButtonInput<KeyCode>>) {
    for mut lp in lps.iter_mut() {
        if keys.pressed(lp.key_code) {
            lp.ticks_held += 1;
        } else {
            lp.ticks_held = 0;
        }
    }
}

pub fn register_input(app: &mut App) {
    app.insert_resource(MouseState::empty());
    app.add_event::<LaunchEvent>();
    app.insert_resource(CameraControlState::new());
    app.add_event::<SwitchCameraModeEvent>();
    app.add_event::<SetCameraModeEvent>();
    app.add_systems(Update, watch_mouse);
    app.add_systems(
        Update,
        watch_camera_input
            .run_if(in_editor.or_else(in_level))
            .run_if(is_not_in_cutscene),
    );
    app.add_systems(Update, update_long_presses.run_if(is_not_in_cutscene));
}
