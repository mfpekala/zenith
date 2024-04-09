use crate::meta::consts::{window_to_screen_ratio, SCREEN_HEIGHT, SCREEN_WIDTH};

use super::{effects::Sizeable, layering::menu_layer};
use bevy::{prelude::*, sprite::Anchor};
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
}

#[derive(Component)]
pub struct FlashingText {
    pub times: (f32, f32),
    pub timer: Timer,
    pub is_on: bool,
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
    pub fn spawn(
        &self,
        commands: &mut Commands,
        asset_server: &Res<AssetServer>,
        z: f32,
    ) -> Entity {
        let bund = Text2dBundle {
            text: Text::from_section(
                self.content.clone(),
                TextStyle {
                    font: self.weight.to_handle(asset_server),
                    font_size: self.size as f32,
                    color: Color::rgba(self.color.0, self.color.1, self.color.2, self.color.3),
                    ..default()
                },
            ),
            text_anchor: self.align.to_anchor(),
            transform: Transform::from_translation(Vec3 {
                x: (-(SCREEN_WIDTH as f32) / 2.0 + self.left as f32) * window_to_screen_ratio(),
                y: ((SCREEN_HEIGHT as f32) / 2.0 - self.top as f32) * window_to_screen_ratio(),
                z,
            }),
            ..default()
        };
        let mut ent = commands.spawn((bund, menu_layer()));
        if let Some((on, off)) = self.flash {
            ent.insert(FlashingText {
                times: (on, off),
                timer: Timer::new(Duration::from_secs_f32(on), TimerMode::Once),
                is_on: true,
            });
        }
        ent.insert(Sizeable::new());
        ent.id()
    }
}

fn update_flashing_text(mut texts: Query<(&mut Text, &mut FlashingText)>, time: Res<Time>) {
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
