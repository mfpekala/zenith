use crate::{
    drawing::layering::{LayeringPlugin, LightCameraMarker, SpriteCameraMarker},
    input::{CameraControlState, CameraZoomEvent, SetCameraModeEvent, SwitchCameraModeEvent},
    math::Spleen,
    meta::{
        consts::{SCREEN_HEIGHT, SCREEN_WIDTH},
        game_state::{in_editor, in_level},
        level_data::LevelRoot,
    },
    physics::dyno::{apply_fields, IntDyno, IntMoveable},
    ship::Ship,
};
use bevy::prelude::*;

/// Need a way to track how many multiples of the screen there are so that the distance dragged
/// for mouse launches can be correctly adjusted
#[derive(Resource)]
pub struct ScreenMults(pub u32);

#[derive(Resource)]
pub struct WindowDims(pub UVec2);

#[derive(Debug, Clone, PartialEq, Eq, Reflect)]
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

#[derive(Debug, Clone, PartialEq, Eq, Reflect)]
pub struct Dislodgment {
    start_pos: IVec2,
    timer: Timer,
    spleen: Spleen,
}

#[derive(Debug, Clone, PartialEq, Eq, Reflect)]
pub enum CameraMode {
    Follow { dislodgement: Option<Dislodgment> },
    Free,
    Controlled,
}
impl CameraMode {
    pub fn rotate(&self, dislodgement: Option<Dislodgment>) -> Self {
        match *self {
            CameraMode::Follow { .. } => CameraMode::Free,
            CameraMode::Free => CameraMode::Follow { dislodgement },
            CameraMode::Controlled => {
                warn!("Tried to rotate controlled camera. This shouldn't happen.");
                CameraMode::Controlled
            }
        }
    }
}

#[derive(Component, Clone, Debug, Reflect)]
pub struct CameraMarker {
    pub mode: CameraMode,
    pub scale: CameraScale,
}
impl CameraMarker {
    pub fn new() -> Self {
        Self {
            mode: CameraMode::Follow { dislodgement: None },
            scale: CameraScale::One,
        }
    }

    pub fn get_dislodgement(start_pos: IVec2, end_pos: IVec2) -> Dislodgment {
        let dist = (start_pos.distance_squared(end_pos) as f32).sqrt().sqrt();
        Dislodgment {
            start_pos,
            timer: Timer::from_seconds(dist / 75.0, TimerMode::Once),
            spleen: Spleen::EaseInOutCubic,
        }
    }

    pub fn rotate(&mut self, dislodge_calc: Option<(IVec2, IVec2)>) {
        let dislodgement =
            dislodge_calc.map(|(start_pos, end_pos)| Self::get_dislodgement(start_pos, end_pos));
        self.mode = self.mode.rotate(dislodgement);
    }
}

#[derive(Bundle)]
pub struct DynamicCameraBundle {
    pub marker: CameraMarker,
    pub moveable: IntMoveable,
    pub spatial: SpatialBundle,
    pub name: Name,
}

/// Manage input for controlling the camera in the Update system
pub(super) fn camera_input(
    mut marker: Query<(&mut CameraMarker, &IntMoveable)>,
    mut switch_event: EventReader<SwitchCameraModeEvent>,
    mut set_event: EventReader<SetCameraModeEvent>,
    ships: Query<&IntDyno, With<Ship>>,
) {
    // Get the camera (do nothing if we can't find one)
    let Ok((mut marker, mv)) = marker.get_single_mut() else {
        return;
    };
    let ship = ships.iter().next();
    // Handle switching
    let num_switches = switch_event.read().into_iter().count();
    if num_switches % 2 == 1 {
        marker.rotate(ship.map(|ship_mv| {
            let ship_ipos =
                IVec2::new(ship_mv.fpos.x.round() as i32, ship_mv.fpos.y.round() as i32);
            (mv.pos.truncate(), ship_ipos)
        }));
    }
    // Handle setting
    if let Some(set_event) = set_event.read().last() {
        marker.mode = set_event.mode.clone();
    }
}

pub fn camera_movement(
    dynos: Query<&IntDyno, Without<CameraMarker>>,
    level_root: Query<&GlobalTransform, With<LevelRoot>>,
    mut marker: Query<(&mut IntMoveable, &mut CameraMarker)>,
    control_state: Res<CameraControlState>,
    mut zooms: EventReader<CameraZoomEvent>,
    mut light_camera: Query<
        (&mut Transform, &mut OrthographicProjection),
        (With<LightCameraMarker>, Without<SpriteCameraMarker>),
    >,
    mut sprite_camera: Query<
        (&mut Transform, &mut OrthographicProjection),
        (With<SpriteCameraMarker>, Without<LightCameraMarker>),
    >,
    time: Res<Time>,
) {
    // Get the camera (do nothing if we can't find one)
    let Ok((mut moveable, mut marker)) = marker.get_single_mut() else {
        return;
    };
    // Update the last pos (cursed vel calc for bg ents)
    // Handle movement
    match &mut marker.mode {
        CameraMode::Follow { dislodgement } => {
            moveable.vel = Vec2::ZERO;
            let Ok(dyno) = dynos.get_single() else {
                return;
            };
            let Ok(root) = level_root.get_single() else {
                return;
            };
            let end_dislodgement = match dislodgement {
                Some(dislodgement) => {
                    // We are trying to follow the ship but just started doing so
                    // The camera should move over smoothly so it looks better
                    let adjusted_start_pos =
                        dislodgement.start_pos.as_vec2() + root.translation().truncate();
                    let adjusted_goal_pos =
                        dyno.ipos.truncate().as_vec2() + root.translation().truncate();
                    dislodgement.timer.tick(time.delta());
                    let pos = adjusted_start_pos
                        + dislodgement.spleen.interp(dislodgement.timer.fraction())
                            * (adjusted_goal_pos - adjusted_start_pos);
                    moveable.pos.x = pos.x.round() as i32;
                    moveable.pos.y = pos.y.round() as i32;
                    dislodgement.timer.finished()
                }
                None => {
                    moveable.pos.x = root.translation().x.round() as i32 + dyno.ipos.x;
                    moveable.pos.y = root.translation().y.round() as i32 + dyno.ipos.y;
                    false
                }
            };
            if end_dislodgement {
                *dislodgement = None;
            }
        }
        CameraMode::Free => {
            if control_state.wasd_dir.length_squared() < 0.1 {
                // Slow to a stop
                moveable.vel *= 0.6;
                if moveable.vel.length_squared() < 0.1 {
                    moveable.vel = Vec2::ZERO;
                }
            } else {
                // Move around
                let max_speed = 16.0 * marker.scale.to_f32();
                moveable.vel += control_state.wasd_dir * marker.scale.to_f32();
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
    let mut zoom_total: i32 = 0;
    for zoom_ev in zooms.read() {
        zoom_total += zoom_ev.0;
    }
    if zoom_total > 0 {
        marker.scale = marker.scale.up();
    } else if zoom_total < 0 {
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
    app.insert_resource(ScreenMults(1));
    app.insert_resource(WindowDims(UVec2::new(
        SCREEN_WIDTH as u32,
        SCREEN_HEIGHT as u32,
    )));
    app.register_type::<CameraMarker>();
    app.add_systems(Update, camera_input.run_if(in_editor.or_else(in_level)));
    app.add_systems(FixedUpdate, camera_movement.after(apply_fields));
}
