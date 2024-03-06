use bevy::{prelude::*, render::view::RenderLayers, sprite::MaterialMesh2dBundle};

use crate::math::regular_polygon;

use super::{lightmap::light_layer, mesh::generate_new_mesh};

#[derive(Component)]
pub struct RegularLight {
    pub num_sides: u32,
    pub radius: f32,
}

#[derive(Bundle)]
pub struct RegularLightBundle {
    // pub regular_light: RegularLight,
    pub mesh: MaterialMesh2dBundle<ColorMaterial>,
    pub render_layers: RenderLayers,
}
impl RegularLightBundle {
    pub fn new(
        num_sides: u32,
        radius: f32,
        mats: &mut ResMut<Assets<ColorMaterial>>,
        meshes: &mut ResMut<Assets<Mesh>>,
    ) -> Self {
        let mat = mats.add(ColorMaterial::from(Color::Hsla {
            hue: 0.0,
            saturation: 1.0,
            lightness: 1.0,
            alpha: 1.0,
        }));
        let points = regular_polygon(num_sides, 0.0, radius);
        let mesh = generate_new_mesh(&points, &mat, meshes);
        Self {
            mesh,
            render_layers: light_layer(),
        }
    }
}
