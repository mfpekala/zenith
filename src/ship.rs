use bevy::prelude::*;

use crate::drawing::Drawable;
use crate::environment::{Field, Rock};
use crate::input::MouseState;
use crate::physics::{force_quad_gravity_helper, move_dyno_helper, move_dynos};
use crate::{input::LaunchEvent, physics::Dyno};

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

pub fn draw_launch_preview(
    dyno: &Dyno,
    base_pos: &Vec2,
    mouse_state: &Res<MouseState>,
    gz: &mut Gizmos,
    rocks: &Query<(&Rock, &Transform), Without<Dyno>>,
    fields: &Query<(&Field, &Transform), Without<Dyno>>,
) {
    let Some(launch_vel) = mouse_state.pending_launch_vel else {
        return;
    };
    let mut scratch_dyno = dyno.clone();
    scratch_dyno.vel = launch_vel;
    let mut scratch_point = base_pos.clone();
    let num_onion_skins = 10;
    let frames_between_onion_skins = 10;
    for _skin in 0..num_onion_skins {
        for _ in 0..frames_between_onion_skins {
            force_quad_gravity_helper(&mut scratch_dyno, &scratch_point, fields);
            move_dyno_helper(&mut scratch_dyno, &mut scratch_point, rocks);
        }
        gz.circle_2d(scratch_point, 5.0, Color::rgb(0.7, 0.7, 0.7));
    }
}

pub fn draw_ships(
    dyno_n_trans: Query<(&Dyno, &Transform)>,
    mut gz: Gizmos,
    mouse_state: Res<MouseState>,
    rocks: Query<(&Rock, &Transform), Without<Dyno>>,
    fields: Query<(&Field, &Transform), Without<Dyno>>,
) {
    for (dyno, tran) in dyno_n_trans.iter() {
        dyno.draw(tran.translation.truncate(), &mut gz);
        draw_launch_preview(
            dyno,
            &tran.translation.truncate(),
            &mouse_state,
            &mut gz,
            &rocks,
            &fields,
        );
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

pub fn register_ship(app: &mut App) {
    app.add_systems(Startup, setup_test_ship);
    app.add_systems(Update, launch_test_ship);
    app.add_systems(Update, draw_ships.after(move_dynos));
}
