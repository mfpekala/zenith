use std::collections::VecDeque;

use bevy::{ecs::system::SystemId, prelude::*};

use crate::{
    camera::{CameraMarker, CameraMode},
    physics::dyno::IntMoveable,
};

use super::{
    operation::ConvoRoot, CameraBeforeConvo, Convo, ConvoBoxBundle, ConvoBoxContent,
    ConvoBoxSpeaker, StartConvo,
};

#[derive(Debug, Clone, Copy)]
pub enum ConvoKind {
    Test,
}

fn spawn_test_convo(In(()): In<()>, mut commands: Commands) {
    let boxes = VecDeque::from_iter([
        ConvoBoxBundle::new(
            ConvoBoxSpeaker::None,
            ConvoBoxContent {
                content: "Hey there little ship boi, how are you?".to_string(),
                camera_mvmt: Some((IVec2::new(-100, 100), IVec2::new(100, -100))),
            },
        ),
        ConvoBoxBundle::new(
            ConvoBoxSpeaker::None,
            ConvoBoxContent {
                content: "I'm okay.".to_string(),
                camera_mvmt: Some((IVec2::new(100, -100), IVec2::new(100, -200))),
            },
        ),
    ]);
    let convo = Convo {
        kind: ConvoKind::Test,
        active_eid: None,
        bundles: boxes,
    };
    commands.spawn(convo);
}

/// RESIST THE URGE TO WRITE A MACRO (for now)
#[derive(Resource)]
struct ConvoOneshots {
    spawn_test_convo: SystemId<(), ()>,
}

fn start_conversations(
    oneshots: Res<ConvoOneshots>,
    mut starts: EventReader<StartConvo>,
    mut commands: Commands,
    existing: Query<&Convo>,
    mut current_camera: Query<(&mut CameraMarker, &IntMoveable)>,
    existing_saved_cameras: Query<Entity, With<CameraBeforeConvo>>,
    convo_root: Query<Entity, With<ConvoRoot>>,
) {
    let Some(start) = starts.read().last() else {
        return;
    };
    if !existing.is_empty() {
        warn!("Tried to spawn convo when another convo was opening");
        return;
    }

    // Save the state of the camera so we can restore it after this conversation is over
    let convo_root = convo_root.single();
    let (mut camera_marker, camera_mv) = current_camera.single_mut();
    for eid in existing_saved_cameras.iter() {
        commands.entity(eid).despawn();
    }
    commands.entity(convo_root).with_children(|parent| {
        parent.spawn(CameraBeforeConvo(camera_marker.clone(), camera_mv.clone()));
    });
    camera_marker.mode = CameraMode::Controlled;

    match start.0 {
        ConvoKind::Test => commands.run_system_with_input(oneshots.spawn_test_convo, ()),
    }
}

pub fn in_convo(convos: Query<&Convo>) -> bool {
    !convos.is_empty()
}

pub(super) fn register_convo_data(app: &mut App) {
    let oneshots: ConvoOneshots = ConvoOneshots {
        spawn_test_convo: app.world.register_system(spawn_test_convo),
    };
    app.insert_resource(oneshots);
    app.add_systems(Update, start_conversations);
}
