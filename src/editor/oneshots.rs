use bevy::{ecs::system::SystemId, prelude::*};

use crate::meta::game_state::EditingMode;

use super::{
    epoint::{delete_points, spawn_point},
    erock::spawn_rock,
    help::{spawn_help, submit_help_command},
    transitions::start_testing_exclusive,
};

#[derive(Resource)]
pub(super) struct EOneshots {
    pub(super) start_testing_exclusive: SystemId<(), ()>,
    pub(super) spawn_help: SystemId<(), ()>,
    pub(super) submit_help_command: SystemId<String, ()>,
    pub(super) spawn_point: SystemId<(EditingMode, IVec2), ()>,
    pub(super) delete_points: SystemId<Vec<Entity>, ()>,
    pub(super) spawn_rock: SystemId<(), ()>,
}

pub(super) fn register_oneshots(app: &mut App) {
    let oneshots = EOneshots {
        start_testing_exclusive: app.world.register_system(start_testing_exclusive),
        spawn_help: app.world.register_system(spawn_help),
        submit_help_command: app.world.register_system(submit_help_command),
        spawn_point: app.world.register_system(spawn_point),
        delete_points: app.world.register_system(delete_points),
        spawn_rock: app.world.register_system(spawn_rock),
    };
    app.insert_resource(oneshots);
}
