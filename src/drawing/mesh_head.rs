use bevy::{prelude::*, render::view::RenderLayers};
use serde::{Deserialize, Serialize};

use crate::{
    editor::save::SaveMarker,
    uid::{fresh_uid, UId, UIdMarker, UIdTranslator},
};

use super::{
    mesh::{generate_new_sprite_mesh, outline_int_points, uvec2_bound},
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
        let scroll_basis = match head.texture_kind {
            MeshTextureKind::Repeating(size) => size.as_vec2(),
            _ => Vec2::ONE,
        };
        (
            ScrollSpriteMat {
                vel: Vec2::new(
                    head.scroll.x / scroll_basis.x,
                    head.scroll.y / scroll_basis.y,
                ),
            },
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

pub struct BorderedMeshHeadStub {
    pub uid: UId,
    pub head: BorderedMeshHead,
}

#[derive(Component, Default)]
pub struct BorderedMeshHeadStubs(pub Vec<BorderedMeshHeadStub>);

#[derive(Component, Debug, Default, Clone, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct BorderedMeshHead {
    pub inner_path: String,
    pub inner_size: UVec2,
    pub outer_path: String,
    pub outer_size: UVec2,
    pub points: Vec<IVec2>,
    pub border_width: f32,
    pub offset: Vec3,
    pub render_layers: Vec<u8>,
    pub scroll: Vec2,
}

#[derive(Bundle, Default)]
pub struct BorderedMeshHeadBundle {
    pub head: BorderedMeshHead,
    spatial: SpatialBundle,
    save: SaveMarker,
}
impl BorderedMeshHeadBundle {
    pub fn from_head(head: BorderedMeshHead) -> Self {
        Self { head, ..default() }
    }
}

#[derive(Component, Default, Clone, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct BorderedMeshBody {
    last_head: BorderedMeshHead,
    inner_uid: UId,
    outer_uid: UId,
}

pub(super) fn resolve_bordered_mesh_head_stubs(
    mut commands: Commands,
    stubs: Query<(Entity, &BorderedMeshHeadStubs)>,
) {
    for (eid, stubs) in stubs.iter() {
        for stub in stubs.0.iter() {
            commands.entity(eid).with_children(|parent| {
                parent.spawn((
                    UIdMarker(stub.uid),
                    BorderedMeshHeadBundle::from_head(stub.head.clone()),
                ));
            });
        }
        commands.entity(eid).remove::<BorderedMeshHeadStubs>();
    }
}

pub(super) fn update_bordered_mesh_heads(
    mut commands: Commands,
    heads: Query<(Entity, &BorderedMeshHead, Option<&Children>)>,
    mut bodies: Query<&mut BorderedMeshBody>,
    mut controlled: Query<&mut MeshHead>,
    ut: Res<UIdTranslator>,
) {
    for (eid, head, children) in heads.iter() {
        match children {
            Some(children) => {
                for child in children {
                    let Ok(mut body) = bodies.get_mut(*child) else {
                        commands.entity(eid).remove::<Children>();
                        continue;
                    };
                    if &body.last_head == head {
                        continue;
                    }

                    let inner_eid = ut.get_entity(body.inner_uid);
                    let outer_eid = ut.get_entity(body.outer_uid);
                    if inner_eid.is_none() || outer_eid.is_none() {
                        continue;
                    }
                    let inner_eid = inner_eid.unwrap();
                    let outer_eid = outer_eid.unwrap();

                    let inner_head = controlled.get_mut(inner_eid);
                    if let Ok(mut inner_head) = inner_head {
                        inner_head.points = outline_int_points(&head.points, -head.border_width);
                        inner_head.path = head.inner_path.clone();
                        inner_head.texture_kind = MeshTextureKind::Repeating(head.inner_size);
                        inner_head.render_layers = head.render_layers.clone();
                        inner_head.scroll = head.scroll;
                    }

                    let outer_head = controlled.get_mut(outer_eid);
                    if let Ok(mut outer_head) = outer_head {
                        outer_head.points = head.points.clone();
                        outer_head.path = head.outer_path.clone();
                        outer_head.texture_kind = MeshTextureKind::Repeating(head.outer_size);
                        outer_head.render_layers = head.render_layers.clone();
                        outer_head.scroll = head.scroll;
                    }

                    body.last_head = head.clone();
                }
            }
            None => {
                // First make the mesh head stubs and group them
                let inner_uid = fresh_uid();
                let outer_uid = fresh_uid();
                let inner_head = MeshHeadStub {
                    uid: inner_uid,
                    head: MeshHead {
                        path: head.inner_path.clone(),
                        points: outline_int_points(&head.points, -head.border_width),
                        render_layers: head.render_layers.clone(),
                        texture_kind: MeshTextureKind::Repeating(head.inner_size),
                        ..default()
                    },
                };
                let outer_head = MeshHeadStub {
                    uid: outer_uid,
                    head: MeshHead {
                        path: head.outer_path.clone(),
                        points: head.points.clone(),
                        render_layers: head.render_layers.clone(),
                        texture_kind: MeshTextureKind::Repeating(head.outer_size),
                        offset: Vec3::new(0.0, 0.0, -0.5),
                        ..default()
                    },
                };
                let mesh_head_stubs = MeshHeadStubs(vec![inner_head, outer_head]);

                // Then spawn the bordered body with the stubs
                let mut bordered_body_eid = Entity::PLACEHOLDER;
                commands.entity(eid).with_children(|parent| {
                    bordered_body_eid = parent
                        .spawn((
                            SpatialBundle::from_transform(Transform::from_translation(head.offset)),
                            BorderedMeshBody {
                                last_head: head.clone(),
                                inner_uid,
                                outer_uid,
                            },
                            SaveMarker,
                            mesh_head_stubs,
                        ))
                        .id();
                });
            }
        }
    }
}

#[derive(Component, Debug, Clone, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct ScrollSpriteMat {
    pub vel: Vec2,
}

pub(super) fn scroll_sprite_materials(
    sprites_q: Query<(&ScrollSpriteMat, &Handle<SpriteMaterial>)>,
    mut mats: ResMut<Assets<SpriteMaterial>>,
) {
    for (scroll, mat_hand) in sprites_q.iter() {
        let Some(mat) = mats.get_mut(mat_hand.id()) else {
            continue;
        };
        mat.x -= scroll.vel.x;
        mat.y += scroll.vel.y;
        mat.x = mat.x.rem_euclid(1.0);
        mat.y = mat.y.rem_euclid(1.0);
    }
}
