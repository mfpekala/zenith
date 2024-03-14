use bevy::prelude::*;
use std::collections::HashMap;

/// Size of the window's width
pub const WINDOW_WIDTH: usize = 800;
/// Size of the window's height
pub const WINDOW_HEIGHT: usize = 800;

/// Number of pixels to show in screen width (should divide WINDOW_WIDTH)
pub const SCREEN_WIDTH: usize = 160;
/// Number of pixels to show in screen height (should divide WINDOW_HEIGHT)
pub const SCREEN_HEIGHT: usize = 160;

pub fn window_to_screen_ratio() -> f32 {
    (WINDOW_WIDTH as f32) / (SCREEN_WIDTH as f32)
}

pub fn fscreen_size() -> Vec2 {
    Vec2 {
        x: SCREEN_WIDTH as f32,
        y: SCREEN_HEIGHT as f32,
    }
}

/// Kinda cursed if it's not this
pub const PIXEL_SIZE: usize = 1;

/// How many collisions can be resolved during a single frame? Caps memory usage of collision mechanism.
pub const MAX_COLLISIONS_PER_FRAME: usize = 16;

#[derive(
    serde::Deserialize,
    bevy::asset::Asset,
    bevy::reflect::TypePath,
    Debug,
    PartialEq,
    Clone,
    Resource,
    Default,
)]
pub struct TuneableConsts {
    pub map: HashMap<String, f32>,
}
impl TuneableConsts {
    pub fn get_or(&self, key: &str, backup: f32) -> f32 {
        match self.map.get(key) {
            Some(x) => x.clone(),
            None => {
                warn!("Seem to missing {} from tuneable constants", key);
                backup
            }
        }
    }
}

#[derive(Resource)]
pub struct TuneableConstsHandle(pub Handle<TuneableConsts>);

fn setup_tuneable_consts(mut commands: Commands, asset_server: Res<AssetServer>) {
    let handle = asset_server.load::<TuneableConsts>("tuneable.consts.ron");
    commands.insert_resource(TuneableConstsHandle(handle));
    commands.insert_resource(TuneableConsts::default());
}

fn update_tuneable_consts(
    handle: Res<TuneableConstsHandle>,
    mut consts: ResMut<TuneableConsts>,
    asset: Res<Assets<TuneableConsts>>,
) {
    if let Some(data) = asset.get(handle.0.id()) {
        if *data != *consts {
            *consts = data.clone();
        }
        // *consts = data.clone();
    }
}

pub struct TuneableConstsPlugin;

impl Plugin for TuneableConstsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_tuneable_consts);
        app.add_systems(Update, update_tuneable_consts);
    }
}
