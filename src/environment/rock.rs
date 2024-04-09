use crate::{
    drawing::{
        layering::sprite_layer_u8,
        mesh_head::{BorderedMeshHead, BorderedMeshHeadStub, BorderedMeshHeadStubs},
    },
    math::{icenter, irecenter},
    meta::level_data::{ExportedRock, Rehydrate},
    physics::collider::{ColliderStaticStub, ColliderStaticStubs},
    uid::fresh_uid,
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, PartialEq, Debug, Clone, Copy, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub enum RockKind {
    #[default]
    Normal,
    SimpleKill,
}
impl RockKind {
    fn bounciness(&self) -> f32 {
        match *self {
            Self::Normal => 0.8,
            Self::SimpleKill => 0.1,
        }
    }

    fn friction(&self) -> f32 {
        match *self {
            Self::Normal => 0.3,
            Self::SimpleKill => 0.9,
        }
    }

    pub fn to_bm_head(&self, points: Vec<IVec2>) -> BorderedMeshHead {
        let ((inner_path, inner_size), (outer_path, outer_size)) = match *self {
            Self::Normal => {
                let inner = ("textures/play_inner.png".to_string(), UVec2::new(36, 36));
                let outer = ("textures/play_outer.png".to_string(), UVec2::new(36, 36));
                (inner, outer)
            }
            Self::SimpleKill => {
                let inner = ("textures/lava.png".to_string(), UVec2::new(36, 36));
                let outer = ("textures/lava.png".to_string(), UVec2::new(36, 36));
                (inner, outer)
            }
        };
        BorderedMeshHead {
            inner_path,
            outer_path,
            inner_size,
            outer_size,
            points,
            border_width: 7.0,
            render_layers: vec![sprite_layer_u8()],
            ..default()
        }
    }

    pub fn to_collider_stub(&self, points: Vec<IVec2>) -> ColliderStaticStub {
        ColliderStaticStub {
            uid: fresh_uid(),
            points,
            active: true,
            bounciness: self.bounciness(),
            friction: self.friction(),
        }
    }
}

/// NOTE: Points MUST be in clockwise order
#[derive(Component, Clone)]
pub struct Rock {
    pub kind: RockKind,
}

#[derive(Bundle)]
pub struct RockBundle {
    pub rock: Rock,
    pub spatial: SpatialBundle,
    pub bm_mesh_stubs: BorderedMeshHeadStubs,
    pub static_stubs: ColliderStaticStubs,
    pub name: Name,
}

impl Rehydrate<RockBundle> for ExportedRock {
    fn rehydrate(self) -> RockBundle {
        let rock = Rock { kind: self.kind };
        let center = icenter(&self.points);
        let new_points = irecenter(self.points, &center);
        let spatial = SpatialBundle::from_transform(Transform::from_translation(
            center.as_vec2().extend(0.0),
        ));
        let bm_mesh = BorderedMeshHeadStub {
            uid: fresh_uid(),
            head: self.kind.to_bm_head(new_points.clone()),
        };
        let collider = self.kind.to_collider_stub(new_points);
        RockBundle {
            rock,
            spatial,
            bm_mesh_stubs: BorderedMeshHeadStubs(vec![bm_mesh]),
            static_stubs: ColliderStaticStubs(vec![collider]),
            name: Name::new("Rock"),
        }
    }
}
