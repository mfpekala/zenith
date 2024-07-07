use bevy::prelude::*;

use super::Convo;

pub mod spawns;
pub mod speakers;

#[derive(Debug, Clone, Copy)]
pub enum ConvoKind {
    Test,
}

pub fn in_convo(convos: Query<&Convo>) -> bool {
    !convos.is_empty()
}

pub(super) fn register_convo_data(app: &mut App) {
    spawns::register_spawns(app);
}
