use bevy::prelude::*;

use crate::{
    physics::collider::{ColliderActive, ColliderTriggerStub, ColliderTriggerStubs},
    ship::Ship,
    uid::fresh_uid,
};

#[derive(Component, Reflect)]
pub struct LivePolyMarker {
    pub ready: bool,
}

#[derive(Bundle)]
pub struct LivePolyBundle {
    marker: LivePolyMarker,
    pub spatial: SpatialBundle,
    pub trigger_stubs: ColliderTriggerStubs,
    pub name: Name,
}
impl LivePolyBundle {
    pub fn new(all_points: Vec<Vec2>) -> Self {
        let mut min = Vec2::MAX;
        let mut max = Vec2::MIN;
        for point in all_points {
            min = min.min(point);
            max = max.max(point);
        }
        min -= Vec2::ONE * Ship::radius() * 5.0;
        max += Vec2::ONE * Ship::radius() * 5.0;
        let points = vec![min, Vec2::new(min.x, max.y), max, Vec2::new(max.x, min.y)]
            .into_iter()
            .map(|v| IVec2::new(v.round().x as i32, v.round().y as i32))
            .collect::<Vec<_>>();
        let trigger = ColliderTriggerStub {
            uid: fresh_uid(),
            refresh_period: 0,
            points: points.clone(),
            active: true,
        };
        Self {
            marker: LivePolyMarker { ready: false },
            spatial: SpatialBundle::default(),
            trigger_stubs: ColliderTriggerStubs(vec![trigger]),
            name: Name::new("live_poly"),
        }
    }
}

pub(super) fn mark_live_polys_ready(
    mut live_polys: Query<(&mut LivePolyMarker, Option<&Children>)>,
    active_colliders: Query<&ColliderActive>,
) {
    for (mut poly, kids) in live_polys.iter_mut() {
        let Some(kids) = kids else {
            poly.ready = false;
            continue;
        };
        for kid in kids {
            if active_colliders.get(*kid).is_ok() {
                poly.ready = true;
                continue;
            }
        }
    }
}
