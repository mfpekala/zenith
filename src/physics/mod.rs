use bevy::prelude::*;

use crate::{
    drawable,
    drawing::Drawable,
    environment::{Field, Rock},
    input::LaunchEvent,
};

#[derive(Component, Clone, Debug)]
pub struct Dyno {
    pub vel: Vec2,
    pub radius: f32,
}
impl Drawable for Dyno {
    fn draw(&self, base_pos: Vec2, gz: &mut Gizmos) {
        gz.circle_2d(base_pos, self.radius, Color::rgb(0.9, 0.7, 0.7));
    }
}
drawable!(Dyno, draw_dynos);

pub fn resolve_dyno_rock_collisions(
    dyno: &mut Dyno,
    point: &mut Vec2,
    rocks: &Query<(&Rock, &Transform), Without<Dyno>>,
) {
    let radius = dyno.radius;
    let mut min_dist: Option<f32> = None;
    let mut min_point: Option<Vec2> = None;
    let mut min_rock: Option<&Rock> = None;
    for (rock, trans) in rocks.iter() {
        let closest_point = rock.closest_point(&point, &trans.translation.truncate());
        let dist = point.distance(closest_point);
        if min_dist.is_none() || dist < min_dist.unwrap() {
            min_dist = Some(dist);
            min_point = Some(closest_point);
            min_rock = Some(rock);
        }
    }
    if min_point.is_none() || min_dist.unwrap_or(f32::MAX) > radius {
        // No collision
        return;
    }
    // First move the dyno out of the rock
    let min_point = min_point.unwrap();
    let diff = *point - min_point;
    *point += diff.normalize() * (radius - diff.length());
    // Then reflect the velocity
    let min_rock = min_rock.unwrap();
    let normal = diff.normalize();
    let pure_parr = -1.0 * dyno.vel.dot(normal) * normal + dyno.vel;
    let new_vel = pure_parr * (1.0 - min_rock.friction)
        - 1.0 * dyno.vel.dot(normal) * normal * min_rock.bounciness;
    dyno.vel = new_vel;
}

pub fn move_dynos(
    mut dynos: Query<(&mut Dyno, &mut Transform), Without<Rock>>,
    rocks: Query<(&Rock, &Transform), Without<Dyno>>,
) {
    for (mut dyno, mut tran) in dynos.iter_mut() {
        let mut point = tran.translation.truncate();
        let mut left_to_move = dyno.vel.length();
        while left_to_move > 0.0 {
            if dyno.vel.length() <= 0.000001 {
                // Prevent little movements
                dyno.vel = Vec2 { x: 0.0, y: 0.0 };
                break;
            }
            let moving_this_step = left_to_move.min(1.0);
            point += dyno.vel.normalize() * moving_this_step;
            resolve_dyno_rock_collisions(&mut dyno, &mut point, &rocks);
            left_to_move -= moving_this_step;
        }
        tran.translation = point.extend(0.0);
    }
}

pub fn force_quad_gravity(
    mut dynos: Query<(&mut Dyno, &Transform), Without<Field>>,
    fields: Query<(&Field, &Transform), Without<Dyno>>,
) {
    for (mut dyno, dyno_tran) in dynos.iter_mut() {
        for (field, field_tran) in fields.iter() {
            for fq in field.force_quads.iter() {
                let mult = fq.effective_mult(
                    &dyno_tran.translation.truncate(),
                    &field_tran.translation.truncate(),
                    dyno.radius,
                );
                if mult > 0.00001 {
                    dyno.vel *= 1.0 - fq.drag;
                    dyno.vel += fq.dir * fq.strength * mult;
                }
            }
        }
    }
}

// FOR TESTING PURPOSES
#[derive(Bundle)]
pub struct ShipBundle {
    dyno: Dyno,
    spatial: SpatialBundle,
}
impl ShipBundle {
    pub fn new(pos: Vec2, radius: f32) -> Self {
        Self {
            dyno: Dyno {
                vel: Vec2::ZERO,
                radius,
            },
            spatial: SpatialBundle {
                transform: Transform {
                    translation: pos.extend(0.0),
                    ..default()
                },
                ..default()
            },
        }
    }
}

fn setup_test_ship(mut commands: Commands) {
    let ship = ShipBundle::new(
        Vec2 {
            x: -205.0,
            y: 200.0,
        },
        16.0,
    );
    commands.spawn(ship);
}

fn launch_test_ship(mut dynos: Query<&mut Dyno>, mut launch_events: EventReader<LaunchEvent>) {
    for launch in launch_events.read() {
        for mut dyno in dynos.iter_mut() {
            dyno.vel = launch.vel;
        }
    }
}

pub fn register_physics(app: &mut App) {
    app.add_systems(Startup, setup_test_ship);
    app.add_systems(Update, force_quad_gravity);
    app.add_systems(Update, move_dynos.after(force_quad_gravity));
    app.add_systems(Update, draw_dynos.after(move_dynos));
    app.add_systems(Update, launch_test_ship);
}
