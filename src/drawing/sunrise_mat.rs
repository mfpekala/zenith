use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, Material2dPlugin},
};

pub const SUNSET_MATERIAL_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(5663469872311192109);

#[derive(Default)]
pub struct SunriseMaterialPlugin;

impl Plugin for SunriseMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<SunriseMaterial>::default());
    }
}

#[derive(AsBindGroup, Debug, Clone, Asset, TypePath)]
pub struct SunriseMaterial {
    #[uniform(1)]
    pub time_frac: f32,
}

impl Material2d for SunriseMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/sunrise_mat.wgsl".into()
    }
}
