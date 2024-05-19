use bevy::{prelude::*, render::render_resource::Extent3d, window::WindowResized};

use crate::{
    camera::ScreenMults,
    drawing::layering::{remake_layering_materials, BlendTexturesMaterial},
    meta::consts::{SCREEN_HEIGHT, SCREEN_WIDTH},
};

use super::{
    animation_mat::AnimationMaterial,
    layering::{CameraTargets, ReducedMaterial, ScaledOutputQuad},
};

pub(super) fn resize_canvases(
    mut events: EventReader<WindowResized>,
    cam_targets: Res<CameraTargets>,
    mut screen_mults: ResMut<ScreenMults>,
    mut images: ResMut<Assets<Image>>,
    mut modified: EventWriter<AssetEvent<Image>>,
    mut materials: ResMut<Assets<BlendTexturesMaterial>>,
    mut dum_materials: ResMut<Assets<ReducedMaterial>>,
    mut anim_materials: ResMut<Assets<AnimationMaterial>>,
    mut scaled_quad: Query<&mut Transform, With<ScaledOutputQuad>>,
) {
    let Some(event) = events.read().last() else {
        return;
    };
    let x_mults = (event.width / (SCREEN_WIDTH as f32)).floor() as u32;
    let y_mults = (event.height / (SCREEN_HEIGHT as f32)).floor() as u32;
    let mults = x_mults.min(y_mults).max(1);
    if mults == screen_mults.0 {
        return;
    }
    screen_mults.0 = mults;
    let image_handles = [&cam_targets.menu_target];
    for image_handle in image_handles {
        let Some(image) = images.get_mut(image_handle.id()) else {
            continue;
        };
        image.resize(Extent3d {
            width: SCREEN_WIDTH as u32 * mults,
            height: SCREEN_HEIGHT as u32 * mults,
            ..default()
        });
        modified.send(AssetEvent::Modified {
            id: image_handle.id(),
        });
    }
    remake_layering_materials(
        &cam_targets,
        &mut materials,
        &mut dum_materials,
        &mut anim_materials,
    );
    if let Ok(mut scaled_quad) = scaled_quad.get_single_mut() {
        scaled_quad.scale = Vec3::ONE * mults as f32;
    };
}
