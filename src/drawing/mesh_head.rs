use bevy::{prelude::*, render::view::RenderLayers};
use serde::{Deserialize, Serialize};

use crate::{
    editor::save::SaveMarker,
    uid::{UId, UIdMarker},
};

use super::{
    mesh::{generate_new_sprite_mesh, uvec2_bound},
    sprite_mat::SpriteMaterial,
};

pub struct MeshHeadStub {
    pub uid: UId,
    pub head: MeshHead,
}

#[derive(Component, Default)]
pub struct MeshHeadStubs(pub Vec<MeshHeadStub>);

#[derive(Default, Clone, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize)]
pub enum MeshTextureKind {
    Repeating(UVec2),
    #[default]
    Grow,
}

#[derive(Component, Default, Clone, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct MeshHead {
    pub path: String,
    pub points: Vec<IVec2>,
    pub render_layers: Vec<u8>,
    pub hidden: bool,
    pub offset: Vec3,
    pub texture_kind: MeshTextureKind,
    pub scroll: Vec2,
}

#[derive(Bundle, Default)]
pub struct MeshHeadBundle {
    pub head: MeshHead,
    spatial: SpatialBundle,
    save: SaveMarker,
}
impl MeshHeadBundle {
    pub fn from_head(head: MeshHead) -> Self {
        Self { head, ..default() }
    }
}

#[derive(Component)]
pub(super) struct MeshBody {
    last_head: MeshHead,
}

pub(super) fn resolve_mesh_head_stubs(
    mut commands: Commands,
    stubs: Query<(Entity, &MeshHeadStubs)>,
) {
    for (eid, stubs) in stubs.iter() {
        for stub in stubs.0.iter() {
            commands.entity(eid).with_children(|parent| {
                parent.spawn((
                    UIdMarker(stub.uid),
                    MeshHeadBundle::from_head(stub.head.clone()),
                ));
            });
        }
        commands.entity(eid).remove::<MeshHeadStubs>();
    }
}

pub(super) fn update_mesh_heads(
    mut commands: Commands,
    heads: Query<(Entity, &MeshHead, Option<&Children>)>,
    bodies: Query<(Entity, &mut MeshBody)>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<SpriteMaterial>>,
) {
    let mut get_bund = |head: &MeshHead| {
        let fpoints: Vec<Vec2> = head
            .points
            .clone()
            .into_iter()
            .map(|p| p.as_vec2())
            .collect();
        let (pos, size) = match head.texture_kind {
            MeshTextureKind::Repeating(img_size) => {
                let bounds = uvec2_bound(&fpoints);
                let (pos, size) = SpriteMaterial::random_sized_bounds(bounds, img_size);
                (Some(pos), Some(size))
            }
            MeshTextureKind::Grow => (None, None),
        };
        let ass = asset_server.load(&head.path);
        let mat = SpriteMaterial::from_handle(ass, pos, size);
        let mat = mats.add(mat);
        let mut mesh = generate_new_sprite_mesh(&fpoints, &mat, &mut meshes);
        mesh.transform.translation = head.offset;
        mesh.visibility = if head.hidden {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        (
            mesh,
            RenderLayers::from_layers(&head.render_layers),
            MeshBody {
                last_head: head.clone(),
            },
        )
    };
    for (eid, head, children) in heads.iter() {
        match children {
            Some(children) => {
                for child in children {
                    let Ok((cid, body)) = bodies.get(*child) else {
                        commands.entity(eid).remove::<Children>();
                        continue;
                    };
                    if &body.last_head == head {
                        continue;
                    }
                    let bund = get_bund(head);
                    commands.entity(cid).insert(bund);
                }
            }
            None => {
                let bund = get_bund(head);
                commands.entity(eid).with_children(|parent| {
                    parent.spawn(bund);
                });
            }
        }
    }
}
