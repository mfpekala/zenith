use crate::{
    drawing::{
        layering::{LayeringPlugin, LightCameraMarker, SpriteCameraMarker},
        post_pixel::PostPixelPlugin,
    },
    input::{CameraControlState, SetCameraModeEvent, SwitchCameraModeEvent},
    meta::{
        consts::{SCREEN_HEIGHT, SCREEN_WIDTH},
        game_state::{in_editor, in_level},
    },
    physics::dyno::{move_int_dynos, IntDyno, IntMoveable},
};
use bevy::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CameraScale {
    Quarter,
    Half,
    One,
    Two,
    Three,
}
impl CameraScale {
    pub fn to_f32(&self) -> f32 {
        match *self {
            CameraScale::Quarter => 0.25,
            CameraScale::Half => 0.5,
            CameraScale::One => 1.0,
            CameraScale::Two => 2.0,
            CameraScale::Three => 3.0,
        }
    }

    pub fn up(&self) -> Self {
        match *self {
            CameraScale::Quarter => CameraScale::Half,
            CameraScale::Half => CameraScale::One,
            CameraScale::One => CameraScale::Two,
            CameraScale::Two => CameraScale::Three,
            CameraScale::Three => CameraScale::Three,
        }
    }

    pub fn down(&self) -> Self {
        match *self {
            CameraScale::Quarter => CameraScale::Quarter,
            CameraScale::Half => CameraScale::Quarter,
            CameraScale::One => CameraScale::Half,
            CameraScale::Two => CameraScale::One,
            CameraScale::Three => CameraScale::Two,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CameraMode {
    Follow,
    Free,
    Controlled,
}
impl CameraMode {
    pub fn rotate(&self) -> Self {
        match *self {
            CameraMode::Follow => CameraMode::Free,
            CameraMode::Free => CameraMode::Follow,
            CameraMode::Controlled => {
                warn!("Tried to rotate controlled camera. This shouldn't happen.");
                CameraMode::Controlled
            }
        }
    }
}

#[derive(Component, Debug)]
pub struct CameraMarker {
    pub mode: CameraMode,
    pub scale: CameraScale,
}
impl CameraMarker {
    pub fn new() -> Self {
        Self {
            mode: CameraMode::Follow,
            scale: CameraScale::One,
        }
    }

    pub fn rotate(&mut self) {
        self.mode = self.mode.rotate();
    }
}

#[derive(Bundle)]
pub struct DynamicCameraBundle {
    pub marker: CameraMarker,
    pub moveable: IntMoveable,
    pub spatial: SpatialBundle,
}

pub fn update_camera(
    dynos: Query<&GlobalTransform, (With<IntDyno>, Without<CameraMarker>)>,
    mut marker: Query<(&mut IntMoveable, &mut CameraMarker)>,
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
    let Ok((mut moveable, mut marker)) = marker.get_single_mut() else {
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
            let ipos = dyno.translation().truncate();
            let ipos = IVec2::new(ipos.x.round() as i32, ipos.y.round() as i32);
            let diff = ipos - moveable.pos.truncate();
            let hor_wiggle = 8;
            if diff.x.abs() > SCREEN_WIDTH as i32 / hor_wiggle {
                moveable.pos.x +=
                    (diff.x.abs() - SCREEN_WIDTH as i32 / hor_wiggle) * diff.x.signum();
            }
            let ver_wiggle = 8;
            if diff.y.abs() > SCREEN_HEIGHT as i32 / ver_wiggle {
                moveable.pos.y +=
                    (diff.y.abs() - SCREEN_HEIGHT as i32 / ver_wiggle) * diff.y.signum();
            }
        }
        CameraMode::Free => {
            if control_state.wasd_dir.length_squared() < 0.1 {
                // Slow to a stop
                moveable.vel *= 0.7;
            } else {
                // Move around
                let max_speed = 10.0 * marker.scale.to_f32();
                moveable.vel += control_state.wasd_dir * 0.5 * marker.scale.to_f32();
                if moveable.vel.length_squared() > max_speed * max_speed {
                    moveable.vel = moveable.vel.normalize() * max_speed;
                }
            }
        }
        CameraMode::Controlled => {
            // Do nothing, something else is driving us
        }
    }
    // Handle zoom
    if control_state.zoom > 0 {
        marker.scale = marker.scale.up();
    } else if control_state.zoom < 0 {
        marker.scale = marker.scale.down();
    }
    // Handle moving the "actual" cameras
    let (lc_tran, lc_proj) = light_camera.single_mut();
    let (sc_tran, sc_proj) = sprite_camera.single_mut();
    for tran in [lc_tran, sc_tran].iter_mut() {
        tran.translation = moveable.pos.truncate().as_vec2().extend(0.0);
    }
    for proj in [lc_proj, sc_proj].iter_mut() {
        proj.scale = marker.scale.to_f32();
    }
}

pub fn register_camera(app: &mut App) {
    app.add_plugins(LayeringPlugin);
    app.add_plugins(PostPixelPlugin);
    app.add_systems(
        Update,
        update_camera
            .run_if(in_editor.or_else(in_level))
            // .run_if(is_not_in_cutscene)
            .after(move_int_dynos),
    );
}
