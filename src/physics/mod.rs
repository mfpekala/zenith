use self::{collider::register_colliders, dyno::register_int_dynos};
use crate::meta::game_state::{EditorState, GameState, MetaState};
use bevy::prelude::*;

pub mod collider;
pub mod dyno;

pub fn should_apply_physics(gs: Res<GameState>) -> bool {
    match gs.meta {
        MetaState::Menu(_) => false,
        MetaState::Editor(editor_state) => match editor_state {
            EditorState::Testing => true,
            _ => false,
        },
        MetaState::Level(_) => true,
    }
}

pub fn register_physics(app: &mut App) {
    register_colliders(app);
    register_int_dynos(app);
}
