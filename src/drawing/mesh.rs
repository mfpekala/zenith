use bevy::{
    prelude::*,
    render::{mesh::Indices, render_asset::RenderAssetUsages, render_resource::PrimitiveTopology},
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

pub fn generate_new_mesh(
    points: &[Vec2],
    mat: &Handle<ColorMaterial>,
    meshes: &mut ResMut<Assets<Mesh>>,
) -> MaterialMesh2dBundle<ColorMaterial> {
    let mut points_vec: Vec<f32> = vec![];
    for point in points.iter() {
        points_vec.push(point.x);
        points_vec.push(point.y);
    }
    let verts: Vec<u32> = earcutr::earcut(&points_vec, &[], 2)
        .unwrap()
        .into_iter()
        .map(|val| val as u32)
        .collect();
    let mut triangle = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD);
    let mut positions: Vec<[f32; 3]> = vec![];
    let mut normals: Vec<[f32; 3]> = vec![];
    for p in points.iter() {
        positions.push([p.x, p.y, 0.0]);
        normals.push([0.0, 0.0, 1.0]);
    }
    triangle.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    triangle.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    triangle.insert_indices(Indices::U32(verts));
    let mesh_handle: Mesh2dHandle = meshes.add(triangle).into();
    MaterialMesh2dBundle {
        mesh: mesh_handle,
        material: mat.clone(),
        ..default()
    }
}
