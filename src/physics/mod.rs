use self::{
    collider::{materialize_collider_stubs, trickle_active},
    dyno::{move_int_dynos, register_int_dynos, IntDyno},
};
use crate::{
    input::MouseState,
    meta::game_state::{EditorState, GameState, MetaState},
};
use bevy::prelude::*;

pub mod collider;
pub mod dyno;

#[derive(Resource)]
pub struct BulletTime {
    pub is_slow: bool,
}
impl BulletTime {
    pub const fn slowdown() -> f32 {
        8.0
    }

    pub fn new() -> Self {
        Self { is_slow: false }
    }

    pub fn factor(&self) -> f32 {
        if self.is_slow {
            1.0 / Self::slowdown()
        } else {
            1.0
        }
    }
}

pub fn should_apply_physics(gs: Res<GameState>) -> bool {
    if gs.paused {
        return false;
    }
    match gs.meta {
        MetaState::Menu(_) => false,
        MetaState::Editor(editor_state) => match editor_state {
            EditorState::Testing => true,
            _ => false,
        },
        MetaState::Level(_) => true,
    }
}

pub fn update_bullet_time(
    mut bullet_time: ResMut<BulletTime>,
    mouse_state: Res<MouseState>,
    mut dynos: Query<&mut IntDyno>,
) {
    let in_bullet_time = match mouse_state.pending_launch.as_ref() {
        Some(pending) => pending.timer.is_some(),
        None => false,
    };
    let mut scale_dyno_vels = |scale: f32| {
        for mut dyno in dynos.iter_mut() {
            dyno.vel *= scale;
        }
    };
    if in_bullet_time {
        if !bullet_time.is_slow {
            // We are entering bullet time
            bullet_time.is_slow = true;
            scale_dyno_vels(bullet_time.factor());
        } else {
            // We are already in bullet time
        }
    } else {
        if !bullet_time.is_slow {
            // We already left bullet time
        } else {
            // We are leaving bullet time
            scale_dyno_vels(1.0 / bullet_time.factor());
            bullet_time.is_slow = false;
        }
    }
}

pub struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        register_int_dynos(app);
        app.insert_resource(BulletTime::new());
        app.register_type::<IntDyno>();
        app.add_systems(Update, materialize_collider_stubs);
        app.add_systems(Update, trickle_active);
        app.add_systems(FixedUpdate, update_bullet_time.before(move_int_dynos));
    }
}
