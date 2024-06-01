use bevy::{
    prelude::*,
    utils::{hashbrown::HashSet, HashMap},
};
use serde::{Deserialize, Serialize};

use crate::{
    drawing::animation::AnimationManager,
    environment::{
        field::Field,
        segment::{Segment, SegmentKind},
    },
};

use super::{
    collider::{
        resolve_static_collisions, resolve_trigger_collisions, update_triggers, ColliderActive,
        ColliderBoundary, ColliderStatic, ColliderTrigger,
    },
    BulletTime,
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

pub(super) fn move_int_moveables(
    mut moveables: Query<(&mut Transform, &mut IntMoveable)>,
    bullet_time: Res<BulletTime>,
) {
    for (mut tran, mut moveable) in moveables.iter_mut() {
        // We move the objects in much the same way that we move dynos
        let would_move = (moveable.vel + moveable.rem) * bullet_time.factor();
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
    pub statics: HashSet<Entity>,
    pub triggers: HashMap<Entity, f32>,
    pub long_statics: HashMap<Entity, u32>,
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

pub(super) fn move_int_dyno_helper(
    dyno: &mut IntDyno,
    statics: &Query<(
        Entity,
        &ColliderBoundary,
        &ColliderStatic,
        Option<&ColliderActive>,
        &Parent,
    )>,
    triggers: &Query<(
        &ColliderBoundary,
        &ColliderTrigger,
        Option<&ColliderActive>,
        &Parent,
    )>,
    segments: &mut Query<(&Segment, &mut AnimationManager)>,
    bullet_time: &Res<BulletTime>,
) {
    let mut amt_travlled = 0.0;
    let to_travel = dyno.vel.length();

    while amt_travlled < to_travel && amt_travlled < dyno.vel.length() {
        let this_step = if dyno.vel.length() - amt_travlled >= 1.0 {
            1.0
        } else {
            let min_move: f32 = 0.0000000001;
            if dyno.vel.length() - amt_travlled > min_move {
                dyno.vel.length().rem_euclid(1.0).max(min_move)
            } else {
                break;
            }
        };
        dyno.fpos += dyno.vel.normalize_or_zero().extend(0.0) * this_step;
        resolve_static_collisions(dyno, statics);
        resolve_trigger_collisions(dyno, triggers);
        let mut killing_ids = HashSet::new();
        let mut sprung = false;
        for (eid, _emult) in dyno.triggers.iter() {
            let Ok((segment, mut anim)) = segments.get_mut(*eid) else {
                continue;
            };
            match segment.kind {
                SegmentKind::Spring => {
                    killing_ids.insert(*eid);
                    if !sprung {
                        let line = (segment.right_parent - segment.left_parent).as_vec2();
                        let norm = Vec2::new(-line.y, line.x).normalize_or_zero();
                        let pure_parr = -1.0 * dyno.vel.dot(norm) * norm + dyno.vel;
                        let new_vel = pure_parr + norm * 3.0 * bullet_time.factor();
                        dyno.vel = new_vel;
                        sprung = true;
                        anim.reset_key("bounce");
                    }
                }
                SegmentKind::Spike => {
                    // Set the velocity to so it stops on the spike, but DON'T add this to the killing_ids
                    // so that a follow-up system can read this trigger and kill the ship
                    dyno.vel = Vec2::ZERO;
                }
            }
        }
        dyno.triggers.retain(|id, _| !killing_ids.contains(id));
        amt_travlled += this_step;
    }
    resolve_trigger_collisions(dyno, triggers);

    dyno.ipos = IVec3::new(
        dyno.fpos.x.round() as i32,
        dyno.fpos.y.round() as i32,
        dyno.fpos.z.round() as i32,
    );
}

pub(super) fn move_int_dynos(
    mut dynos: Query<(&mut IntDyno, &mut Transform)>,
    statics: Query<(
        Entity,
        &ColliderBoundary,
        &ColliderStatic,
        Option<&ColliderActive>,
        &Parent,
    )>,
    triggers: Query<(
        &ColliderBoundary,
        &ColliderTrigger,
        Option<&ColliderActive>,
        &Parent,
    )>,
    mut segments: Query<(&Segment, &mut AnimationManager)>,
    bullet_time: Res<BulletTime>,
) {
    for (mut dyno, mut tran) in dynos.iter_mut() {
        // Clear the old collisions/triggers
        dyno.statics = HashSet::new();
        dyno.triggers = HashMap::new();
        move_int_dyno_helper(
            dyno.as_mut(),
            &statics,
            &triggers,
            &mut segments,
            &bullet_time,
        );

        // Update the long statics (for replenishing shot)
        let statics = dyno.statics.clone();
        for key in statics.iter() {
            let contained = dyno.long_statics.contains_key(key);
            if contained {
                let count = dyno.long_statics.get_mut(key).unwrap();
                *count += 1;
            } else {
                dyno.long_statics.insert(*key, 1);
            }
        }
        let mut killing = vec![];
        for key in dyno.long_statics.keys() {
            if !dyno.statics.contains(key) {
                killing.push(*key);
            }
        }
        for key in killing.iter() {
            dyno.long_statics.remove(key);
        }

        tran.translation.x = dyno.ipos.x as f32;
        tran.translation.y = dyno.ipos.y as f32;
    }
}

pub fn apply_fields(
    mut dynos: Query<&mut IntDyno>,
    fields: Query<(&Field, Option<&ColliderActive>)>,
    bullet_time: Res<BulletTime>,
) {
    for mut dyno in dynos.iter_mut() {
        let mut diff = Vec2::ZERO;
        let mut killing_ids = HashSet::new();
        for (trigger_id, mult) in dyno.triggers.iter() {
            if let Ok((field, active)) = fields.get(*trigger_id) {
                killing_ids.insert(*trigger_id);
                if active.is_some() && !active.unwrap().0 {
                    continue;
                }
                diff += field.dir
                    * field.strength.to_f32()
                    * *mult
                    * bullet_time.factor()
                    * bullet_time.factor();
                // slowdown *= (1.0 - field.drag.to_f32()).powf(*mult);
            }
        }
        dyno.vel += diff;
        dyno.triggers.retain(|id, _| !killing_ids.contains(id));
    }
}

pub fn register_int_dynos(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (move_int_dynos, update_triggers, apply_fields).chain(),
    );

    app.add_systems(FixedUpdate, move_int_moveables.after(move_int_dynos));
}
