use bevy::{ecs::system::SystemId, prelude::*};

use super::{
    help::{spawn_help, submit_help_command},
    transitions::start_testing_exclusive,
};

#[derive(Resource)]
pub(super) struct EOneshots {
    pub(super) start_testing_exclusive: SystemId<(), ()>,
    pub(super) spawn_help: SystemId<(), ()>,
    pub(super) submit_help_command: SystemId<String, ()>,
}

pub(super) fn register_oneshots(app: &mut App) {
    let oneshots = EOneshots {
        start_testing_exclusive: app.world.register_system(start_testing_exclusive),
        spawn_help: app.world.register_system(spawn_help),
        submit_help_command: app.world.register_system(submit_help_command),
    };
    app.insert_resource(oneshots);
}
