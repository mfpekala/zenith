use bevy::prelude::*;

use crate::math::MathLine;

use super::dyno::IntDyno;

#[derive(Component, Debug)]
pub struct ColliderBoundary {
    pub points: Vec<IVec2>,
    pub lines: Vec<MathLine>,
    pub center: Vec2,
    pub bound_squared: f32,
}
impl ColliderBoundary {
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

#[derive(Bundle)]
pub struct ColliderTriggerBundle {
    _trigger: ColliderTrigger,
    boundary: ColliderBoundary,
    active: ColliderActive,
}

/// A helper function to resolve collisions between an IntDyno and a ColliderStatic
pub fn resolve_static_collisions(
    dyno: &mut IntDyno,
    statics: &Query<(Entity, &ColliderBoundary, &ColliderStatic)>,
) -> bool {
    let mut fpos = dyno.pos.as_vec2();
    let mut min_dist_sq: Option<f32> = None;
    let mut min_point: Option<Vec2> = None;
    let mut min_id: Option<Entity> = None;
    for (id, boundary, _) in statics.iter() {
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
    let Ok((_, _, stat)) = statics.get(min_id) else {
        error!("Weird stuff2 happened in resolving static collisions...");
        return false;
    };
    // We want to move the dyno out of the rock, but also snap it to an integer position
    let diff = fpos - min_point;
    fpos += diff.normalize() * (dyno.radius - diff.length());
    dyno.pos = IVec2 {
        x: fpos.x.round() as i32,
        y: fpos.y.round() as i32,
    };
    // Now we apply forces to the velocity
    let normal = diff.normalize();
    let pure_parr = -1.0 * dyno.vel.dot(normal) * normal + dyno.vel;
    let new_vel =
        pure_parr * (1.0 - stat.friction) - 1.0 * dyno.vel.dot(normal) * normal * stat.bounciness;
    dyno.vel = new_vel;
    dyno.statics.push(min_id);
    true
}

/// TODO: A function to go through all the dynos and mark ids of trigger colliders they are in
pub fn update_triggers() {}
