use bevy::{prelude::*, render::view::RenderLayers, sprite::MaterialMesh2dBundle};

use crate::{
    camera::{update_camera, CameraMarker, CameraMode},
    physics::move_dynos,
    ship::Ship,
};

use super::lightmap::sprite_layer;

#[derive(Component)]
pub struct PixelMesh;

#[derive(Bundle)]
pub struct PixelMeshBundle {
    pub pixel_mesh_marker: PixelMesh,
    pub mesh: MaterialMesh2dBundle<ColorMaterial>,
    pub render_layers: RenderLayers,
}
impl PixelMeshBundle {
    pub fn new(mesh: MaterialMesh2dBundle<ColorMaterial>) -> Self {
        Self {
            pixel_mesh_marker: PixelMesh,
            mesh,
            render_layers: sprite_layer(),
        }
    }
}

fn snap_pixel_meshes(
    mut pixel_meshes: Query<(&Parent, &mut Transform), With<PixelMesh>>,
    q_parent: Query<(&GlobalTransform, Option<&Ship>)>,
    cam_marker: Query<&CameraMarker>,
) {
    // let Ok(cam) = cam_marker.get_single() else {
    //     return;
    // };
    // for (parent, mut local_tran) in pixel_meshes.iter_mut() {
    //     let Ok((parent_tran, ship)) = q_parent.get(parent.get()) else {
    //         continue;
    //     };
    //     let parent_pos = parent_tran.translation().truncate();
    //     // TODO: Maybe less jank? Maybe it's ok? Idk
    //     if ship.is_some() && cam.mode == CameraMode::Follow {
    //         local_tran.translation = (cam.pixel_align(cam.fake_pos) - parent_pos).extend(0.0);
    //         continue;
    //     }
    //     local_tran.translation = (cam.pixel_align(parent_pos) - parent_pos).extend(0.0);
    // }
}

pub fn register_pixel_meshes(app: &mut App) {
    app.add_systems(
        Update,
        snap_pixel_meshes.after(update_camera).after(move_dynos),
    );
}
