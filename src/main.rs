pub mod camera;
pub mod drawing;
pub mod editor;
pub mod environment;
pub mod input;
pub mod math;
pub mod menu;
pub mod meta;
pub mod physics;
pub mod ship;

use bevy::{prelude::*, window::WindowResolution};
use camera::register_camera;
use drawing::register_drawing;
use editor::register_editor;
use environment::register_environment;
use input::register_input;
use menu::register_menus;
use meta::{
    consts::{WINDOW_HEIGHT, WINDOW_WIDTH},
    game_state::register_game_state,
};
use physics::register_physics;
use ship::register_ship;

pub fn main_setup(mut gz_conf: ResMut<GizmoConfig>) {
    gz_conf.line_width = 4.0;
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            // present_mode: (),
            resolution: WindowResolution::new(WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32),
            title: "Zenith".to_string(),
            ..default()
        }),
        ..default()
    }))
    .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
    .add_systems(Startup, main_setup);
    // First register the game state
    register_game_state(&mut app);
    // Then we can register everything else
    register_camera(&mut app);
    register_drawing(&mut app);
    register_editor(&mut app);
    register_environment(&mut app);
    register_input(&mut app);
    register_menus(&mut app);
    register_physics(&mut app);
    register_ship(&mut app);
    app.run();
}
