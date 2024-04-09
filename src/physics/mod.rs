use self::{
    collider::materialize_collider_stubs,
    dyno::{register_int_dynos, IntDyno},
};
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

pub struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        register_int_dynos(app);
        app.register_type::<IntDyno>();
        app.add_systems(Update, materialize_collider_stubs);
    }
}
