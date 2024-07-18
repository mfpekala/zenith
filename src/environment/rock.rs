use crate::{
    drawing::animation::{MultiAnimationManager, SpriteInfo},
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
            Self::Normal => 0.7,
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
                    ..default()
                },
                SpriteInfo {
                    path: "textures/rock/normal_outer.png".to_string(),
                    size: UVec2::new(36, 36),
                    ..default()
                },
            ),
            Self::SimpleKill => (
                SpriteInfo {
                    path: "textures/rock/kill_inner.png".to_string(),
                    size: UVec2::new(36, 36),
                    ..default()
                },
                SpriteInfo {
                    path: "textures/rock/kill_outer.png".to_string(),
                    size: UVec2::new(36, 36),
                    ..default()
                },
            ),
            Self::MagLev => (
                SpriteInfo {
                    path: "textures/rock/maglev_inner.png".to_string(),
                    size: UVec2::new(36, 36),
                    ..default()
                },
                SpriteInfo {
                    path: "textures/rock/maglev_outer.png".to_string(),
                    size: UVec2::new(36, 36),
                    ..default()
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

    pub fn to_collision_sound_path(&self) -> String {
        match self {
            Self::Normal => "sound_effects/normal_rock.ogg".to_string(),
            Self::SimpleKill => "sound_effects/normal_rock.ogg".to_string(),
            Self::MagLev => "sound_effects/normal_rock.ogg".to_string(),
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
    pub name: Name,
    pub rock: Rock,
    pub spatial: SpatialBundle,
    pub multi: MultiAnimationManager,
    pub static_stubs: ColliderStaticStubs,
}
impl RockBundle {
    pub fn new(kind: RockKind, points: Vec<IVec2>) -> Self {
        let rock = Rock { kind };
        let spatial = SpatialBundle::default();
        let (inner, outer) = kind.to_sprite_infos();
        let multi = MultiAnimationManager::bordered_mesh(points.clone(), inner, outer, 6.0);
        let collider = kind.to_collider_stub(points.clone());
        RockBundle {
            name: Name::new("rock"),
            rock,
            spatial,
            multi,
            static_stubs: ColliderStaticStubs(vec![collider]),
        }
    }
}
