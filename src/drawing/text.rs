use crate::menu::placement::GameRelativePlacement;

use super::layering::{menu_layer, sprite_layer};
use bevy::{prelude::*, render::view::RenderLayers, sprite::Anchor, utils::HashMap};
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct ZenithTextPlugin;

#[derive(Component, Default, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize)]
pub enum TextWeight {
    Bold,
    Medium,
    #[default]
    Regular,
    SemiBold,
}
impl TextWeight {
    pub fn to_handle_ass(&self, asset_server: &Res<AssetServer>) -> Handle<Font> {
        let filename = match *self {
            Self::Bold => "monogram.ttf",
            Self::Medium => "monogram.ttf",
            Self::Regular => "monogram.ttf",
            Self::SemiBold => "monogram.ttf",
        };
        asset_server.load(format!("fonts/{}", filename))
    }
}

#[derive(Default, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize)]
pub enum TextAlign {
    Left,
    Right,
    #[default]
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
    font: TextWeight,
}
impl TextBoxBundle {
    pub fn new_menu_text(
        content: &str,
        size: f32,
        placement: GameRelativePlacement,
        color: Color,
        weight: TextWeight,
        align: TextAlign,
    ) -> (Self, GameRelativePlacement) {
        (
            Self {
                inner: Text2dBundle {
                    text: Text::from_section(
                        content.to_string(),
                        TextStyle {
                            font_size: size,
                            color,
                            ..default()
                        },
                    )
                    .with_justify(align.to_justify()),
                    text_anchor: align.to_anchor(),
                    transform: Transform::from_translation(placement.pos.as_vec3()),
                    visibility: Visibility::Hidden,
                    ..default()
                },
                render_layers: menu_layer(),
                font: weight,
            },
            placement,
        )
    }

    pub fn new_sprite_text(
        content: &str,
        size: f32,
        pos: IVec3,
        color: Color,
        weight: TextWeight,
        align: TextAlign,
    ) -> Self {
        Self {
            inner: Text2dBundle {
                text: Text::from_section(
                    content.to_string(),
                    TextStyle {
                        font_size: size,
                        color,
                        ..default()
                    },
                )
                .with_justify(align.to_justify()),
                text_anchor: align.to_anchor(),
                transform: Transform::from_translation(pos.as_vec3()),
                visibility: Visibility::Hidden,
                ..default()
            },
            render_layers: sprite_layer(),
            font: weight,
        }
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

/// TODO: Support render layers
#[derive(Default, Clone, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize)]
pub struct TextNode {
    pub content: String,
    pub weight: TextWeight,
    pub align: TextAlign,
    pub color: Color,
    pub size: f32,
    pub pos: IVec3,
}

/// Gotta love hierarchies in Bevy! Maybe I'm stupid
#[derive(Component, Default, Clone, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct TextManager {
    map: HashMap<String, TextNode>,
}
impl TextManager {
    pub fn from_pairs(pairs: Vec<(&str, TextNode)>) -> Self {
        let mut map = HashMap::new();
        for (key, val) in pairs.into_iter() {
            map.insert(key.to_string(), val);
        }
        Self { map }
    }
}

fn setup_zenith_text(asset_server: Res<AssetServer>) {
    let _ = asset_server.load::<Font>("fonts/PixelifySans-Bold.ttf");
    let _ = asset_server.load::<Font>("fonts/PixelifySans-Medium.ttf");
    let _ = asset_server.load::<Font>("fonts/PixelifySans-Regular.ttf");
    let _ = asset_server.load::<Font>("fonts/PixelifySans-SemiBold.ttf");
}

/// For correcting font weights. Maybe not the best solution but is ergonomic and
/// allows you to spawn text without passing around an AssetServer everywhere.
fn correct_fonts(
    mut texts: Query<(Entity, &mut Visibility, &TextWeight, &mut Text)>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    for (eid, mut viz, weight, mut text) in texts.iter_mut() {
        for section in text.sections.iter_mut() {
            section.style.font = weight.to_handle_ass(&asset_server);
            *viz = Visibility::Inherited;
            commands.entity(eid).remove::<TextWeight>();
        }
    }
}

/// Watch for changed managers and just redo all their kids
fn update_text_managers(
    managers: Query<(Entity, &TextManager), Changed<TextManager>>,
    mut commands: Commands,
) {
    for (eid, manager) in managers.iter() {
        commands.entity(eid).despawn_descendants();
        commands.entity(eid).with_children(|parent| {
            for (key, node) in manager.map.iter() {
                let text_bund = TextBoxBundle::new_sprite_text(
                    &node.content,
                    node.size,
                    node.pos,
                    node.color,
                    node.weight,
                    node.align,
                );
                parent.spawn((text_bund, Name::new(key.clone())));
            }
        });
    }
}

impl Plugin for ZenithTextPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<TextManager>();
        app.register_type::<TextNode>();

        app.add_systems(Startup, setup_zenith_text);
        app.add_systems(Update, update_flashing_text);
        app.add_systems(Update, correct_fonts);
        app.add_systems(Update, update_text_managers);
    }
}
