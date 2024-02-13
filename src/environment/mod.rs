use crate::{
    drawing::{
        hollow::{draw_hollow_polygon, HollowDrawable},
        mesh::generate_new_mesh,
    },
    hollow_drawable,
    math::{get_shell, MathLine},
    meta::game_state::{in_editor, in_level},
};
use bevy::{prelude::*, sprite::MaterialMesh2dBundle, utils::HashMap};

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
}

/// NOTE: Points MUST be in clockwise order
#[derive(Component, Clone, Debug)]
pub struct ForceQuad {
    pub points: [Vec2; 4],
    pub strength: f32,
    pub dir: Vec2,
    pub drag: f32,
}
impl HollowDrawable for ForceQuad {
    fn draw_hollow(&self, base_pos: Vec2, gz: &mut Gizmos) {
        draw_hollow_polygon(base_pos, &self.points, Color::YELLOW, gz);
    }
}
impl ForceQuad {
    pub fn effective_mult(&self, pos: &Vec2, base_pos: &Vec2, radius: f32) -> f32 {
        let lines = MathLine::from_points(&self.points);
        let mut max_dist = f32::MIN;
        let adjusted_pos = *pos - *base_pos;
        for line in lines {
            let signed_dist = line.signed_distance_from_point(&adjusted_pos);
            max_dist = max_dist.max(signed_dist);
        }
        max_dist = (-1.0 * (max_dist / radius) + 1.0) / 2.0;
        max_dist.min(1.0).max(0.0)
    }
}

#[derive(Component, Clone, Debug)]
pub struct Field {
    pub force_quads: Vec<ForceQuad>,
}
impl Field {
    pub fn empty() -> Self {
        Self {
            force_quads: vec![],
        }
    }

    pub fn uniform_around_rock(rock: &Rock, reach: f32, strength: f32) -> Self {
        let shell = get_shell(&rock.points, reach);
        let mut regions: Vec<ForceQuad> = vec![];
        for ix in 0..rock.points.len() {
            let p1 = shell[ix];
            let p2 = shell[(ix + 1) % shell.len()];
            let diff = (p2 - p1).normalize();
            let region = ForceQuad {
                points: [
                    p1,
                    p2,
                    rock.points[(ix + 1) % rock.points.len()],
                    rock.points[ix],
                ],
                strength,
                dir: Vec2 {
                    x: diff.y,
                    y: -diff.x,
                },
                drag: 0.0003,
            };
            regions.push(region);
        }
        Self {
            force_quads: regions,
        }
    }
}
impl HollowDrawable for Field {
    fn draw_hollow(&self, base_pos: Vec2, gz: &mut Gizmos) {
        for fq in self.force_quads.iter() {
            fq.draw_hollow(base_pos, gz);
        }
    }
}
hollow_drawable!(Field, draw_fields);

#[derive(Component)]
pub struct Planet;

#[derive(Bundle)]
pub struct PlanetBundle {
    planet: Planet,
    rock: RockBundle,
    field: Field,
}
impl PlanetBundle {
    pub fn new(
        base_pos: Vec2,
        rock: Rock,
        reach_n_strength: Option<(f32, f32)>,
        meshes: &mut ResMut<Assets<Mesh>>,
    ) -> Self {
        let mut rock_bundle = RockBundle::from_rock(rock.clone(), meshes);
        rock_bundle.mesh.transform.translation = base_pos.extend(0.0);
        match reach_n_strength {
            Some((reach, strength)) => Self {
                planet: Planet,
                field: Field::uniform_around_rock(&rock, reach, strength),
                rock: rock_bundle,
            },
            None => Self {
                planet: Planet,
                rock: rock_bundle,
                field: Field::empty(),
            },
        }
    }
}

#[derive(Clone, Eq, Hash, PartialEq)]
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

pub fn register_environment(app: &mut App) {
    app.insert_resource(RockResources::blank());
    app.add_systems(Startup, init_rock_materials);
    app.add_systems(Update, draw_fields.run_if(in_editor.or_else(in_level)));
}
