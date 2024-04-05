use crate::{
    drawing::{layering::sprite_layer, mesh_head::MeshHeadStubs},
    physics::collider::{ColliderTriggerBundle, ColliderTriggerStubs},
};
use bevy::{prelude::*, render::view::RenderLayers};

#[derive(Default, Debug, Clone, Eq, Hash, PartialEq, serde::Serialize, serde::Deserialize)]
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

#[derive(Default, Debug, Clone, Eq, Hash, PartialEq, serde::Serialize, serde::Deserialize)]
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
