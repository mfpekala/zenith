pub mod drawing;
pub mod environment;
pub mod math;
pub mod meta;
pub mod physics;

use bevy::{prelude::*, window::WindowResolution};
use environment::register_environment;
use meta::consts::{WINDOW_HEIGHT, WINDOW_WIDTH};
use physics::register_physics;

pub fn main_setup(
    mut commands: Commands,
    // mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<ColorMaterial>>,
    // asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2dBundle::default());
}

pub fn scratch_fn(mut gz: Gizmos) {}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            // present_mode: (),
            resolution: WindowResolution::new(WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32),
            title: "PUPIL".to_string(),
            ..default()
        }),
        ..default()
    }))
    .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
    .add_systems(Startup, main_setup);
    app.add_systems(Update, scratch_fn);
    register_environment(&mut app);
    register_physics(&mut app);
    app.run();
}
