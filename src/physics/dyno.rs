use bevy::prelude::*;

use super::collider::{resolve_static_collisions, ColliderBoundary, ColliderStatic};

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
    statics: &Query<(Entity, &ColliderBoundary, &ColliderStatic)>,
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

    // As a limitation, we only handle one collision per frame
    let mut had_collision = false;

    if move_x != 0 {
        // There's horizontal motion to resolve
        had_collision = resolve_inching(
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
        if !had_collision {
            resolve_inching(
                dyno,
                IVec2 {
                    x: 0,
                    y: move_y.signum(),
                },
                move_y.abs() as u32,
            );
        }
        dyno.rem.y = would_move.y - move_y as f32;
    } else {
        // No vertical motion, but remember our "progress" in the remainder for next frame
        dyno.rem.y = would_move.y;
    }
}

fn move_int_dynos(
    mut dynos: Query<(&mut IntDyno, &mut Transform)>,
    statics: Query<(Entity, &ColliderBoundary, &ColliderStatic)>,
) {
    for (mut dyno, mut tran) in dynos.iter_mut() {
        move_int_dyno_helper(dyno.as_mut(), &statics);
        tran.translation.x = dyno.pos.x as f32;
        tran.translation.y = dyno.pos.y as f32;
    }
}

pub fn register_int_dynos(app: &mut App) {
    app.add_systems(FixedUpdate, move_int_dynos);
}
