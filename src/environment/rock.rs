use crate::{
    drawing::{lightmap::sprite_layer, mesh::generate_new_mesh},
    math::{regular_polygon, MathLine},
};
use bevy::{prelude::*, render::view::RenderLayers, sprite::MaterialMesh2dBundle, utils::HashMap};

#[derive(Debug, Clone, Eq, Hash, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum RockKind {
    Normal,
    SimpleKill,
}

#[derive(Clone)]
pub struct RockFeatures {
    pub bounciness: f32,
    pub friction: f32,
    mat: Handle<ColorMaterial>,
}

#[derive(Resource)]
pub struct RockResources {
    pub feature_map: HashMap<RockKind, RockFeatures>,
}
impl RockResources {
    pub fn blank() -> Self {
        Self {
            feature_map: HashMap::new(),
        }
    }

    pub fn get_type(&self, rock_type: RockKind) -> RockFeatures {
        self.feature_map[&rock_type].clone()
    }
}

fn init_rock_materials(
    mut rock_resources: ResMut<RockResources>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Normal rock
    let normal_features = RockFeatures {
        bounciness: 0.6,
        friction: 0.3,
        mat: materials.add(ColorMaterial::from(Color::Hsla {
            hue: 87.0,
            saturation: 0.28,
            lightness: 0.8,
            alpha: 1.0,
        })),
    };
    rock_resources
        .feature_map
        .insert(RockKind::Normal, normal_features);
    // Simple kill rock
    let simple_kill_features = RockFeatures {
        bounciness: 0.0,
        friction: 1.0,
        mat: materials.add(ColorMaterial::from(Color::RED)),
    };
    rock_resources
        .feature_map
        .insert(RockKind::SimpleKill, simple_kill_features);
}

/// NOTE: Points MUST be in clockwise order
#[derive(Component, Clone)]
pub struct Rock {
    pub points: Vec<Vec2>,
    pub kind: RockKind,
    pub features: RockFeatures,
}
impl Rock {
    pub fn new(points: Vec<Vec2>, kind: RockKind, features: RockFeatures) -> Self {
        Rock {
            points,
            kind,
            features,
        }
    }

    pub fn closest_point(&self, point: &Vec2, base_point: &Vec2) -> Vec2 {
        let lines = MathLine::from_points(&self.points);
        let mut min_dist = f32::MAX;
        let mut min_point = Vec2 {
            x: f32::MAX,
            y: f32::MAX,
        };
        let adjusted_point = *point - *base_point;
        for line in lines {
            let close_point = line.closest_point_on_segment(&adjusted_point);
            let dist = adjusted_point.distance(close_point);
            if dist < min_dist {
                min_dist = dist;
                min_point = close_point;
            }
        }
        min_point + *base_point
    }

    pub fn from_regular_polygon(
        num_sides: u32,
        radius: f32,
        angle: f32,
        features: RockFeatures,
    ) -> Self {
        let points = regular_polygon(num_sides, angle, radius);
        Self {
            points,
            kind: RockKind::Normal,
            features,
        }
    }
}

#[derive(Bundle)]
pub struct RockBundle {
    pub rock: Rock,
    pub mesh: MaterialMesh2dBundle<ColorMaterial>,
    pub render_layers: RenderLayers,
}
impl RockBundle {
    pub fn from_rock(rock: Rock, meshes: &mut ResMut<Assets<Mesh>>) -> Self {
        let mesh = generate_new_mesh(&rock.points, &rock.features.mat, meshes);
        Self {
            rock,
            mesh,
            render_layers: sprite_layer(),
        }
    }

    pub fn spawn(
        commands: &mut Commands,
        base_pos: Vec2,
        rock: Rock,
        meshes: &mut ResMut<Assets<Mesh>>,
    ) {
        let mut bundle = Self::from_rock(rock, meshes);
        bundle.mesh.transform.translation = base_pos.extend(0.0);
        commands.spawn(bundle);
    }
}

pub fn register_rocks(app: &mut App) {
    app.insert_resource(RockResources::blank());
    app.add_systems(Startup, init_rock_materials);
}
