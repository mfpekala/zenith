use bevy::{ecs::system::SystemState, prelude::*};

use crate::{
    camera::{CameraMarker, CameraMode},
    input::SetCameraModeEvent,
    meta::level_data::LevelDataOneshots,
    physics::dyno::IntMoveable,
    uid::UIdTranslator,
};

use super::help::HelpBarEvent;

const EDITING_HOME: IVec2 = IVec2::new(0, 0);
const TESTING_HOME: IVec2 = IVec2::new(6000, 6000);

pub(super) fn start_testing(
    world: &mut World,
    params: &mut SystemState<(
        Res<LevelDataOneshots>,
        Query<&mut IntMoveable, With<CameraMarker>>,
        EventWriter<HelpBarEvent>,
    )>,
) {
    let (level_oneshots, _, _) = params.get_mut(world);
    let level_oneshots = level_oneshots.clone();
    match world.run_system(level_oneshots.crystallize_level_data_id) {
        Ok(level_data) => {
            world
                .run_system_with_input(level_oneshots.spawn_level_id, (1, level_data, TESTING_HOME))
                .unwrap();
            let (_, mut camera_q, mut event) = params.get_mut(world);
            let Ok(mut camera) = camera_q.get_single_mut() else {
                event.send(HelpBarEvent("Couldn't get camera".to_string()));
                return;
            };
            camera.pos = TESTING_HOME.extend(0);
        }
        Err(_) => {
            let (_, _, mut event) = params.get_mut(world);
            event.send(HelpBarEvent(
                "Failed to crystallize level data (system)".to_string(),
            ));
        }
    }
}

pub(super) fn stop_testing(
    mut camera_q: Query<&mut IntMoveable, With<CameraMarker>>,
    ut: Res<UIdTranslator>,
    mut commands: Commands,
    mut set_event: EventWriter<SetCameraModeEvent>,
) {
    let Ok(mut camera) = camera_q.get_single_mut() else {
        return;
    };
    set_event.send(SetCameraModeEvent {
        mode: CameraMode::Free,
    });
    camera.pos = EDITING_HOME.extend(0);
    let Some(eid) = ut.get_entity(1) else {
        return;
    };
    commands.entity(eid).despawn_recursive();
}
