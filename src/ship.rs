use bevy::prelude::*;

use crate::drawing::Drawable;
use crate::environment::{Field, Rock};
use crate::input::MouseState;
use crate::meta::game_state::{entered_editor, in_editor, in_level};
use crate::physics::{force_quad_gravity_helper, move_dyno_helper, move_dynos};
use crate::{input::LaunchEvent, physics::Dyno};

#[derive(Bundle)]
pub struct ShipBundle {
    dyno: Dyno,
    spatial: SpatialBundle,
    launch_preview: LaunchPreview,
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
            launch_preview: LaunchPreview::new(),
        }
    }
}

#[derive(Component)]
pub struct LaunchPreview {
    pub tick: u32,
    pub speed: u32,
    pub num_skins: u32,
    pub ticks_between_skins: u32,
}
impl LaunchPreview {
    pub fn new() -> Self {
        Self {
            tick: 0,
            speed: 3,
            num_skins: 12,
            ticks_between_skins: 12,
        }
    }
}

fn draw_ships(dyno_n_trans: Query<(&Dyno, &Transform)>, mut gz: Gizmos) {
    for (dyno, tran) in dyno_n_trans.iter() {
        dyno.draw(tran.translation.truncate(), &mut gz);
    }
}

fn draw_launch_previews(
    mut prev_n_trans: Query<(&mut LaunchPreview, &Dyno, &Transform)>,
    mouse_state: Res<MouseState>,
    mut gz: Gizmos,
    rocks: Query<(&Rock, &Transform), Without<Dyno>>,
    fields: Query<(&Field, &Transform), Without<Dyno>>,
) {
    let Some(launch_vel) = mouse_state.pending_launch_vel else {
        return;
    };
    for (mut prev, dyno, tran) in prev_n_trans.iter_mut() {
        let mut scratch_dyno = dyno.clone();
        scratch_dyno.vel = launch_vel;
        let mut scratch_point = tran.translation.truncate();
        // Offset
        let prev_applied = prev.tick / prev.speed;
        for _tick in 0..prev_applied {
            force_quad_gravity_helper(&mut scratch_dyno, &scratch_point, &fields);
            move_dyno_helper(&mut scratch_dyno, &mut scratch_point, &rocks);
        }
        prev.tick = (prev.tick + 1) % (prev.ticks_between_skins * prev.speed);
        // Draw the damn things
        for skin in 0..prev.num_skins {
            let alpha = 1.0
                - (prev_applied as f32 + skin as f32 * prev.ticks_between_skins as f32)
                    / (prev.num_skins as f32 * prev.ticks_between_skins as f32);
            gz.circle_2d(scratch_point, 5.0, Color::rgba(0.7, 0.7, 0.7, alpha));
            for _ in 0..prev.ticks_between_skins {
                force_quad_gravity_helper(&mut scratch_dyno, &scratch_point, &fields);
                move_dyno_helper(&mut scratch_dyno, &mut scratch_point, &rocks);
            }
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

pub fn register_ship(app: &mut App) {
    app.add_systems(Update, setup_test_ship.run_if(entered_editor));
    app.add_systems(Update, launch_test_ship.run_if(in_editor.or_else(in_level)));
    app.add_systems(
        Update,
        (draw_ships, draw_launch_previews)
            .after(move_dynos)
            .run_if(in_editor.or_else(in_level)),
    );
}
