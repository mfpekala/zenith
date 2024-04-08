use crate::{
    drawing::{
        layering::sprite_layer_u8,
        mesh_head::{MeshHead, MeshHeadStub, MeshHeadStubs, MeshTextureKind},
    },
    math::{icenter, irecenter},
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
            Self::Normal => 1.0,
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
    pub mesh_stubs: MeshHeadStubs,
    pub trigger_stubs: ColliderTriggerStubs,
}

impl Rehydrate<FieldBundle> for ExportedField {
    fn rehydrate(self) -> FieldBundle {
        let field = Field {
            dir: self.dir,
            strength: self.strength,
            drag: self.drag,
        };
        let center = icenter(&self.points);
        let new_points = irecenter(self.points, &center);
        let spatial = SpatialBundle::from_transform(Transform::from_translation(
            center.as_vec2().extend(0.0),
        ));
        let mesh = MeshHeadStub {
            uid: fresh_uid(),
            head: MeshHead {
                path: "sprites/field/field_bg.png".to_string(),
                points: new_points.clone(),
                render_layers: vec![sprite_layer_u8()],
                texture_kind: MeshTextureKind::Repeating(UVec2::new(12, 12)),
                scroll: self.dir,
                ..default()
            },
        };
        let trigger = ColliderTriggerStub {
            uid: fresh_uid(),
            points: new_points,
            active: true,
        };
        FieldBundle {
            field,
            spatial,
            mesh_stubs: MeshHeadStubs(vec![mesh]),
            trigger_stubs: ColliderTriggerStubs(vec![trigger]),
        }
    }
}
