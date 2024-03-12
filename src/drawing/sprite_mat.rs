use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, Material2dPlugin},
};

pub const SPRITE_MATERIAL_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(3263429872114192109);

#[derive(Default)]
pub struct SpriteMaterialPlugin;

impl Plugin for SpriteMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<SpriteMaterial>::default());
    }
}

#[derive(AsBindGroup, Debug, Clone, Asset, TypePath)]
pub struct SpriteMaterial {
    #[texture(1)]
    #[sampler(2)]
    pub sprite_texture: Handle<Image>,
}

impl Material2d for SpriteMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/sprite_mat.wgsl".into()
    }
}
