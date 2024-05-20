use bevy::{prelude::*, window::WindowResized};

use crate::{
    camera::{ScreenMults, WindowDims},
    meta::consts::{SCREEN_HEIGHT, SCREEN_WIDTH},
};

use super::layering::{ScaledMenuQuad, ScaledOutputQuad};

pub(super) fn resize_canvases(
    mut events: EventReader<WindowResized>,
    mut screen_mults: ResMut<ScreenMults>,
    mut window_dims: ResMut<WindowDims>,
    mut scaled_output_quad: Query<&mut Transform, With<ScaledOutputQuad>>,
    mut scaled_menu_quad: Query<&mut Transform, (With<ScaledMenuQuad>, Without<ScaledOutputQuad>)>,
) {
    let Some(event) = events.read().last() else {
        return;
    };
    window_dims.0.x = event.width.round() as u32;
    window_dims.0.y = event.height.round() as u32;
    let x_mults = (event.width / (SCREEN_WIDTH as f32)).floor() as u32;
    let y_mults = (event.height / (SCREEN_HEIGHT as f32)).floor() as u32;
    let mults = x_mults.min(y_mults).max(1);
    if mults == screen_mults.0 {
        return;
    }
    screen_mults.0 = mults;
    if let Ok(mut scaled_quad) = scaled_output_quad.get_single_mut() {
        scaled_quad.scale = Vec3::ONE * mults as f32;
    };
    if let Ok(mut scaled_quad) = scaled_menu_quad.get_single_mut() {
        scaled_quad.scale = Vec3::ONE * mults as f32;
    };
}
