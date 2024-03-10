use bevy::prelude::*;

use crate::{environment::field::Field, ship::launch_ship};

use super::collider::{
    resolve_static_collisions, update_triggers, ColliderActive, ColliderBoundary, ColliderStatic,
    ColliderTrigger,
};

#[derive(Component, Debug)]
pub struct IntDyno {
    pub vel: Vec2,
    pub pos: IVec2,
    pub rem: Vec2,
    pub radius: f32,
    pub statics: Vec<Entity>,
    pub triggers: Vec<(Entity, f32)>,
}

pub fn move_int_dyno_helper(
    dyno: &mut IntDyno,
    statics: &Query<(
        Entity,
        &ColliderBoundary,
        &ColliderStatic,
        Option<&ColliderActive>,
    )>,
) {
    // We'll be "inching" and checking for collisions in both directions (like Celeste),
    // so best to keep that logic in a helper function
    let resolve_inching = |dyno: &mut IntDyno, diff: IVec2, num_steps: u32| -> bool {
        for _ in 0..num_steps {
            dyno.pos += diff;
            if resolve_static_collisions(dyno, statics) {
                return true;
            }
        }
        false
    };

    // We want to only move in integer amounts, but keep track of remainders so fractional velocities make sense
    let would_move = dyno.vel + dyno.rem;
    let move_x = would_move.x.round() as i32;
    let move_y = would_move.y.round() as i32;

    if move_x != 0 {
        // There's horizontal motion to resolve
        resolve_inching(
            dyno,
            IVec2 {
                x: move_x.signum(),
                y: 0,
            },
            move_x.abs() as u32,
        );
        dyno.rem.x = would_move.x - move_x as f32;
    } else {
        // No horizontal motion, but remember our "progress" in the remainder for next frame
        dyno.rem.x = would_move.x;
    }

    if move_y != 0 {
        // There's vertical motion to resolve
        resolve_inching(
            dyno,
            IVec2 {
                x: 0,
                y: move_y.signum(),
            },
            move_y.abs() as u32,
        );
        dyno.rem.y = would_move.y - move_y as f32;
    } else {
        // No vertical motion, but remember our "progress" in the remainder for next frame
        dyno.rem.y = would_move.y;
    }
}

pub fn move_int_dynos(
    mut dynos: Query<(&mut IntDyno, &mut Transform)>,
    statics: Query<(
        Entity,
        &ColliderBoundary,
        &ColliderStatic,
        Option<&ColliderActive>,
    )>,
) {
    for (mut dyno, mut tran) in dynos.iter_mut() {
        move_int_dyno_helper(dyno.as_mut(), &statics);
        tran.translation.x = dyno.pos.x as f32;
        tran.translation.y = dyno.pos.y as f32;
    }
}

fn resolve_dynos(
    mut dynos: Query<&mut IntDyno>,
    _statics: Query<(Entity, &ColliderStatic, Option<&ColliderActive>)>,
    triggers: Query<(&Parent, &ColliderTrigger, Option<&ColliderActive>)>,
    fields: Query<&Field>,
) {
    for mut dyno in dynos.iter_mut() {
        let mut diff = Vec2::ZERO;
        let mut slowdown = 1.0;
        for (trigger_id, mult) in dyno.triggers.iter() {
            let Ok((parent, _, active)) = triggers.get(*trigger_id) else {
                continue;
            };
            if active.is_some() && !active.unwrap().0 {
                continue;
            }
            if let Ok(field) = fields.get(parent.get()) {
                diff += field.dir * field.strength * *mult;
                slowdown *= (1.0 - field.drag).powf(*mult);
            }
        }
        dyno.vel += diff;
        dyno.vel *= slowdown;
        dyno.triggers = vec![];
    }
}

pub fn register_int_dynos(app: &mut App) {
    app.add_systems(FixedUpdate, move_int_dynos.after(launch_ship));
    app.add_systems(FixedUpdate, resolve_dynos.after(update_triggers));
}
