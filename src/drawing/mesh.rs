use bevy::{
    prelude::*,
    render::{
        mesh::Indices, render_asset::RenderAssetUsages, render_resource::PrimitiveTopology,
        view::RenderLayers,
    },
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use crate::meta::consts::{SCREEN_HEIGHT, SCREEN_WIDTH};

use super::sprite_mat::SpriteMaterial;

fn points_to_mesh(points: &[Vec2], meshes: &mut ResMut<Assets<Mesh>>) -> Mesh2dHandle {
    let mut points_vec: Vec<f32> = vec![];
    let mut top_left = Vec2::new(f32::MAX, f32::MAX);
    let mut bot_right = Vec2::new(f32::MIN, f32::MIN);
    for point in points.iter() {
        points_vec.push(point.x);
        points_vec.push(point.y);
        top_left.x = top_left.x.min(point.x);
        top_left.y = top_left.y.min(point.y);
        bot_right.x = bot_right.x.max(point.x);
        bot_right.y = bot_right.y.max(point.y);
    }
    let verts: Vec<u32> = earcutr::earcut(&points_vec, &[], 2)
        .unwrap()
        .into_iter()
        .map(|val| val as u32)
        .collect();
    let mut triangle = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    let mut positions: Vec<[f32; 3]> = vec![];
    let mut normals: Vec<[f32; 3]> = vec![];
    let mut uvs: Vec<[f32; 2]> = vec![];
    for p in points.iter() {
        positions.push([p.x, p.y, 0.0]);
        normals.push([0.0, 0.0, 1.0]);
        let uv_x = (p.x - top_left.x) / (bot_right.x - top_left.x);
        let uv_y = (p.y - top_left.y) / (bot_right.y - top_left.y);
        uvs.push([uv_x, uv_y]);
    }
    triangle.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    triangle.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    triangle.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    triangle.insert_indices(Indices::U32(verts));
    meshes.add(triangle).into()
}

pub fn generate_new_color_mesh(
    points: &[Vec2],
    mat: &Handle<ColorMaterial>,
    meshes: &mut ResMut<Assets<Mesh>>,
) -> MaterialMesh2dBundle<ColorMaterial> {
    let mesh_handle = points_to_mesh(points, meshes);
    MaterialMesh2dBundle {
        mesh: mesh_handle,
        material: mat.clone(),
        ..default()
    }
}

pub fn generate_new_sprite_mesh(
    points: &[Vec2],
    mat: &Handle<SpriteMaterial>,
    meshes: &mut ResMut<Assets<Mesh>>,
) -> MaterialMesh2dBundle<SpriteMaterial> {
    let mesh_handle = points_to_mesh(points, meshes);
    MaterialMesh2dBundle {
        mesh: mesh_handle,
        material: mat.clone(),
        ..default()
    }
}

/// Returns a mesh that covers the screen
pub fn generate_new_screen_mesh(meshes: &mut ResMut<Assets<Mesh>>) -> Mesh2dHandle {
    let x = SCREEN_WIDTH as f32 / 2.0;
    let y = SCREEN_HEIGHT as f32 / 2.0;
    let points = vec![
        Vec2::new(-x, -y),
        Vec2::new(-x, y),
        Vec2::new(x, y),
        Vec2::new(x, -y),
    ];
    points_to_mesh(&points, meshes)
}

#[derive(Component)]
pub struct MeshOutline {
    pub width: f32,
    pub color: Color,
}

#[derive(Bundle)]
pub struct MeshOutlineBundle {
    outline: MeshOutline,
    mesh: MaterialMesh2dBundle<ColorMaterial>,
}
impl MeshOutlineBundle {
    pub fn new(
        outline: MeshOutline,
        points: &Vec<Vec2>,
        mats: &mut ResMut<Assets<ColorMaterial>>,
        meshes: &mut ResMut<Assets<Mesh>>,
    ) -> Self {
        let mut center = Vec2::ZERO;
        for point in points {
            center += *point;
        }
        center /= points.len() as f32;
        let mut new_points = vec![];
        for point in points {
            let diff = *point - center;
            new_points.push(*point + diff.normalize_or_zero() * outline.width);
        }
        let mesh = points_to_mesh(&new_points, meshes);
        let mat = mats.add(ColorMaterial::from(outline.color));
        Self {
            outline,
            mesh: MaterialMesh2dBundle {
                material: mat,
                mesh,
                transform: Transform::from_translation(Vec2::ZERO.extend(-0.5)),
                ..default()
            },
        }
    }
}

// fn update_mesh_outlines(
//     mesh_res: Res<Assets<Mesh>>,
//     meshes_handles: Query<(&Transform, &Mesh2dHandle), Without<MeshOutline>>,
//     mut outlines: Query<
//         (&Parent, &mut Transform, &MeshOutline, &mut Mesh2dHandle),
//         With<MeshOutline>,
//     >,
// ) {
//     for (parent, mut tran, outline, mut handle) in outlines.iter_mut() {
//         let parent_id = parent.get();
//         let Ok((parent_tran, parent_mesh_handle)) = meshes_handles.get(parent_id) else {
//             continue;
//         };
//         let test = mesh_res.get(parent_mesh_handle.0.id()).unwrap().clone();
//         let points = test.attribute(Mesh::ATTRIBUTE_POSITION).unwrap();
//         println!("points: {:?}", points);

//         *handle = parent_mesh_handle.clone();
//         tran.scale = parent_tran.scale * outline.scale;
//         tran.translation.z = -0.5;
//     }
// }

pub struct MyMeshPlugin;

impl Plugin for MyMeshPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(Update, update_mesh_outlines);
    }
}
