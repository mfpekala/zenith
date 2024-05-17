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
    slowdown: u32,
    fixed_equiv: u32,
}
impl BulletTime {
    pub fn from_slowdown_u32(slowdown: u32) -> Self {
        Self {
            slowdown,
            fixed_equiv: 0,
        }
    }

    pub fn factor(&self) -> f32 {
        1.0 / self.slowdown as f32
    }

    pub fn is_special_frame(&self) -> bool {
        self.fixed_equiv == 0
    }
}

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
        if bullet_time.slowdown == 1 {
            // We are entering bullet time
            bullet_time.slowdown = 20;
            bullet_time.fixed_equiv = 0;
            scale_dyno_vels(1.0 / 20.0);
        } else {
            // We are already in bullet time
            bullet_time.fixed_equiv = (bullet_time.fixed_equiv + 1) % bullet_time.slowdown;
        }
    } else {
        if bullet_time.slowdown == 1 {
            // We already left bullet time
        } else {
            // We are leaving bullet time
            bullet_time.slowdown = 1;
            bullet_time.fixed_equiv = 0;
            scale_dyno_vels(20.0);
        }
    }
}

pub struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        register_int_dynos(app);
        app.insert_resource(BulletTime::from_slowdown_u32(1));
        app.register_type::<IntDyno>();
        app.add_systems(Update, materialize_collider_stubs);
        app.add_systems(Update, trickle_active);
        app.add_systems(FixedUpdate, update_bullet_time.before(move_int_dynos));
    }
}
