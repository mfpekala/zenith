use crate::{
    drawing::{
        lightmap::{LightCameraMarker, LightmapPlugin, SpriteCameraMarker},
        post_pixel::PostPixelPlugin,
    },
    input::{CameraControlState, SetCameraModeEvent, SwitchCameraModeEvent},
    meta::{
        consts::PIXEL_WIDTH,
        game_state::{in_editor, in_level},
    },
    physics::{move_dynos, Dyno},
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
    pub fake_pos: Vec2,
    vel: Vec2,
    pub zoom: f32,
}
impl CameraMarker {
    pub fn new() -> Self {
        Self {
            mode: CameraMode::Follow,
            vel: Vec2::ZERO,
            fake_pos: Vec2::ZERO,
            zoom: 1.0,
        }
    }

    pub fn rotate(&mut self) {
        self.mode = self.mode.rotate();
        self.vel = Vec2::ZERO;
    }

    pub fn pixel_align(&self, pos: Vec2) -> Vec2 {
        let modulo = PIXEL_WIDTH as f32 * self.zoom;
        let mut initial = Vec2 {
            x: pos.x - pos.x.rem_euclid(PIXEL_WIDTH as f32 * self.zoom),
            y: pos.y - pos.y.rem_euclid(PIXEL_WIDTH as f32 * self.zoom),
        };
        if initial.x < 0.0 - modulo / 2.0 {
            initial.x += modulo;
        }
        if initial.y < 0.0 - modulo / 2.0 {
            initial.y += modulo;
        }
        initial
    }
}

pub fn update_camera(
    dynos: Query<(&Dyno, &Transform), Without<CameraMarker>>,
    mut marker: Query<&mut CameraMarker, Without<Dyno>>,
    control_state: Res<CameraControlState>,
    mut switch_event: EventReader<SwitchCameraModeEvent>,
    mut set_event: EventReader<SetCameraModeEvent>,
    mut light_camera: Query<
        (&mut Transform, &mut OrthographicProjection),
        (
            With<LightCameraMarker>,
            Without<SpriteCameraMarker>,
            Without<CameraMarker>,
            Without<Dyno>,
        ),
    >,
    mut sprite_camera: Query<
        (&mut Transform, &mut OrthographicProjection),
        (
            With<SpriteCameraMarker>,
            Without<LightCameraMarker>,
            Without<CameraMarker>,
            Without<Dyno>,
        ),
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
            let Ok((_, dyno_tran)) = dynos.get_single() else {
                return;
            };
            marker.fake_pos = dyno_tran.translation.truncate();
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
            marker.fake_pos += vec;
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
        tran.translation = marker.pixel_align(marker.fake_pos).extend(0.0);
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
