//! This module handles all the necessary editor transitions.
//! This includes:
//! - Entering/leaving the editor in general (inclusive of editing and testing)
//! - Entering/leaving editing
//! - Entering/leaving testing

use crate::{
    camera::{CameraMarker, CameraMode},
    environment::background::*,
    input::SetCameraModeEvent,
    meta::{game_state::*, old_level_data::LevelDataOneshots},
    physics::dyno::IntMoveable,
    when_becomes_false, when_becomes_true,
};
use bevy::{ecs::system::SystemState, prelude::*};

use super::oneshots::EOneshots;

fn is_editing_helper(gs: &GameState) -> bool {
    match gs.meta {
        MetaState::Editor(editor_state) => match editor_state {
            EditorState::Editing(_) => true,
            _ => false,
        },
        _ => false,
    }
}
pub fn in_editing(gs: Res<GameState>) -> bool {
    is_editing_helper(&gs)
}
when_becomes_true!(is_editing_helper, entered_editing);
when_becomes_false!(is_editing_helper, left_editing);

fn is_testing_helper(gs: &GameState) -> bool {
    match gs.meta {
        MetaState::Editor(editor_state) => match editor_state {
            EditorState::Testing => true,
            _ => false,
        },
        _ => false,
    }
}
pub fn in_testing(gs: Res<GameState>) -> bool {
    is_testing_helper(&gs)
}
when_becomes_true!(is_testing_helper, entered_testing);
when_becomes_false!(is_testing_helper, left_testing);

const EROOT_HOME: IVec2 = IVec2::ZERO;
const TROOT_HOME: IVec2 = IVec2::new(6_000, 6_000);
const HROOT_HOME: IVec2 = IVec2::ZERO;

#[derive(Component, Default)]
struct ERoot;
#[derive(Resource)]
pub(super) struct ERootEid(pub Entity);

#[derive(Component, Default)]
struct TRoot;
#[derive(Resource)]
pub(super) struct TRootEid(pub Entity);

#[derive(Component, Default)]
struct HRoot;
#[derive(Resource)]
pub(super) struct HRootEid(pub Entity);

#[derive(Bundle)]
struct CommonRootBundle<T: Component + Default> {
    marker: T,
    name: Name,
    spatial: SpatialBundle,
}
impl<T: Component + Default> CommonRootBundle<T> {
    fn new(name: &str, pos: IVec2) -> Self {
        Self {
            marker: T::default(),
            name: Name::new(name.to_string()),
            spatial: SpatialBundle::from_transform(Transform::from_translation(
                pos.as_vec2().extend(0.0),
            )),
        }
    }
}

/// Called exactly once when the MetaState becomes the Editor variant
pub(super) fn setup_editor(
    mut commands: Commands,
    e_oneshots: Res<EOneshots>,
    mut eroot: ResMut<ERootEid>,
    mut troot: ResMut<TRootEid>,
    mut hroot: ResMut<HRootEid>,
    mut set_event: EventWriter<SetCameraModeEvent>,
    mut bg_manager: ResMut<BgManager>,
) {
    *eroot = ERootEid(
        commands
            .spawn(CommonRootBundle::<ERoot>::new("ERoot", EROOT_HOME))
            .id(),
    );
    *troot = TRootEid(
        commands
            .spawn(CommonRootBundle::<TRoot>::new("TRoot", TROOT_HOME))
            .id(),
    );
    *hroot = HRootEid(
        commands
            .spawn(CommonRootBundle::<HRoot>::new("HRoot", HROOT_HOME))
            .id(),
    );
    commands.run_system(e_oneshots.spawn_help);
    set_event.send(SetCameraModeEvent {
        mode: CameraMode::Free,
    });
    bg_manager.set_kind(BgKind::ParallaxStars(300));
}

/// Called exactly once when the MetaState leaves the Editor variant
pub(super) fn destroy_editor(
    mut commands: Commands,
    mut eroot: ResMut<ERootEid>,
    mut troot: ResMut<TRootEid>,
    mut hroot: ResMut<HRootEid>,
) {
    if let Some(commands) = commands.get_entity(eroot.0) {
        commands.despawn_recursive();
    }
    if let Some(commands) = commands.get_entity(troot.0) {
        commands.despawn_recursive();
    }
    if let Some(commands) = commands.get_entity(hroot.0) {
        commands.despawn_recursive();
    }
    *eroot = ERootEid(Entity::PLACEHOLDER);
    *troot = TRootEid(Entity::PLACEHOLDER);
    *hroot = HRootEid(Entity::PLACEHOLDER);
}

/// Called exactly once when the MetaState::Editor becomes the Editing variant
/// TRICKY: `Editing` = working on a level, `Editor` = `Editing` || `Testing`
pub(super) fn setup_editing(mut camera_q: Query<(&mut CameraMarker, &mut IntMoveable)>) {
    let (mut marker, mut mv) = camera_q.single_mut();
    marker.mode = CameraMode::Free;
    mv.fpos = EROOT_HOME.as_vec2().extend(mv.fpos.z);
}

/// Called exactly once when the MetaState::Editor leaves the Editing variant
/// TRICKY: `Editing` = working on a level, `Editor` = `Editing` || `Testing`
pub(super) fn destroy_editing() {}

/// Called exactly once when the MetaState::Editor becomes the Testing variant
pub(super) fn setup_testing(
    mut camera_q: Query<(&mut CameraMarker, &mut IntMoveable)>,
    e_oneshots: Res<EOneshots>,
    mut commands: Commands,
) {
    let (mut marker, mut mv) = camera_q.single_mut();
    marker.mode = CameraMode::Follow { dislodgement: None };
    mv.fpos = TROOT_HOME.as_vec2().extend(mv.fpos.z);
    commands.run_system(e_oneshots.start_testing_exclusive);
}

/// Called exactly once when the MetaState::Editor leaves the Testing variant
pub(super) fn destroy_testing() {}

/// This is helpful to have as a oneshot so setup_testing can be cleaner.
pub(super) fn start_testing_exclusive(
    In(()): In<()>,
    world: &mut World,
    params: &mut SystemState<Res<LevelDataOneshots>>,
) {
    let level_oneshots = params.get_mut(world);
    let level_oneshots = level_oneshots.clone();
    match world.run_system(level_oneshots.crystallize_level_data_id) {
        Ok(level_data) => {
            world
                .run_system_with_input(level_oneshots.old_spawn_level, (1, level_data, TROOT_HOME))
                .unwrap();
        }
        Err(e) => {
            warn!("Failed to crystallize level data (system): {e:?}");
        }
    }
}
