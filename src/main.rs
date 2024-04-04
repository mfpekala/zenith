pub mod camera;
pub mod cutscenes;
pub mod drawing;
pub mod editor;
pub mod environment;
pub mod input;
pub mod leveler;
pub mod math;
pub mod menu;
pub mod meta;
// pub mod old_editor;
pub mod physics;
pub mod ship;
pub mod sound;
pub mod uid;

use bevy::{prelude::*, window::WindowResolution};
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use camera::register_camera;
use cutscenes::CutscenesPlugin;
use drawing::register_drawing;
use editor::EditorPlugin;
use environment::register_environment;
use input::register_input;
use leveler::register_leveler;
use menu::{menu_asset::MenuAsset, register_menus};
use meta::{
    consts::{TuneableConsts, TuneableConstsPlugin, WINDOW_HEIGHT, WINDOW_WIDTH},
    game_state::register_game_state,
};
use physics::register_physics;
use ship::register_ship;
use sound::SoundPlugin;
use uid::UIdPlugin;

pub fn main_setup() {}

fn main() {
    env_logger::init();
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: WindowResolution::new(WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32),
                    resizable: false,
                    title: "Zenith".to_string(),
                    ..default()
                }),
                ..default()
            })
            .set(ImagePlugin::default_nearest()),
    )
    .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
    .add_systems(Startup, main_setup);
    app.insert_resource(Time::<Fixed>::from_hz(24.0));
    app.add_plugins(RonAssetPlugin::<MenuAsset>::new(&["menu.ron"]));
    app.add_plugins(UIdPlugin);
    app.add_plugins(TuneableConstsPlugin);
    app.add_plugins(RonAssetPlugin::<TuneableConsts>::new(&["consts.ron"]));
    app.add_plugins(CutscenesPlugin);
    app.add_plugins(WorldInspectorPlugin::new());
    app.add_plugins(SoundPlugin);
    app.add_plugins(EditorPlugin);
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
    register_environment(&mut app);
    register_input(&mut app);
    register_leveler(&mut app);
    register_menus(&mut app);
    register_physics(&mut app);
    register_ship(&mut app);
    app.run();
}
