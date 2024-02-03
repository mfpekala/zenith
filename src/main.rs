pub mod camera;
pub mod drawing;
pub mod environment;
pub mod input;
pub mod math;
pub mod meta;
pub mod physics;

use bevy::{prelude::*, window::WindowResolution};
use camera::register_camera;
use environment::register_environment;
use input::register_input;
use meta::consts::{WINDOW_HEIGHT, WINDOW_WIDTH};
use physics::register_physics;

pub fn main_setup() {}

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
    register_camera(&mut app);
    register_environment(&mut app);
    register_input(&mut app);
    register_physics(&mut app);
    app.run();
}
