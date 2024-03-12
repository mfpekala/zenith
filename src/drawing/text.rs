use bevy::prelude::*;

pub struct ZenithTextPlugin;

#[derive(serde::Deserialize, Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum TextWeight {
    Bold,
    Medium,
    #[default]
    Regular,
    SemiBold,
}

#[derive(serde::Deserialize, Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum TextAlign {
    #[default]
    Left,
    Right,
    Center,
}

#[derive(serde::Deserialize, Debug, Component, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct TextBox {
    pub content: String,
    pub weight: TextWeight,
    pub align: TextAlign,
    pub top: u32,
    pub left: u32,
    pub width: u32,
    pub height: u32,
}

fn setup_zenith_text(asset_server: Res<AssetServer>) {
    let _ = asset_server.load::<Font>("fonts/PixelifySans-Bold.ttf");
    let _ = asset_server.load::<Font>("fonts/PixelifySans-Medium.ttf");
    let _ = asset_server.load::<Font>("fonts/PixelifySans-Regular.ttf");
    let _ = asset_server.load::<Font>("fonts/PixelifySans-SemiBold.ttf");
}

impl Plugin for ZenithTextPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_zenith_text);
    }
}
