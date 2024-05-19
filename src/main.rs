pub mod camera;
pub mod cutscenes;
pub mod drawing;
pub mod editor;
pub mod environment;
pub mod input;
pub mod math;
pub mod menu;
pub mod meta;
pub mod physics;
pub mod ship;
pub mod sound;
pub mod uid;

use bevy::{ecs::system::SystemState, prelude::*, window::PrimaryWindow, winit::WinitWindows};
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use camera::register_camera;
use cutscenes::CutscenesPlugin;
use drawing::register_drawing;
use editor::EditorPlugin;
use environment::EnvironmentPlugin;
use input::register_input;
use menu::{menu_asset::MenuAsset, register_menus};
use meta::{
    consts::{TuneableConsts, TuneableConstsPlugin, FRAMERATE},
    game_state::register_game_state,
    MetaPlugin,
};
use physics::PhysicsPlugin;
use ship::register_ship;
use sound::SoundPlugin;
use uid::UIdPlugin;

pub fn main_setup() {}

fn get_size(q_windows: Query<&Window, With<PrimaryWindow>>) {}

fn main() {
    env_logger::init();
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    resizable: true,
                    title: "Zenith".to_string(),
                    // mode: bevy::window::WindowMode::BorderlessFullscreen,
                    ..default()
                }),
                ..default()
            })
            .set(ImagePlugin::default_nearest()),
    )
    .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
    .add_systems(Startup, main_setup);
    app.insert_resource(Time::<Fixed>::from_hz(FRAMERATE));
    app.add_plugins(RonAssetPlugin::<MenuAsset>::new(&["menu.ron"]));
    app.add_plugins(UIdPlugin);
    app.add_plugins(TuneableConstsPlugin);
    app.add_plugins(RonAssetPlugin::<TuneableConsts>::new(&["consts.ron"]));
    app.add_plugins(CutscenesPlugin);
    app.add_plugins(WorldInspectorPlugin::new());
    app.add_plugins(SoundPlugin);
    app.add_plugins(EditorPlugin);
    app.add_plugins(PhysicsPlugin);
    app.add_plugins(MetaPlugin);
    app.add_plugins(EnvironmentPlugin);
    app.register_type::<Vec2>();
    app.register_type::<IVec2>();
    app.register_type::<IVec3>();
    app.register_type::<Option<Vec2>>();
    app.register_type::<Rect>();
    app.register_type::<Option<Rect>>();
    app.register_type::<Name>();
    // First register the game state
    register_game_state(&mut app);
    // Then we can register everything else
    register_camera(&mut app);
    register_drawing(&mut app);
    register_input(&mut app);
    register_menus(&mut app);
    register_ship(&mut app);
    app.run();
}
