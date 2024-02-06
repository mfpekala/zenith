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
    pub bounciness: f32,
    pub friction: f32,
}
impl Rock {
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
        bounciness: f32,
        friction: f32,
    ) -> Self {
        let mut points: Vec<Vec2> = vec![];
        for _ in 0..num_sides {
            let x = angle.to_radians().cos() * radius;
            let y = angle.to_radians().sin() * radius;
            points.push(Vec2 { x, y });
            angle -= 360.0 / (num_sides as f32);
        }
        Self {
            points,
            bounciness,
            friction,
        }
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
pub struct ForceQuad {
    pub points: [Vec2; 4],
    pub strength: f32,
    pub dir: Vec2,
    pub drag: f32,
}
impl Drawable for ForceQuad {
    fn draw(&self, base_pos: Vec2, gz: &mut Gizmos) {
        draw_polygon(base_pos, &self.points, Color::YELLOW, gz);
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
            let diff = p2 - p1;
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
impl Drawable for Field {
    fn draw(&self, base_pos: Vec2, gz: &mut Gizmos) {
        for fq in self.force_quads.iter() {
            fq.draw(base_pos, gz);
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
        Vec2::new(40.0, 0.0),
        Rock::regular_polygon(6, 100.0, 20.0, 0.6, 0.3),
        Some((200.0, 0.0002)),
    );
    commands.spawn(bundle);
}

pub fn register_environment(app: &mut App) {
    app.add_systems(Startup, test_comets);
    app.add_systems(Update, (draw_rocks, draw_fields));
}
