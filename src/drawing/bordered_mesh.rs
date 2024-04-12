use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::{
    animation::{AnimationManager, MultiAnimationManager, SpriteInfo},
    mesh::ioutline_points,
};

#[derive(Component)]
pub struct BorderedMeshBodyMarker;

#[derive(Component, Default, Clone, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct BorderedMesh {
    inners: Vec<(String, SpriteInfo)>,
    outers: Vec<(String, SpriteInfo)>,
    width: f32,
    points: Vec<IVec2>,
    key: String,
    is_changed: bool,
}
impl BorderedMesh {
    pub fn new(
        inners: Vec<(String, SpriteInfo)>,
        outers: Vec<(String, SpriteInfo)>,
        width: f32,
    ) -> Self {
        Self {
            key: inners[0].0.to_string(),
            inners: inners
                .into_iter()
                .map(|pair| (pair.0.to_string(), pair.1))
                .collect(),
            outers: outers
                .into_iter()
                .map(|pair| (pair.0.to_string(), pair.1))
                .collect(),
            width,
            points: vec![],
            is_changed: true,
        }
    }

    pub fn force_change(&mut self) {
        self.is_changed = true;
    }

    pub fn set_points(&mut self, points: Vec<IVec2>) {
        if points == self.points {
            // Do nothing
            return;
        }
        self.points = points;
        self.is_changed = true;
    }

    pub fn set_key(&mut self, key: &str) {
        if &self.key == key {
            // Do nothing
            return;
        }
        self.key = key.to_string();
        self.is_changed = true;
    }
}

pub(super) fn materialize_bordered_meshes(
    mut commands: Commands,
    mut bms: Query<(Entity, &mut BorderedMesh, Option<&Children>)>,
    bodies: Query<&BorderedMeshBodyMarker>,
) {
    for (eid, mut bm, children) in bms.iter_mut() {
        let is_materialized = match children {
            None => false,
            Some(cids) => cids.iter().any(|cid| bodies.get(*cid).is_ok()),
        };
        if is_materialized {
            continue;
        }
        let mut str_inners = vec![];
        for inner in bm.inners.iter() {
            str_inners.push((inner.0.as_str(), inner.1.clone()));
        }
        let mut str_outers = vec![];
        for outer in bm.outers.iter() {
            str_outers.push((outer.0.as_str(), outer.1.clone()));
        }
        let inner = AnimationManager::from_static_pairs(str_inners);
        let mut outer = AnimationManager::from_static_pairs(str_outers);
        outer.set_offset(IVec3::new(0, 0, -1));
        let multi = MultiAnimationManager::from_pairs(vec![("inner", inner), ("outer", outer)]);
        commands.entity(eid).with_children(|parent| {
            parent.spawn((
                Name::new("BMBody - MultiAnimationBundle"),
                multi,
                SpatialBundle::default(),
                BorderedMeshBodyMarker,
            ));
        });
        bm.force_change();
    }
}

pub(super) fn update_bordered_meshes(
    mut bms: Query<&mut BorderedMesh>,
    mut multis: Query<(&mut MultiAnimationManager, &Parent)>,
) {
    for (mut multi, parent) in multis.iter_mut() {
        let Ok(mut bm) = bms.get_mut(parent.get()) else {
            continue;
        };
        if !bm.is_changed {
            return;
        }

        let Some(inner) = multi.map.get_mut("inner") else {
            continue;
        };
        inner.set_points(ioutline_points(&bm.points, -bm.width));
        inner.set_key(&bm.key);

        let Some(outer) = multi.map.get_mut("outer") else {
            continue;
        };
        outer.set_points(bm.points.clone());
        outer.set_key(&bm.key);

        bm.is_changed = false;
    }
}
