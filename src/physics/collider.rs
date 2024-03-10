use bevy::prelude::*;

use crate::{math::MathLine, meta::consts::MAX_COLLISIONS_PER_FRAME};

use super::dyno::{move_int_dynos, IntDyno};

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

    pub fn closest_point(&self, point: &IVec2) -> Vec2 {
        let point = point.as_vec2();
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

    pub fn effective_mult(&self, point: &IVec2, radius: f32) -> f32 {
        let mut max_dist = f32::MIN;
        let fpos = point.as_vec2();
        for line in self.lines.iter() {
            let signed_dist = line.signed_distance_from_point(&fpos);
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

#[derive(Bundle)]
pub struct ColliderStaticBundle {
    _static: ColliderStatic,
    boundary: ColliderBoundary,
    active: ColliderActive,
}
impl ColliderStaticBundle {
    pub fn new(stat: ColliderStatic, boundary_points: Vec<IVec2>, active: bool) -> Self {
        Self {
            _static: stat,
            boundary: ColliderBoundary::from_points(boundary_points),
            active: ColliderActive(active),
        }
    }
}

#[derive(Bundle)]
pub struct ColliderTriggerBundle {
    _trigger: ColliderTrigger,
    boundary: ColliderBoundary,
    active: ColliderActive,
}
impl ColliderTriggerBundle {
    pub fn new(boundary_points: Vec<IVec2>, active: bool) -> Self {
        Self {
            _trigger: ColliderTrigger,
            boundary: ColliderBoundary::from_points(boundary_points),
            active: ColliderActive(active),
        }
    }
}

/// A helper function to resolve collisions between an IntDyno and a ColliderStatic
pub fn resolve_static_collisions(
    dyno: &mut IntDyno,
    statics: &Query<(
        Entity,
        &ColliderBoundary,
        &ColliderStatic,
        Option<&ColliderActive>,
    )>,
) -> bool {
    let mut fpos = dyno.pos.as_vec2();
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
        let closest_point = boundary.closest_point(&dyno.pos);
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
    // We want to move the dyno out of the rock, but also snap it to an integer position
    let diff = fpos - min_point;
    let normal = diff.normalize();
    let mut rounded = fpos.round();
    while rounded.distance(min_point) < dyno.radius + 0.1 {
        fpos += normal;
        rounded = fpos.round();
    }
    fpos = rounded;
    dyno.pos = IVec2 {
        x: fpos.x.round() as i32,
        y: fpos.y.round() as i32,
    };
    dyno.rem = Vec2::ZERO;
    // Now we apply forces to the velocity
    let pure_parr = -1.0 * dyno.vel.dot(normal) * normal + dyno.vel;
    let fixed_normal = if dyno.vel.dot(normal) < 0.0 {
        normal
    } else {
        -normal
    };
    let new_vel = pure_parr * (1.0 - stat.friction)
        - 1.0 * dyno.vel.dot(fixed_normal) * normal * stat.bounciness;
    dyno.vel = new_vel;
    if dyno.statics.len() < MAX_COLLISIONS_PER_FRAME {
        dyno.statics.push(min_id);
    }
    true
}

/// A helper function to resolve collisions between an IntDyno and a ColliderStatic
pub fn resolve_trigger_collisions(
    dyno: &mut IntDyno,
    triggers: &Query<(
        Entity,
        &ColliderBoundary,
        &ColliderTrigger,
        Option<&ColliderActive>,
    )>,
) {
    let fpos = dyno.pos.as_vec2();
    for (id, boundary, _, active) in triggers.iter() {
        if active.is_some() && !active.unwrap().0 {
            continue;
        }
        // We use bounding circles to cut down on the number of checks we actually have to do
        let prune_dist = fpos.distance_squared(boundary.center) - dyno.radius.powi(2);
        if prune_dist > boundary.bound_squared {
            continue;
        }
        let em = boundary.effective_mult(&dyno.pos, dyno.radius);
        if em > 0.001 {
            if dyno.triggers.len() < MAX_COLLISIONS_PER_FRAME {
                dyno.triggers.push((id, em));
            }
        }
    }
}

/// TODO: A function to go through all the dynos and mark ids of trigger colliders they are in
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

pub fn register_colliders(app: &mut App) {
    app.add_systems(FixedUpdate, update_triggers.after(move_int_dynos));
}
