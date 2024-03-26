use crate::{
    add_hot_resource,
    drawing::layering::menu_layer,
    meta::game_state::{EditingMode, GameState},
};
use bevy::{prelude::*, render::view::RenderLayers};
use std::fmt;

#[derive(Component)]
pub(super) struct EditorHelp;

#[derive(Component)]
pub(super) struct EditorGrayBox;

#[derive(Component, Debug)]
pub(super) struct HelpKV {
    pub key: String,
    pub value: String,
}
impl fmt::Display for HelpKV {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.key, self.value)
    }
}

#[derive(Bundle)]
pub(super) struct HelpKVBundle {
    pub kv: HelpKV,
    pub text: Text2dBundle,
    pub render_layers: RenderLayers,
}

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
pub struct EditorHelpConfig {
    pub box_top_left: IVec2,
    pub box_bottom_right: IVec2,
    pub font_size: f32,
}

add_hot_resource!(
    EditorHelpConfig,
    "editor/hot.editor_help.ron",
    setup_editor_help_config,
    update_editor_help_config
);

pub(super) fn setup_editor_help(mut commands: Commands, help_config: Res<EditorHelpConfig>) {
    let center = (help_config.box_top_left + help_config.box_bottom_right) / 2;
    let width = help_config.box_bottom_right.x - help_config.box_top_left.x;
    let height = help_config.box_top_left.y - help_config.box_bottom_right.y;
    commands
        .spawn((
            EditorHelp,
            SpatialBundle::from_transform(Transform::from_translation(
                center.as_vec2().extend(0.0),
            )),
            menu_layer(),
        ))
        .with_children(|parent| {
            parent.spawn((
                EditorGrayBox,
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::GRAY,
                        ..default()
                    },
                    transform: Transform {
                        translation: Vec3::new(0.0, 0.0, -1.0),
                        scale: Vec3::new(width as f32, height as f32, 1.0),
                        ..default()
                    },
                    ..default()
                },
                menu_layer(),
            ));
            parent.spawn(HelpKVBundle {
                kv: HelpKV {
                    key: "Mode".to_string(),
                    value: "".to_string(),
                },
                text: Text2dBundle {
                    transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                    text_anchor: bevy::sprite::Anchor::Center,
                    ..default()
                },
                render_layers: menu_layer(),
            });
        });
}

pub(super) fn update_editor_help(
    mut kvs: Query<(&mut HelpKV, &mut Text)>,
    gs: Res<GameState>,
    help_config: Res<EditorHelpConfig>,
    mut editor_help: Query<&mut Transform, With<EditorHelp>>,
    mut gray_box: Query<&mut Transform, (With<EditorGrayBox>, Without<EditorHelp>)>,
) {
    let mode = gs.get_editing_mode();
    let center = (help_config.box_top_left + help_config.box_bottom_right) / 2;
    let width = help_config.box_bottom_right.x - help_config.box_top_left.x;
    let height = help_config.box_top_left.y - help_config.box_bottom_right.y;
    let Ok(mut editor_help) = editor_help.get_single_mut() else {
        return;
    };
    let Ok(mut gray_box) = gray_box.get_single_mut() else {
        return;
    };

    // Update the box
    editor_help.translation = center.as_vec2().extend(0.0);
    gray_box.scale = Vec3::new(width as f32, height as f32, 1.0);

    // Update the keys
    for (mut kv, mut text) in kvs.iter_mut() {
        if &kv.key == "Mode" {
            let mode_string = match mode {
                None => "None".to_string(),
                Some(thing) => match thing {
                    EditingMode::Free => "free".to_string(),
                    EditingMode::CreatingPlanet(id) => format!("create({:?})", id),
                    EditingMode::EditingPlanet(id) => format!("edit({:?})", id),
                },
            };
            kv.value = format!("{:?}", mode_string);
        }
        text.sections = vec![TextSection::new(
            format!("{}", *kv),
            TextStyle {
                font_size: help_config.font_size,
                ..default()
            },
        )]
    }
}

pub(super) fn teardown_editor_help(
    mut commands: Commands,
    editor_help: Query<Entity, With<EditorHelp>>,
) {
    for id in editor_help.iter() {
        commands.entity(id).despawn_recursive();
    }
}
