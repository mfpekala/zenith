use bevy::{ecs::system::SystemId, prelude::*};

use super::transitions::start_testing_exclusive;

#[derive(Resource)]
pub(super) struct EOneshots {
    pub(super) start_testing_exclusive: SystemId<(), ()>,
}

pub(super) fn register_oneshots(app: &mut App) {
    let oneshots = EOneshots {
        start_testing_exclusive: app.world.register_system(start_testing_exclusive),
    };
    app.insert_resource(oneshots);
}
