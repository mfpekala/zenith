use bevy::{ecs::system::SystemId, prelude::*};

use crate::meta::{game_state::EditingMode, level_data::LevelData};

use super::{
    efield::spawn_field,
    egoal::spawn_goal,
    elive_poly::spawn_live_poly,
    epoint::{delete_points, spawn_point},
    ereplenish::spawn_replenish,
    erock::spawn_rock,
    estart::spawn_start,
    export::{freeze_level_data, load_level, save_level},
    help::{spawn_help, submit_help_command},
    transitions::start_testing_exclusive,
};

#[derive(Resource, Clone)]
pub(super) struct EOneshots {
    pub(super) start_testing_exclusive: SystemId<(), ()>,
    pub(super) spawn_help: SystemId<(), ()>,
    pub(super) submit_help_command: SystemId<String, ()>,
    pub(super) spawn_point: SystemId<(EditingMode, IVec2), ()>,
    pub(super) delete_points: SystemId<Vec<Entity>, ()>,
    pub(super) spawn_rock: SystemId<(), ()>,
    pub(super) spawn_field: SystemId<(), ()>,
    pub(super) spawn_replenish: SystemId<IVec2, ()>,
    pub(super) spawn_start: SystemId<IVec2, ()>,
    pub(super) spawn_goal: SystemId<IVec2, ()>,
    pub(super) spawn_live_poly: SystemId<(), ()>,
    pub(super) freeze_level_data: SystemId<(), Result<LevelData, String>>,
    pub(super) save_level: SystemId<String, ()>,
    pub(super) load_level: SystemId<String, ()>,
}

pub(super) fn register_oneshots(app: &mut App) {
    let oneshots = EOneshots {
        start_testing_exclusive: app.world.register_system(start_testing_exclusive),
        spawn_help: app.world.register_system(spawn_help),
        submit_help_command: app.world.register_system(submit_help_command),
        spawn_point: app.world.register_system(spawn_point),
        delete_points: app.world.register_system(delete_points),
        spawn_rock: app.world.register_system(spawn_rock),
        spawn_field: app.world.register_system(spawn_field),
        spawn_replenish: app.world.register_system(spawn_replenish),
        spawn_start: app.world.register_system(spawn_start),
        spawn_goal: app.world.register_system(spawn_goal),
        spawn_live_poly: app.world.register_system(spawn_live_poly),
        freeze_level_data: app.world.register_system(freeze_level_data),
        save_level: app.world.register_system(save_level),
        load_level: app.world.register_system(load_level),
    };
    app.insert_resource(oneshots);
}
