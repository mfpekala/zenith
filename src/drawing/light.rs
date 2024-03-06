use bevy::{prelude::*, render::view::RenderLayers, sprite::MaterialMesh2dBundle};

use crate::math::regular_polygon;

use super::{lightmap::light_layer, mesh::generate_new_mesh, LightGizmoGroup};

#[derive(Component)]
pub struct RegularLight {
    pub radius: f32,
}

#[derive(Bundle)]
pub struct RegularLightBundle {
    pub regular_light: RegularLight,
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
            regular_light: RegularLight { radius },
            mesh,
            render_layers: light_layer(),
        }
    }
}

fn draw_regular_rings(
    regular: Query<(&GlobalTransform, &RegularLight)>,
    mut gz: Gizmos<LightGizmoGroup>,
) {
    for (tran, light) in regular.iter() {
        for add_rad in 0..light.radius as u32 {
            let frac = (add_rad as f32) / light.radius;
            gz.circle_2d(
                tran.translation().truncate(),
                light.radius - 1.0 + add_rad as f32,
                Color::Hsla {
                    hue: 1.0,
                    saturation: 1.0,
                    lightness: 1.0,
                    // alpha: ((frac * 3.14159265 / 2.0).cos()).powi(3),
                    alpha: (1.0 - frac).powi(3),
                },
            );
        }
    }
}

pub fn register_light(app: &mut App) {
    app.add_systems(Update, draw_regular_rings);
}
