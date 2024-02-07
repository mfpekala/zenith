use bevy::prelude::*;

use crate::{
    input::{CameraControlState, SwitchCameraModeEvent},
    meta::game_state::{in_editor, in_level},
    physics::{move_dynos, Dyno},
};

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
    mode: CameraMode,
    vel: Vec2,
    pub zoom: f32,
}
impl CameraMarker {
    pub fn rotate(&mut self) {
        self.mode = self.mode.rotate();
        self.vel = Vec2::ZERO;
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            transform: Transform::from_xyz(100.0, 200.0, 0.0),
            ..default()
        },
        CameraMarker {
            mode: CameraMode::Follow,
            vel: Vec2::ZERO,
            zoom: 1.0,
        },
    ));
}

fn update_camera(
    dynos: Query<(&Dyno, &Transform), Without<CameraMarker>>,
    mut tran_n_marker: Query<(&mut Transform, &mut CameraMarker), Without<Dyno>>,
    mut projection: Query<&mut OrthographicProjection, With<CameraMarker>>,
    control_state: Res<CameraControlState>,
    mut switch_event: EventReader<SwitchCameraModeEvent>,
) {
    // Get the camera (do nothing if we can't find one)
    let (Ok((mut cam_tran, mut marker)), Ok(mut cam_proj)) =
        (tran_n_marker.get_single_mut(), projection.get_single_mut())
    else {
        return;
    };
    // Handle switching
    let num_switches = switch_event.read().into_iter().count();
    if num_switches % 2 == 1 {
        marker.rotate();
    }
    // Handle movement
    match marker.mode {
        CameraMode::Follow => {
            let Ok((_, dyno_tran)) = dynos.get_single() else {
                return;
            };
            cam_tran.translation = dyno_tran.translation;
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
            cam_tran.translation += marker.vel.extend(0.0);
        }
    }
    // Handle zooming
    if control_state.zoom < 0.0 {
        marker.zoom *= 1.02;
    } else if control_state.zoom > 0.0 {
        marker.zoom /= 1.02;
    }
    cam_proj.scale = marker.zoom;
}

pub fn register_camera(app: &mut App) {
    app.add_systems(Startup, setup_camera);
    app.add_systems(
        Update,
        update_camera
            .run_if(in_editor.or_else(in_level))
            .after(move_dynos),
    );
}
