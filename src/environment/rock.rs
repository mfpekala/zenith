use crate::{drawing::mesh::generate_new_mesh, math::MathLine};
use bevy::{prelude::*, sprite::MaterialMesh2dBundle, utils::HashMap};

#[derive(Clone, Eq, Hash, PartialEq, serde::Serialize)]
pub enum RockType {
    Normal,
}

#[derive(Clone)]
pub struct RockFeatures {
    pub bounciness: f32,
    pub friction: f32,
    mat: Handle<ColorMaterial>,
}

#[derive(Resource)]
pub struct RockResources {
    pub feature_map: HashMap<RockType, RockFeatures>,
}
impl RockResources {
    pub fn blank() -> Self {
        Self {
            feature_map: HashMap::new(),
        }
    }

    pub fn get_type(&self, rock_type: RockType) -> RockFeatures {
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
        mat: materials.add(ColorMaterial::from(Color::ANTIQUE_WHITE)),
    };
    rock_resources
        .feature_map
        .insert(RockType::Normal, normal_features);
}

/// NOTE: Points MUST be in clockwise order
#[derive(Component, Clone)]
pub struct Rock {
    pub points: Vec<Vec2>,
    pub features: RockFeatures,
}
impl Rock {
    pub fn new(points: Vec<Vec2>, features: RockFeatures) -> Self {
        Rock { points, features }
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

    pub fn regular_polygon(
        num_sides: u32,
        radius: f32,
        mut angle: f32,
        features: RockFeatures,
    ) -> Self {
        let mut points: Vec<Vec2> = vec![];
        for _ in 0..num_sides {
            let x = angle.to_radians().cos() * radius;
            let y = angle.to_radians().sin() * radius;
            points.push(Vec2 { x, y });
            angle -= 360.0 / (num_sides as f32);
        }
        Self { points, features }
    }
}

#[derive(Bundle)]
pub struct RockBundle {
    pub rock: Rock,
    pub mesh: MaterialMesh2dBundle<ColorMaterial>,
}
impl RockBundle {
    pub fn from_rock(rock: Rock, meshes: &mut ResMut<Assets<Mesh>>) -> Self {
        let mesh = generate_new_mesh(&rock.points, &rock.features.mat, meshes);
        Self { rock, mesh }
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
