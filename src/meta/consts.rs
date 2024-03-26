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

pub struct TuneableConstsPlugin;

impl Plugin for TuneableConstsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_tuneable_consts);
        app.add_systems(Update, update_tuneable_consts);
    }
}

#[macro_export]
macro_rules! add_hot_resource {
    ($res_struct: ident, $ron_path: expr, $setup_fname: ident, $update_fname: ident) => {
        pub(super) fn $setup_fname(mut commands: Commands, asset_server: Res<AssetServer>) {
            let handle = asset_server.load::<$res_struct>($ron_path);
            // NOTE: This (kind of) dangling handle just ensures the constants never get unloaded
            commands.insert_resource($res_struct::default());
            commands.spawn(handle);
        }

        pub(super) fn $update_fname(
            mut consts: ResMut<$res_struct>,
            asset: Res<Assets<$res_struct>>,
        ) {
            let Some(id) = asset.ids().next() else {
                return;
            };
            if let Some(data) = asset.get(id) {
                if *data != *consts {
                    *consts = data.clone();
                }
            }
        }
    };
}

add_hot_resource!(
    TuneableConsts,
    "tuneable.consts.ron",
    setup_tuneable_consts,
    update_tuneable_consts
);
