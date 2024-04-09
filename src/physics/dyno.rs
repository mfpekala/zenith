use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::environment::field::Field;

use super::collider::{
    resolve_static_collisions, update_triggers, ColliderActive, ColliderBoundary, ColliderStatic,
    ColliderTrigger,
};

#[derive(Component, Debug, Default, Clone, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct IntMoveable {
    pub vel: Vec2,
    pub pos: IVec3,
    pub rem: Vec2,
}
impl IntMoveable {
    pub fn new(pos: IVec3) -> Self {
        Self {
            vel: Vec2::ZERO,
            rem: Vec2::ZERO,
            pos,
        }
    }
}

pub fn move_int_moveables(mut moveables: Query<(&mut Transform, &mut IntMoveable)>) {
    for (mut tran, mut moveable) in moveables.iter_mut() {
        // We move the objects in much the same way that we move dynos
        let would_move = moveable.vel + moveable.rem;
        let move_x = would_move.x.round() as i32;
        let move_y = would_move.y.round() as i32;
        if move_x != 0 {
            moveable.pos.x += move_x;
            moveable.rem.x = would_move.x - move_x as f32;
        } else {
            moveable.rem.x = would_move.x;
        }
        if move_y != 0 {
            moveable.pos.y += move_y;
            moveable.rem.y = would_move.y - move_y as f32;
        } else {
            moveable.rem.y = would_move.y;
        }
        tran.translation = moveable.pos.as_vec3();
    }
}

#[derive(Component, Debug, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct IntDyno {
    pub vel: Vec2,
    pub fpos: Vec3,
    pub ipos: IVec3,
    pub radius: f32,
    pub statics: Vec<Entity>,
    pub triggers: Vec<(Entity, f32)>,
}
impl IntDyno {
    pub fn new(pos: IVec3, radius: f32) -> Self {
        Self {
            fpos: pos.as_vec3(),
            ipos: pos,
            radius,
            ..default()
        }
    }
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
    let true_start = dyno.fpos;
    let true_dir = dyno.vel.normalize_or_zero();
    let inch_by = |dyno: &mut IntDyno, amt: f32| -> bool {
        dyno.fpos = true_start + true_dir.extend(0.0) * amt;
        resolve_static_collisions(dyno, statics)
    };
    let total_amt = dyno.vel.length().floor() as u32;
    for amt in 1..=total_amt {
        if inch_by(dyno, amt as f32) {
            break;
        }
    }
    if dyno.statics.len() == 0 {
        inch_by(dyno, dyno.vel.length());
    }
    dyno.ipos = IVec3::new(
        dyno.fpos.x.round() as i32,
        dyno.fpos.y.round() as i32,
        dyno.fpos.z.round() as i32,
    );
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
        // Clear the old static collisions
        dyno.statics = vec![];
        move_int_dyno_helper(dyno.as_mut(), &statics);
        tran.translation.x = dyno.ipos.x as f32;
        tran.translation.y = dyno.ipos.y as f32;
    }
}

pub fn resolve_dynos(
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
                diff += field.dir * field.strength.to_f32() * *mult;
                slowdown *= (1.0 - field.drag.to_f32()).powf(*mult);
            }
        }
        dyno.vel += diff;
        dyno.vel *= slowdown;
        dyno.triggers = vec![];
    }
}

pub fn register_int_dynos(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (move_int_dynos, update_triggers, resolve_dynos).chain(),
    );

    app.add_systems(FixedUpdate, move_int_moveables);
}
