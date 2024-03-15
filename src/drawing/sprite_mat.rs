use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, Material2dPlugin},
};
use rand::{thread_rng, Rng};

use crate::math::lerp;

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
    #[uniform(3)]
    pub x: f32,
    #[uniform(4)]
    pub y: f32,
    #[uniform(5)]
    pub w: f32,
    #[uniform(6)]
    pub h: f32,
}
impl SpriteMaterial {
    /// Creates a sprite material which will render only a subrectangle of size `size` at
    /// pos `pos` from the sprite
    pub fn from_handle(handle: Handle<Image>, pos: Option<Vec2>, size: Option<Vec2>) -> Self {
        let pos = pos.unwrap_or(Vec2::ZERO);
        let size = size.unwrap_or(Vec2::ONE);
        Self {
            sprite_texture: handle,
            x: pos.x,
            y: pos.y,
            w: size.x,
            h: size.y,
        }
    }

    /// Given a mesh that is bounded by a rectangle of size `mesh_size`, return a random
    /// `pos` and `size` to take a slice of a texture of size `image_size` that will not
    /// stretch pixels
    pub fn random_sized_bounds(mesh_size: UVec2, image_size: UVec2) -> (Vec2, Vec2) {
        let mut rng = thread_rng();
        let mut helper = |m, i| {
            if m >= i {
                (0.0, 1.0)
            } else {
                let frac_w = (m as f32) / (i as f32);
                let x = lerp(rng.gen(), 0.0, 1.0 - frac_w);
                (x, frac_w)
            }
        };
        let (pos_x, size_x) = helper(mesh_size.x, image_size.x);
        let (pos_y, size_y) = helper(mesh_size.y, image_size.y);
        (Vec2::new(pos_x, pos_y), Vec2::new(size_x, size_y))
    }
}

impl Material2d for SpriteMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/sprite_mat.wgsl".into()
    }
}
