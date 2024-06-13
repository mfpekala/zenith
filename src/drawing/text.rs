use crate::menu::placement::{GameRelativePlacement, GameRelativePlacementBundle};

use super::layering::menu_layer;
use bevy::{prelude::*, render::view::RenderLayers, sprite::Anchor};
use std::time::Duration;

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
    pub fn to_anchor(&self) -> Anchor {
        match *self {
            Self::Left => Anchor::CenterLeft,
            Self::Right => Anchor::CenterRight,
            Self::Center => Anchor::Center,
        }
    }

    pub fn to_justify(&self) -> JustifyText {
        match *self {
            Self::Left => JustifyText::Left,
            Self::Center => JustifyText::Center,
            Self::Right => JustifyText::Right,
        }
    }
}

#[derive(Bundle)]
pub struct TextBoxBundle {
    pub inner: Text2dBundle,
    render_layers: RenderLayers,
}
impl TextBoxBundle {
    pub fn new_menu_text(
        content: &str,
        size: f32,
        placement: GameRelativePlacement,
        color: Color,
        weight: TextWeight,
        align: TextAlign,
        asset_server: &Res<AssetServer>,
    ) -> (Self, GameRelativePlacement) {
        (
            Self {
                inner: Text2dBundle {
                    text: Text::from_section(
                        content.to_string(),
                        TextStyle {
                            font: weight.to_handle(asset_server),
                            font_size: size,
                            color,
                            ..default()
                        },
                    )
                    .with_justify(align.to_justify()),
                    text_anchor: align.to_anchor(),
                    transform: Transform::from_translation(placement.pos.as_vec3()),
                    ..default()
                },
                render_layers: menu_layer(),
            },
            placement,
        )
    }
}

#[derive(Component)]
pub struct Flashing {
    pub times: (f32, f32),
    pub timer: Timer,
    pub is_on: bool,
}
impl Flashing {
    pub fn new(time_on: f32, time_off: f32) -> Self {
        Self {
            times: (time_on, time_off),
            timer: Timer::new(Duration::from_secs_f32(time_on), TimerMode::Once),
            is_on: true,
        }
    }
}

fn update_flashing_text(mut texts: Query<(&mut Text, &mut Flashing)>, time: Res<Time>) {
    for (mut text, mut flash) in texts.iter_mut() {
        flash.timer.tick(time.delta());
        if flash.timer.finished() {
            if flash.is_on {
                flash.is_on = false;
                flash.timer = Timer::new(Duration::from_secs_f32(flash.times.1), TimerMode::Once);
                for section in text.sections.iter_mut() {
                    section.style.color.set_a(0.0);
                }
            } else {
                flash.is_on = true;
                flash.timer = Timer::new(Duration::from_secs_f32(flash.times.0), TimerMode::Once);
                for section in text.sections.iter_mut() {
                    section.style.color.set_a(1.0);
                }
            }
        }
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
        app.add_systems(Update, update_flashing_text);
    }
}
