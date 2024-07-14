use bevy::{ecs::system::SystemState, prelude::*};

use crate::{
    camera::{CameraMarker, CameraMode},
    input::SetCameraModeEvent,
    meta::old_level_data::LevelDataOneshots,
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
        EventWriter<HelpBarEvent>,
        EventWriter<SetCameraModeEvent>,
    )>,
) {
    let (level_oneshots, _, _) = params.get_mut(world);
    let level_oneshots = level_oneshots.clone();
    let success = match world.run_system(level_oneshots.crystallize_level_data_id) {
        Ok(level_data) => {
            world
                .run_system_with_input(
                    level_oneshots.old_spawn_level,
                    (1, level_data, TESTING_HOME),
                )
                .unwrap();
            true
        }
        Err(_) => {
            let (_, mut event, _) = params.get_mut(world);
            event.send(HelpBarEvent(
                "Failed to crystallize level data (system)".to_string(),
            ));
            false
        }
    };
    if success {
        let (_, _, mut writer) = params.get_mut(world);
        writer.send(SetCameraModeEvent {
            mode: CameraMode::Follow { dislodgement: None },
        });
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
    camera.fpos = EDITING_HOME.extend(0);
    let Some(eid) = ut.get_entity(1) else {
        return;
    };
    commands.entity(eid).despawn_recursive();
}
