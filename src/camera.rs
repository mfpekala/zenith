use crate::{
    drawing::{
        lightmap::{LightCameraMarker, LightmapPlugin, SpriteCameraMarker},
        post_pixel::PostPixelPlugin,
    },
    input::{CameraControlState, SetCameraModeEvent, SwitchCameraModeEvent},
    meta::game_state::{in_editor, in_level},
    physics::{dyno::IntDyno, move_dynos},
};
use bevy::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CameraMode {
    Follow,
    Free,
}
impl CameraMode {
    pub fn rotate(&self) -> Self {
        match *self {
            CameraMode::Follow => CameraMode::Free,
            CameraMode::Free => CameraMode::Follow,
        }
    }
}

#[derive(Component)]
pub struct CameraMarker {
    pub mode: CameraMode,
    pub vel: Vec2,
    pub pos: IVec2,
    pub zoom: f32,
}
impl CameraMarker {
    pub fn new() -> Self {
        Self {
            mode: CameraMode::Follow,
            vel: Vec2::ZERO,
            pos: IVec2::ZERO,
            zoom: 1.0,
        }
    }

    pub fn rotate(&mut self) {
        self.mode = self.mode.rotate();
        self.vel = Vec2::ZERO;
    }
}

pub fn update_camera(
    dynos: Query<&IntDyno, Without<CameraMarker>>,
    mut marker: Query<&mut CameraMarker>,
    control_state: Res<CameraControlState>,
    mut switch_event: EventReader<SwitchCameraModeEvent>,
    mut set_event: EventReader<SetCameraModeEvent>,
    mut light_camera: Query<
        (&mut Transform, &mut OrthographicProjection),
        (With<LightCameraMarker>, Without<SpriteCameraMarker>),
    >,
    mut sprite_camera: Query<
        (&mut Transform, &mut OrthographicProjection),
        (With<SpriteCameraMarker>, Without<LightCameraMarker>),
    >,
) {
    // Get the camera (do nothing if we can't find one)
    let Ok(mut marker) = marker.get_single_mut() else {
        return;
    };
    // Handle switching
    let num_switches = switch_event.read().into_iter().count();
    if num_switches % 2 == 1 {
        marker.rotate();
    }
    // Handle setting
    if let Some(set_event) = set_event.read().last() {
        marker.mode = set_event.mode.clone();
    }
    // Handle movement
    match marker.mode {
        CameraMode::Follow => {
            let Ok(dyno) = dynos.get_single() else {
                return;
            };
            marker.pos = dyno.pos;
        }
        CameraMode::Free => {
            if control_state.wasd_dir.length_squared() < 0.1 {
                // Slow to a stop
                marker.vel *= 0.89;
            } else {
                // Move around
                let max_speed = 10.0;
                marker.vel += control_state.wasd_dir * 0.5;
                if marker.vel.length_squared() > max_speed * max_speed {
                    marker.vel = marker.vel.normalize() * max_speed;
                }
            }
            let vec = marker.vel;
            marker.pos = IVec2 {
                x: (marker.pos.x as f32 + vec.x) as i32,
                y: (marker.pos.y as f32 + vec.y) as i32,
            };
        }
    }
    // Handle zooming
    if control_state.zoom < 0.0 {
        marker.zoom *= 1.02;
    } else if control_state.zoom > 0.0 {
        marker.zoom /= 1.02;
    }
    let (lc_tran, lc_proj) = light_camera.single_mut();
    let (sc_tran, sc_proj) = sprite_camera.single_mut();
    for tran in [lc_tran, sc_tran].iter_mut() {
        tran.translation = marker.pos.as_vec2().extend(0.0);
    }
    for proj in [lc_proj, sc_proj].iter_mut() {
        proj.scale = marker.zoom;
    }
}

pub fn register_camera(app: &mut App) {
    app.add_plugins(LightmapPlugin);
    app.add_plugins(PostPixelPlugin);
    app.add_systems(
        Update,
        update_camera
            .run_if(in_editor.or_else(in_level))
            .after(move_dynos),
    );
}
