use crate::{
    drawing::animation::{AnimationManager, SpriteInfo},
    physics::collider::{ColliderTriggerStub, ColliderTriggerStubs},
    uid::fresh_uid,
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, PartialEq, Debug, Clone, Copy, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub enum FieldStrength {
    #[default]
    Normal,
}
impl FieldStrength {
    pub fn to_f32(&self) -> f32 {
        match *self {
            Self::Normal => 0.3,
        }
    }
}

#[derive(Component, PartialEq, Debug, Clone, Copy, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub enum FieldDrag {
    #[default]
    Normal,
}
impl FieldDrag {
    pub fn to_f32(&self) -> f32 {
        match *self {
            Self::Normal => 0.0003,
        }
    }
}

/// NOTE: Points MUST be in clockwise order
#[derive(Component, Clone, Debug)]
pub struct Field {
    pub dir: Vec2,
    pub strength: FieldStrength,
    pub drag: FieldDrag,
}

#[derive(Bundle)]
pub struct FieldBundle {
    pub name: Name,
    pub field: Field,
    pub spatial: SpatialBundle,
    pub anim: AnimationManager,
    pub trigger_stubs: ColliderTriggerStubs,
}
impl FieldBundle {
    pub fn new(field: Field, points: Vec<IVec2>) -> Self {
        let mut spatial = SpatialBundle::default();
        spatial.transform.translation.z = -10.0;
        let mut anim = AnimationManager::single_repeating(
            SpriteInfo {
                path: "sprites/field/field_dyno.png".to_string(),
                size: UVec2::new(8, 8),
                ..default()
            },
            8,
        )
        .force_mat_rot(field.dir.to_angle());
        anim.set_points(points.clone());
        let trigger = ColliderTriggerStub {
            uid: fresh_uid(),
            refresh_period: 0,
            points: points.clone(),
            active: true,
        };
        Self {
            name: Name::new("Field"),
            field,
            spatial,
            anim,
            trigger_stubs: ColliderTriggerStubs(vec![trigger]),
        }
    }
}
