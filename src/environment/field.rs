use crate::{
    drawing::animation::{AnimationManager, SpriteInfo},
    meta::level_data::{ExportedField, Rehydrate},
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
    pub field: Field,
    pub spatial: SpatialBundle,
    pub anim: AnimationManager,
    pub trigger_stubs: ColliderTriggerStubs,
    pub name: Name,
}

impl Rehydrate<FieldBundle> for ExportedField {
    fn rehydrate(self) -> FieldBundle {
        let field = Field {
            dir: self.dir,
            strength: self.strength,
            drag: self.drag,
        };
        // let center = icenter(&self.points);
        let mut spatial = SpatialBundle::default();
        spatial.transform.translation.z = -10.0;
        let mut anim = AnimationManager::single_static(SpriteInfo {
            path: "sprites/field/field_bg.png".to_string(),
            size: UVec2::new(12, 12),
        });
        anim.set_points(self.points.clone());
        anim.set_scroll(field.dir * 0.1);
        let trigger = ColliderTriggerStub {
            uid: fresh_uid(),
            refresh_period: 0,
            points: self.points.clone(),
            active: true,
        };
        FieldBundle {
            field,
            spatial,
            anim,
            trigger_stubs: ColliderTriggerStubs(vec![trigger]),
            name: Name::new("Field"),
        }
    }
}
