use bevy::prelude::*;

use crate::{
    drawable,
    drawing::{draw_polygon, Drawable},
    math::{get_shell, MathLine},
};

/// NOTE: Points MUST be in clockwise order
#[derive(Component, Clone, Debug)]
pub struct Rock {
    pub points: Vec<Vec2>,
}
impl Rock {
    pub fn closest_point(&self, point: &Vec2) -> Vec2 {
        let lines = MathLine::from_points(&self.points);
        let mut min_dist = f32::MAX;
        let mut min_point = Vec2 {
            x: f32::MAX,
            y: f32::MAX,
        };
        for line in lines {
            let close_point = line.closest_point(point);
            let dist = point.distance(close_point);
            if dist < min_dist {
                min_dist = dist;
                min_point = close_point;
            }
        }
        min_point
    }
}
impl Drawable for Rock {
    fn draw(&self, base_pos: Vec2, gz: &mut Gizmos) {
        draw_polygon(base_pos, &self.points, Color::WHITE, gz);
    }
}
drawable!(Rock, draw_rocks);

/// NOTE: Points MUST be in clockwise order
#[derive(Component, Clone, Debug)]
pub struct ForceRegion {
    pub points: [Vec2; 4],
    pub strength: f32,
}
impl Drawable for ForceRegion {
    fn draw(&self, base_pos: Vec2, gz: &mut Gizmos) {
        draw_polygon(base_pos, &self.points, Color::YELLOW, gz);
    }
}

#[derive(Component, Clone, Debug)]
pub struct Field {
    pub regions: Vec<ForceRegion>,
}
impl Field {
    pub fn empty() -> Self {
        Self { regions: vec![] }
    }

    pub fn uniform_around_rock(rock: &Rock, reach: f32, strength: f32) -> Self {
        let shell = get_shell(&rock.points, reach);
        let mut regions: Vec<ForceRegion> = vec![];
        for ix in 0..rock.points.len() {
            let region = ForceRegion {
                points: [
                    shell[ix],
                    shell[(ix + 1) % shell.len()],
                    rock.points[(ix + 1) % rock.points.len()],
                    rock.points[ix],
                ],
                strength,
            };
            regions.push(region);
        }
        Self { regions }
    }
}
impl Drawable for Field {
    fn draw(&self, base_pos: Vec2, gz: &mut Gizmos) {
        for region in self.regions.iter() {
            region.draw(base_pos, gz);
        }
    }
}
drawable!(Field, draw_fields);

#[derive(Bundle)]
pub struct PlanetBundle {
    rock: Rock,
    field: Field,
    spatial: SpatialBundle,
}
impl PlanetBundle {
    pub fn new(base_pos: Vec2, rock: Rock, reach_n_strength: Option<(f32, f32)>) -> Self {
        let spatial = SpatialBundle {
            transform: Transform {
                translation: base_pos.extend(0.0),
                ..default()
            },
            ..default()
        };
        match reach_n_strength {
            Some((reach, strength)) => Self {
                field: Field::uniform_around_rock(&rock, reach, strength),
                rock,
                spatial,
            },
            None => Self {
                rock,
                field: Field::empty(),
                spatial,
            },
        }
    }
}

fn test_comets(mut commands: Commands) {
    let bundle = PlanetBundle::new(
        Vec2::new(0.0, 0.0),
        Rock {
            points: vec![
                Vec2::new(-100.0, 100.0),
                Vec2::new(100.0, 100.0),
                Vec2::new(100.0, -100.0),
                Vec2::new(-100.0, -100.0),
            ],
        },
        Some((50.0, 50.0)),
    );
    commands.spawn(bundle);
}

pub fn register_environment(app: &mut App) {
    app.add_systems(Startup, test_comets);
    app.add_systems(Update, (draw_rocks, draw_fields));
}
