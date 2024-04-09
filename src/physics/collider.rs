use bevy::prelude::*;

use crate::{
    math::MathLine,
    meta::consts::MAX_COLLISIONS_PER_FRAME,
    uid::{UId, UIdMarker},
};

use super::dyno::IntDyno;

#[derive(Component, Debug)]
pub struct ColliderBoundary {
    pub points: Vec<IVec2>,
    pub lines: Vec<MathLine>,
    pub center: Vec2,
    pub bound_squared: f32,
}
impl ColliderBoundary {
    pub fn from_points(boundary_points: Vec<IVec2>) -> Self {
        let mut center = Vec2::ZERO;
        for point in boundary_points.iter() {
            center += point.as_vec2();
        }
        if boundary_points.len() > 0 {
            center /= boundary_points.len() as f32;
        }
        let mut max_dist_sq: f32 = 0.0;
        for point in boundary_points.iter() {
            max_dist_sq = max_dist_sq.max(point.as_vec2().distance_squared(center));
        }
        let fpoints: Vec<Vec2> = boundary_points.iter().map(|p| p.as_vec2()).collect();
        let lines = MathLine::from_points(&fpoints);
        ColliderBoundary {
            points: boundary_points,
            lines,
            center,
            bound_squared: max_dist_sq,
        }
    }

    pub fn closest_point(&self, point: Vec2) -> Vec2 {
        let mut min_dist_sq = f32::MAX;
        let mut min_point = Vec2 {
            x: f32::MAX,
            y: f32::MAX,
        };
        for line in self.lines.iter() {
            let close_point = line.closest_point_on_segment(&point);
            let dist_sq = point.distance_squared(close_point);
            if dist_sq < min_dist_sq {
                min_dist_sq = dist_sq;
                min_point = close_point;
            }
        }
        min_point
    }

    pub fn effective_mult(&self, point: Vec2, radius: f32) -> f32 {
        let mut max_dist = f32::MIN;
        for line in self.lines.iter() {
            let signed_dist = line.signed_distance_from_point(&point);
            max_dist = max_dist.max(signed_dist);
        }
        max_dist = (-1.0 * (max_dist / radius) + 1.0) / 2.0;
        max_dist.min(1.0).max(0.0)
    }
}

#[derive(Component, Debug)]
pub struct ColliderStatic {
    pub bounciness: f32,
    pub friction: f32,
}

#[derive(Component, Debug)]
pub struct ColliderTrigger;

#[derive(Component, Debug)]
pub struct ColliderActive(pub bool);

pub struct ColliderStaticStub {
    pub uid: UId,
    pub points: Vec<IVec2>,
    pub active: bool,
    pub bounciness: f32,
    pub friction: f32,
}

#[derive(Component)]
pub struct ColliderStaticStubs(pub Vec<ColliderStaticStub>);

#[derive(Bundle)]
pub struct ColliderStaticBundle {
    _static: ColliderStatic,
    boundary: ColliderBoundary,
    active: ColliderActive,
}

pub struct ColliderTriggerStub {
    pub uid: UId,
    pub points: Vec<IVec2>,
    pub active: bool,
}

#[derive(Component)]
pub struct ColliderTriggerStubs(pub Vec<ColliderTriggerStub>);

#[derive(Bundle)]
pub struct ColliderTriggerBundle {
    pub trigger: ColliderTrigger,
    pub boundary: ColliderBoundary,
    pub active: ColliderActive,
}

/// Materialize the collider stubs, creating actual colliders
pub(super) fn materialize_collider_stubs(
    mut commands: Commands,
    static_stubs: Query<(Entity, &ColliderStaticStubs)>,
    trigger_stubs: Query<(Entity, &ColliderTriggerStubs)>,
) {
    // Statics
    for (eid, stubs) in static_stubs.iter() {
        for stub in stubs.0.iter() {
            commands.entity(eid).with_children(|parent| {
                parent.spawn((
                    UIdMarker(stub.uid),
                    ColliderStaticBundle {
                        _static: ColliderStatic {
                            bounciness: stub.bounciness,
                            friction: stub.friction,
                        },
                        boundary: ColliderBoundary::from_points(stub.points.clone()),
                        active: ColliderActive(stub.active),
                    },
                ));
            });
        }
        commands.entity(eid).remove::<ColliderStaticStubs>();
    }
    // Triggers
    for (eid, stubs) in trigger_stubs.iter() {
        for stub in stubs.0.iter() {
            commands.entity(eid).with_children(|parent| {
                parent.spawn((
                    UIdMarker(stub.uid),
                    ColliderTriggerBundle {
                        trigger: ColliderTrigger,
                        boundary: ColliderBoundary::from_points(stub.points.clone()),
                        active: ColliderActive(stub.active),
                    },
                    Name::new("ColliderTrigger"),
                ));
            });
        }
        commands.entity(eid).remove::<ColliderTriggerStubs>();
    }
}

/// A helper function to resolve collisions between an IntDyno and a ColliderStatic
pub(super) fn resolve_static_collisions(
    dyno: &mut IntDyno,
    statics: &Query<(
        Entity,
        &ColliderBoundary,
        &ColliderStatic,
        Option<&ColliderActive>,
    )>,
) -> bool {
    let mut fpos = dyno.fpos.truncate();
    let mut min_dist_sq: Option<f32> = None;
    let mut min_point: Option<Vec2> = None;
    let mut min_id: Option<Entity> = None;
    for (id, boundary, _, active) in statics.iter() {
        if active.is_some() && !active.unwrap().0 {
            continue;
        }
        // We use bounding circles to cut down on the number of checks we actually have to do
        let prune_dist = fpos.distance_squared(boundary.center) - dyno.radius.powi(2);
        if prune_dist > boundary.bound_squared {
            continue;
        }
        let closest_point = boundary.closest_point(fpos);
        let dist_sq = fpos.distance_squared(closest_point);
        if min_dist_sq.is_none() || min_dist_sq.unwrap() > dist_sq {
            min_dist_sq = Some(dist_sq);
            min_point = Some(closest_point);
            min_id = Some(id);
        }
    }
    // Early exit when there's no collision
    if min_dist_sq.unwrap_or(f32::MAX) > dyno.radius.powi(2) {
        return false;
    }
    let (_, Some(min_point), Some(min_id)) = (min_dist_sq, min_point, min_id) else {
        error!("Weird stuff happened in resolving static collisions...");
        return false;
    };
    let Ok((_, _, stat, _)) = statics.get(min_id) else {
        error!("Weird stuff2 happened in resolving static collisions...");
        return false;
    };

    if dyno.statics.len() < MAX_COLLISIONS_PER_FRAME {
        dyno.statics.push(min_id);
    }
    let diff = fpos - min_point;
    let normal = diff.normalize_or_zero();
    if normal.dot(dyno.vel) >= 0.0 {
        println!("exiting here");
        return false;
    }
    println!("not exiting here");

    let pure_parr = -1.0 * dyno.vel.dot(normal) * normal + dyno.vel;
    let new_vel =
        pure_parr * (1.0 - stat.friction) - 1.0 * dyno.vel.dot(normal) * normal * stat.bounciness;
    println!("new_vel.length(): {:?}", new_vel.length());
    dyno.vel = new_vel;
    let diff = fpos - min_point;
    let normal = diff.normalize_or_zero();
    fpos += normal * (dyno.radius - fpos.distance(min_point));
    dyno.fpos.x = fpos.x;
    dyno.fpos.y = fpos.y;
    true

    // // We want to move the dyno out of the rock, but also snap it to an integer position
    // let diff = fpos - min_point;
    // let normal = diff.normalize_or_zero();
    // let mut rounded = fpos;
    // // let mut rounded = fpos.round();
    // while normal.length_squared() > 0.1 && rounded.distance(min_point) < dyno.radius + 0.1 {
    //     fpos += normal;
    //     rounded = fpos.round();
    // }
    // fpos = rounded;
    // dyno.pos.x = fpos.x.round() as i32;
    // dyno.pos.y = fpos.y.round() as i32;
    // dyno.rem = Vec2::ZERO;
    // // Now we apply forces to the velocity
    // let pure_parr = -1.0 * dyno.vel.dot(normal) * normal + dyno.vel;
    // let fixed_normal = if dyno.vel.dot(normal) < 0.0 {
    //     normal
    // } else {
    //     -normal
    // };
    // let new_vel = pure_parr * (1.0 - stat.friction)
    //     - 1.0 * dyno.vel.dot(fixed_normal) * normal * stat.bounciness;
    // dyno.vel = new_vel;
    // if dyno.statics.len() < MAX_COLLISIONS_PER_FRAME {
    //     dyno.statics.push(min_id);
    // }
    // true
}

/// A helper function to resolve collisions between an IntDyno and a ColliderStatic
pub(super) fn resolve_trigger_collisions(
    dyno: &mut IntDyno,
    triggers: &Query<(
        Entity,
        &ColliderBoundary,
        &ColliderTrigger,
        Option<&ColliderActive>,
    )>,
) {
    let fpos = dyno.fpos.truncate();
    for (id, boundary, _, active) in triggers.iter() {
        if active.is_some() && !active.unwrap().0 {
            continue;
        }
        // We use bounding circles to cut down on the number of checks we actually have to do
        let prune_dist = fpos.distance_squared(boundary.center) - dyno.radius.powi(2);
        if prune_dist > boundary.bound_squared {
            continue;
        }
        let em = boundary.effective_mult(dyno.fpos.truncate(), dyno.radius);
        if em > 0.001 {
            if dyno.triggers.len() < MAX_COLLISIONS_PER_FRAME {
                dyno.triggers.push((id, em));
            }
        }
    }
}

pub fn update_triggers(
    mut dynos: Query<&mut IntDyno>,
    triggers: Query<(
        Entity,
        &ColliderBoundary,
        &ColliderTrigger,
        Option<&ColliderActive>,
    )>,
) {
    for mut dyno in dynos.iter_mut() {
        resolve_trigger_collisions(dyno.as_mut(), &triggers);
    }
}
