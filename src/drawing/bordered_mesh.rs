use bevy::{
    prelude::*,
    render::{primitives::Aabb, view::RenderLayers},
    sprite::Mesh2dHandle,
};
use serde::{Deserialize, Serialize};

use crate::physics::dyno::IntMoveable;

use super::{
    mesh::{generate_new_sprite_mesh, outline_points, uvec2_bound, ScrollSprite, SpriteInfo},
    sprite_mat::SpriteMaterial,
};

#[derive(Component, Debug, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct BorderMeshType(pub String);

#[derive(Clone, PartialEq, Debug, Default, Reflect, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize)]
pub struct BorderedMatData {
    pub path: String,
    pub size: UVec2,
}
impl BorderedMatData {
    pub fn new(path: &str, size: UVec2) -> Self {
        Self {
            path: path.to_string(),
            size,
        }
    }
}

#[derive(Component, Debug, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct BorderedMesh {
    pub last_points: Vec<IVec2>,
    pub points: Vec<IVec2>,
    pub last_material: BorderedMatData,
    pub material: BorderedMatData,
    pub last_border_material: Option<BorderedMatData>,
    pub border_material: Option<BorderedMatData>,
    pub border_width: Option<f32>,
    pub scroll: Vec2,
}
impl BorderedMesh {
    fn spawn(
        commands: &mut ChildBuilder,
        meshes: &mut ResMut<Assets<Mesh>>,
        points: Vec<IVec2>,
        material: (BorderedMatData, Handle<SpriteMaterial>),
        border_material: Option<(BorderedMatData, Handle<SpriteMaterial>)>,
        border_width: Option<f32>,
        render_layer: RenderLayers,
        inner_info: SpriteInfo,
        outer_info: Option<SpriteInfo>,
        z_index: i32,
    ) -> Entity {
        let fpoints: Vec<Vec2> = points.clone().into_iter().map(|p| p.as_vec2()).collect();
        let inner_points = outline_points(&fpoints, -border_width.unwrap_or(0.0));
        let mut inner_mesh = generate_new_sprite_mesh(&inner_points, &material.1, meshes);
        inner_mesh.transform.translation.z += 0.5;
        let outer_mesh = match &border_material {
            Some(mat) => Some(generate_new_sprite_mesh(&fpoints, &mat.1, meshes)),
            None => None,
        };
        commands
            .spawn((
                BorderedMesh {
                    last_points: points.clone(),
                    points,
                    last_material: material.0.clone(),
                    material: material.0.clone(),
                    last_border_material: match border_material.clone() {
                        Some(thing) => Some(thing.0),
                        None => None,
                    },
                    border_material: match border_material {
                        Some(thing) => Some(thing.0),
                        None => None,
                    },
                    border_width,
                    scroll: default(),
                },
                IntMoveable::new(IVec2::ZERO.extend(z_index)),
                SpatialBundle::default(),
            ))
            .with_children(|parent| {
                parent.spawn((
                    inner_mesh,
                    render_layer.clone(),
                    BorderMeshType("inner".to_string()),
                    ScrollSprite::default(),
                    inner_info.clone(),
                ));
                if let (Some(outer_mesh), Some(outer_info)) = (outer_mesh, outer_info) {
                    parent.spawn((
                        outer_mesh,
                        render_layer.clone(),
                        BorderMeshType("outer".to_string()),
                        ScrollSprite::default(),
                        outer_info.clone(),
                    ));
                }
            })
            .id()
    }

    pub fn spawn_easy(
        commands: &mut ChildBuilder,
        asset_server: &Res<AssetServer>,
        meshes: &mut ResMut<Assets<Mesh>>,
        mats: &mut ResMut<Assets<SpriteMaterial>>,
        points: Vec<IVec2>,
        material: (&str, UVec2),
        border_material: Option<(&str, UVec2)>,
        border_width: Option<f32>,
        render_layer: RenderLayers,
        z_index: i32,
    ) -> Entity {
        let fpoints: Vec<Vec2> = points.clone().into_iter().map(|p| p.as_vec2()).collect();
        let bounds = uvec2_bound(&fpoints);

        let (inner_pos, inner_size) = SpriteMaterial::random_sized_bounds(bounds, material.1);
        let inner_ass = asset_server.load(material.0.to_string());
        let inner_mat = SpriteMaterial::from_handle(inner_ass, Some(inner_pos), Some(inner_size));
        let inner_mat = mats.add(inner_mat);

        let outer_mat = match border_material {
            Some((path, size)) => {
                let (outer_pos, outer_size) = SpriteMaterial::random_sized_bounds(bounds, size);
                let outer_ass = asset_server.load(path.to_string());
                let outer_mat =
                    SpriteMaterial::from_handle(outer_ass, Some(outer_pos), Some(outer_size));
                let outer_mat = mats.add(outer_mat);
                let helper = (BorderedMatData::new(path, size), outer_mat);
                Some(helper)
            }
            None => None,
        };

        let inner_info = SpriteInfo {
            sprite_size: material.1,
            bounds,
        };
        let outer_info = match border_material {
            Some(thing) => Some(SpriteInfo {
                sprite_size: thing.1,
                bounds,
            }),
            None => None,
        };

        Self::spawn(
            commands,
            meshes,
            points,
            (BorderedMatData::new(material.0, material.1), inner_mat),
            outer_mat,
            border_width,
            render_layer,
            inner_info,
            outer_info,
            z_index,
        )
    }

    pub fn regen(
        &self,
        is_inner: bool,
        asset_server: &Res<AssetServer>,
        meshes: &mut ResMut<Assets<Mesh>>,
        mats: &mut ResMut<Assets<SpriteMaterial>>,
    ) -> Option<(Mesh2dHandle, Handle<SpriteMaterial>)> {
        let fpoints: Vec<Vec2> = self
            .points
            .clone()
            .into_iter()
            .map(|p| p.as_vec2())
            .collect();
        let bounds = uvec2_bound(&fpoints);
        let mat_data = if is_inner {
            self.material.clone()
        } else {
            match self.border_material.clone() {
                Some(thing) => thing,
                None => return None,
            }
        };
        let border_diff = if is_inner {
            self.border_width.unwrap_or(0.0)
        } else {
            0.0
        };

        let (pos, size) = SpriteMaterial::random_sized_bounds(bounds, mat_data.size);
        let ass = asset_server.load(mat_data.path);
        let mat = SpriteMaterial::from_handle(ass, Some(pos), Some(size));
        let mat = mats.add(mat);

        let points = outline_points(&fpoints, -border_diff);
        let bund = generate_new_sprite_mesh(&points, &mat, meshes);
        Some((bund.mesh, bund.material))
    }

    pub fn needs_update(&self) -> bool {
        self.last_points != self.points
            || self.last_material != self.material
            || self.last_border_material != self.border_material
    }

    pub fn complete_update(&mut self) {
        self.last_points = self.points.clone();
        self.last_material = self.material.clone();
        self.last_border_material = self.border_material.clone();
    }
}

/// Every frame translates updates to the BorderedMesh component to it's children
pub(super) fn bordered_mesh_trickle_down(
    mut bms: Query<(&mut BorderedMesh, &Children)>,
    mut handles: Query<(
        &mut Mesh2dHandle,
        &mut Handle<SpriteMaterial>,
        &mut ScrollSprite,
        &BorderMeshType,
    )>,
    asset_server: Res<AssetServer>,
    mut mats: ResMut<Assets<SpriteMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    for (mut bm, children) in bms.iter_mut() {
        for child in children {
            let Ok((mut mesh_handle, mut sprite_handle, mut scroll_sprite, kind)) =
                handles.get_mut(*child)
            else {
                continue;
            };
            // Always set the scroll
            scroll_sprite.vel = bm.scroll;
            if !bm.needs_update() {
                continue;
            }
            // Only regen the mesh/mat if we need to (but do both whenever one changes)
            let is_inner = &kind.0 == "inner";
            let Some((new_mesh_handle, new_sprite_handle)) =
                bm.regen(is_inner, &asset_server, &mut meshes, &mut mats)
            else {
                continue;
            };
            *mesh_handle = new_mesh_handle;
            *sprite_handle = new_sprite_handle;
            commands.entity(*child).remove::<Aabb>();
        }
        bm.complete_update();
    }
}
