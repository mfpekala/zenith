use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::{
    camera::{camera_movement, CameraMarker, CameraMode, ScreenMults, WindowDims},
    cutscenes::is_not_in_cutscene,
    drawing::{
        animation::{AnimationManager, MultiAnimationManager, SpriteInfo},
        layering::menu_layer_u8,
    },
    environment::convo::Convo,
    meta::{
        consts::{MENU_GROWTH, SCREEN_WIDTH},
        game_state::{in_editor, in_level, GameState},
    },
    physics::should_apply_physics,
    ship::Ship,
};

#[derive(Debug)]
pub struct PendingLaunch {
    pub timer: Option<Timer>,
    pub launch_start: IVec2,
    pub launch_vel: Vec2,
}

#[derive(Resource, Debug)]
pub struct MouseState {
    pub pos: IVec2,
    pub world_pos: IVec2,
    pub button_input: ButtonInput<MouseButton>,
    pub pending_launch: Option<PendingLaunch>,
}
impl MouseState {
    pub fn empty() -> Self {
        Self {
            pos: IVec2::ZERO,
            world_pos: IVec2::ZERO,
            button_input: default(),
            pending_launch: None,
        }
    }
}

#[derive(Event, Debug)]
pub struct LaunchEvent {
    pub vel: Vec2,
}

const MULT_THINGY: f32 = 0.32;
pub fn watch_mouse(
    buttons: Res<ButtonInput<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    mut mouse_state: ResMut<MouseState>,
    mut launch_event: EventWriter<LaunchEvent>,
    camera_n_tran: Query<(&Transform, &CameraMarker)>,
    ships: Query<&Ship>,
    time: Res<Time>,
    screen_mults: Res<ScreenMults>,
    window_dims: Res<WindowDims>,
    gs: Res<GameState>,
    convos: Query<&Convo>,
) {
    let can_shoot = ships.iter().all(|ship| ship.can_shoot);
    // Helper function to terminate a launch
    let send_launch = |mouse_state: &mut MouseState,
                       launch_event: &mut EventWriter<LaunchEvent>| {
        let pending = mouse_state.pending_launch.take().unwrap();
        launch_event.send(LaunchEvent {
            vel: pending.launch_vel,
        });
    };
    if should_apply_physics(gs, convos) {
        // Update the timer for the launch
        let has_timer = match mouse_state.pending_launch.as_ref() {
            Some(pending) => pending.timer.is_some(),
            None => false,
        };
        if has_timer {
            let did_timer_expire = match mouse_state.pending_launch.as_mut() {
                Some(pending) => {
                    pending.timer.as_mut().unwrap().tick(time.delta());
                    pending.timer.as_ref().unwrap().finished()
                }
                None => false,
            };
            if did_timer_expire {
                send_launch(&mut mouse_state, &mut launch_event);
            }
        } else if let Some(pending) = mouse_state.pending_launch.as_mut() {
            if can_shoot {
                pending.timer = Some(Timer::from_seconds(1.0, TimerMode::Once));
            }
        }
    }
    let Some(mut mouse_pos) = q_windows.single().cursor_position() else {
        // Mouse is not in the window, don't do anything
        return;
    };
    mouse_pos.x -= window_dims.0.x as f32 / 2.0;
    mouse_pos.y -= window_dims.0.y as f32 / 2.0;
    mouse_pos /= screen_mults.0 as f32;
    mouse_pos *= MENU_GROWTH as f32;

    let Some((camera_tran, camera_marker)) = camera_n_tran.iter().next() else {
        // Camera not found, don't do anything
        return;
    };
    let scale_down_to_screen = (SCREEN_WIDTH as f32) / (SCREEN_WIDTH as f32);
    mouse_state.pos = IVec2::new(mouse_pos.x.round() as i32, mouse_pos.y.round() as i32);
    mouse_pos *= scale_down_to_screen;
    let fworld_pos = camera_tran.translation.truncate()
        - camera_marker.scale.to_f32()
            * Vec2 {
                x: -mouse_pos.x / MENU_GROWTH as f32,
                y: mouse_pos.y / MENU_GROWTH as f32,
            };
    mouse_state.world_pos = IVec2::new(fworld_pos.x.round() as i32, fworld_pos.y.round() as i32);
    mouse_state.button_input = buttons.clone();
    // Begin a launch
    if buttons.just_pressed(MouseButton::Left) && mouse_state.pending_launch.is_none() {
        // Beginning launch
        mouse_state.pending_launch = Some(PendingLaunch {
            timer: if can_shoot {
                Some(Timer::from_seconds(1.0, TimerMode::Once))
            } else {
                None
            },
            launch_start: mouse_state.pos,
            launch_vel: Vec2::ZERO,
        });
    }

    if buttons.pressed(MouseButton::Left) {
        // Continue updating an existing launch
        let pos = mouse_state.pos;
        if let Some(pending_launch) = mouse_state.pending_launch.as_mut() {
            let mut almost_vel = (pending_launch.launch_start - pos).as_vec2();
            almost_vel.y *= -1.0;
            let norm = almost_vel.normalize_or_zero();
            let mag = if almost_vel.length() > 0.1 {
                almost_vel.length().sqrt() * MULT_THINGY
            } else {
                0.0
            };
            pending_launch.launch_vel = norm * mag;
        }
    } else {
        // Mouse button released, launch should happen
        if mouse_state.pending_launch.is_some() {
            send_launch(&mut mouse_state, &mut launch_event);
        }
    }
}

#[derive(Resource, Debug)]
pub struct CameraControlState {
    pub wasd_dir: Vec2,
}
impl CameraControlState {
    pub fn new() -> Self {
        Self {
            wasd_dir: Vec2::ZERO,
        }
    }
}

#[derive(Event, Debug)]
pub struct SwitchCameraModeEvent;

#[derive(Event, Debug)]
pub struct SetCameraModeEvent {
    pub mode: CameraMode,
}

#[derive(Event, Debug)]
pub struct CameraZoomEvent(pub i32);

pub fn watch_camera_input(
    mut camera_control_state: ResMut<CameraControlState>,
    keys: Res<ButtonInput<KeyCode>>,
    mut switch_event: EventWriter<SwitchCameraModeEvent>,
    mut zoom_event: EventWriter<CameraZoomEvent>,
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
    if zoom != 0 {
        zoom_event.send(CameraZoomEvent(zoom));
    }
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

#[derive(Component)]
struct ShotArrowMarker;

#[derive(Bundle)]
struct ShotArrowBundle {
    marker: ShotArrowMarker,
    spatial: SpatialBundle,
    anim: MultiAnimationManager,
    name: Name,
}
impl ShotArrowBundle {
    pub fn new() -> Self {
        let mut head = AnimationManager::single_static(SpriteInfo {
            path: "sprites/arrow_head.png".to_string(),
            size: UVec2::new(12, 10),
            ..default()
        });
        head.set_hidden(true);
        head.set_render_layers(vec![menu_layer_u8()]);
        let mut body = AnimationManager::single_static(SpriteInfo {
            path: "sprites/arrow_body.png".to_string(),
            size: UVec2::new(6, 12),
            ..default()
        });
        body.set_hidden(true);
        body.set_offset(IVec3::new(0, 0, -1));
        body.set_render_layers(vec![menu_layer_u8()]);
        let spatial =
            SpatialBundle::from_transform(Transform::from_translation(Vec3::new(0.0, 0.0, 102.0)));
        Self {
            marker: ShotArrowMarker,
            spatial,
            anim: MultiAnimationManager::from_pairs(vec![("head", head), ("body", body)]),
            name: Name::new("shot_arrow_marker"),
        }
    }
}

fn spawn_shot_arrow(mut commands: Commands) {
    commands.spawn(ShotArrowBundle::new());
}

fn update_shot_arrow(
    mut arrow: Query<(&mut Transform, &mut MultiAnimationManager), With<ShotArrowMarker>>,
    mouse_state: Res<MouseState>,
    gs: Res<GameState>,
    convos: Query<&Convo>,
) {
    let Ok((mut tran, mut multi)) = arrow.get_single_mut() else {
        return;
    };
    let set_invisible = |multi: &mut Mut<MultiAnimationManager>| {
        multi.map.get_mut("head").unwrap().set_hidden(true);
        multi.map.get_mut("body").unwrap().set_hidden(true);
    };
    if !should_apply_physics(gs, convos) {
        set_invisible(&mut multi);
        return;
    }
    tran.scale = Vec3::ONE * MENU_GROWTH as f32;
    match mouse_state.pending_launch.as_ref() {
        Some(pending_launch) => {
            let start = pending_launch.launch_start.as_vec2();
            tran.translation.x = start.x;
            tran.translation.y = -start.y;
            let end = start + pending_launch.launch_vel;
            let angle = Vec2::Y.angle_between(end - start);
            let body_len = ((start - end).length() / MULT_THINGY * 1.5).round() as i32;
            let body = multi.map.get_mut("body").unwrap();
            body.set_tran_angle(angle);
            body.set_hidden(false);
            body.set_points(vec![
                IVec2::new(-3, -1),
                IVec2::new(3, -1),
                IVec2::new(3, -body_len),
                IVec2::new(-3, -body_len),
            ]);
            let head = multi.map.get_mut("head").unwrap();
            head.set_tran_angle(angle);
            head.set_hidden(false);
        }
        None => {
            set_invisible(&mut multi);
        }
    }
}

pub fn register_input(app: &mut App) {
    app.insert_resource(MouseState::empty());
    app.add_event::<LaunchEvent>();
    app.insert_resource(CameraControlState::new());
    app.add_event::<SwitchCameraModeEvent>();
    app.add_event::<SetCameraModeEvent>();
    app.add_event::<CameraZoomEvent>();
    app.add_systems(Update, watch_mouse);
    app.add_systems(
        Update,
        watch_camera_input
            .run_if(in_editor.or_else(in_level))
            .run_if(is_not_in_cutscene),
    );
    app.add_systems(FixedUpdate, update_long_presses.run_if(is_not_in_cutscene));

    // Shot arrow
    app.add_systems(Startup, spawn_shot_arrow);
    app.add_systems(FixedUpdate, update_shot_arrow.after(camera_movement));
}
