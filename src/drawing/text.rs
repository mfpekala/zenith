use bevy::prelude::*;

use crate::meta::consts::window_to_screen_ratio;

use super::lightmap::{menu_layer, sprite_layer};

pub struct ZenithTextPlugin;

#[derive(serde::Deserialize, Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum TextWeight {
    Bold,
    Medium,
    #[default]
    Regular,
    SemiBold,
}
impl TextWeight {
    pub fn to_handle(&self, asset_server: &Res<AssetServer>) -> Handle<Font> {
        // let filename = match *self {
        //     Self::Bold => "PixelifySans-Bold.ttf",
        //     Self::Medium => "PixelifySans-Medium.ttf",
        //     Self::Regular => "PixelifySans-Regular.ttf",
        //     Self::SemiBold => "PixelifySans-SemiBold.ttf",
        // };
        let filename = match *self {
            Self::Bold => "monogram.ttf",
            Self::Medium => "monogram.ttf",
            Self::Regular => "monogram.ttf",
            Self::SemiBold => "monogram.ttf",
        };
        asset_server.load(format!("fonts/{}", filename))
    }
}

#[derive(serde::Deserialize, Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum TextAlign {
    #[default]
    Left,
    Right,
    Center,
}
impl TextAlign {
    pub fn to_justify_text(&self) -> JustifyText {
        match *self {
            Self::Left => JustifyText::Left,
            Self::Right => JustifyText::Right,
            Self::Center => JustifyText::Center,
        }
    }
}

#[derive(serde::Deserialize, Debug, Clone, Default, PartialEq, PartialOrd)]
pub struct TextBox {
    pub content: String,
    pub weight: TextWeight,
    pub align: TextAlign,
    pub size: f32,
    pub top: u32,
    pub left: u32,
    pub color: (f32, f32, f32, f32),
    /// On time, off time
    pub flash: Option<(f32, f32)>,
}
impl TextBox {
    pub fn spawn(&self, commands: &mut Commands, asset_server: &Res<AssetServer>) -> Entity {
        commands
            .spawn((
                TextBundle::from_section(
                    self.content.clone(),
                    TextStyle {
                        font: self.weight.to_handle(asset_server),
                        font_size: self.size as f32,
                        color: Color::rgba(self.color.0, self.color.1, self.color.2, self.color.3),
                        ..default()
                    },
                )
                .with_text_justify(self.align.to_justify_text())
                .with_style(Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(self.top as f32 * window_to_screen_ratio()),
                    left: Val::Px(self.left as f32 * window_to_screen_ratio()),
                    ..default()
                }),
                menu_layer(),
            ))
            .id()
    }
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
