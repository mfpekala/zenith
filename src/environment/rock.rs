use crate::{
    drawing::{animation::SpriteInfo, bordered_mesh::BorderedMesh},
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
    MagLev,
}
impl RockKind {
    fn bounciness(&self) -> f32 {
        match *self {
            Self::Normal => 0.8,
            Self::SimpleKill => 0.1,
            Self::MagLev => 0.0,
        }
    }

    fn friction(&self) -> f32 {
        match *self {
            Self::Normal => 0.3,
            Self::SimpleKill => 0.9,
            Self::MagLev => 0.0,
        }
    }

    pub fn to_sprite_infos(&self) -> (SpriteInfo, SpriteInfo) {
        match *self {
            Self::Normal => (
                SpriteInfo {
                    path: "textures/rock/normal_inner.png".to_string(),
                    size: UVec2::new(36, 36),
                },
                SpriteInfo {
                    path: "textures/rock/normal_outer.png".to_string(),
                    size: UVec2::new(36, 36),
                },
            ),
            Self::SimpleKill => (
                SpriteInfo {
                    path: "textures/rock/kill_inner.png".to_string(),
                    size: UVec2::new(36, 36),
                },
                SpriteInfo {
                    path: "textures/rock/kill_outer.png".to_string(),
                    size: UVec2::new(36, 36),
                },
            ),
            Self::MagLev => (
                SpriteInfo {
                    path: "textures/rock/maglev_inner.png".to_string(),
                    size: UVec2::new(36, 36),
                },
                SpriteInfo {
                    path: "textures/rock/maglev_outer.png".to_string(),
                    size: UVec2::new(36, 36),
                },
            ),
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
impl std::fmt::Display for RockKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match *self {
            Self::Normal => "normal",
            Self::SimpleKill => "simple_kill",
            Self::MagLev => "maglev",
        };
        write!(f, "{}", s)
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
    pub bm: BorderedMesh,
    pub static_stubs: ColliderStaticStubs,
    pub name: Name,
}

impl Rehydrate<RockBundle> for ExportedRock {
    fn rehydrate(self) -> RockBundle {
        let rock = Rock { kind: self.kind };
        let spatial = SpatialBundle::default();
        let key = self.kind.to_string();
        let (inner, outer) = self.kind.to_sprite_infos();
        let bm = BorderedMesh::new(vec![(key.clone(), inner)], vec![(key.clone(), outer)], 7.0);
        let collider = self.kind.to_collider_stub(self.points.clone());
        RockBundle {
            rock,
            spatial,
            bm,
            static_stubs: ColliderStaticStubs(vec![collider]),
            name: Name::new("Rock"),
        }
    }
}
