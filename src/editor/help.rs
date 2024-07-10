use crate::{
    add_hot_resource,
    drawing::layering::menu_layer,
    meta::{
        consts::{MENU_HEIGHT, MENU_WIDTH},
        game_state::{EditingMode, EditorState, GameState, SetMetaState},
    },
};
use bevy::{ecs::system::SystemState, prelude::*, render::view::RenderLayers};
use std::fmt;

use super::{
    field::CreateStandaloneFieldEvent,
    save::{ExportLevelEvent, LoadEditorEvent, SaveEditorEvent},
    start_goal::{EGoal, EStart},
    EditingSceneRoot,
};

#[derive(Component)]
pub(super) struct EditorHelpBox;

#[derive(Component)]
pub(super) struct EditorGrayHelpBox;

#[derive(Component, Debug)]
pub(super) struct HelpBoxKV {
    pub key: String,
    pub value: String,
}
impl fmt::Display for HelpBoxKV {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.key, self.value)
    }
}

#[derive(Bundle)]
pub(super) struct HelpBoxKVBundle {
    pub kv: HelpBoxKV,
    pub text: Text2dBundle,
    pub render_layers: RenderLayers,
}

#[derive(Component, Debug)]
pub(super) struct HelpBarData {
    pub input: String,
    pub captured: bool,
    pub submitted: bool,
    pub output: Vec<String>,
}

#[derive(Component, Debug)]
pub(super) struct HelpBarInput;

#[derive(Component, Debug)]
pub(super) struct HelpBarOutput;

#[derive(Event, Debug)]
pub(super) struct HelpBarEvent(pub String);

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
    pub box_size: IVec2,
    pub box_font_size: f32,

    pub bar_size: IVec2,
    pub bar_font_size: f32,
}

add_hot_resource!(
    EditorHelpConfig,
    "editor/hot.editor_help.ron",
    setup_editor_help_config,
    update_editor_help_config
);

fn get_box_center(help_config: &Res<EditorHelpConfig>) -> IVec2 {
    let bottom_left = IVec2::new(-(MENU_WIDTH as i32 / 2), -(MENU_HEIGHT as i32 / 2));
    let box_center = bottom_left + help_config.box_size / 2;
    box_center
}

fn get_bar_center(help_config: &Res<EditorHelpConfig>) -> IVec2 {
    let bottom_right = IVec2::new(MENU_WIDTH as i32 / 2, -(MENU_HEIGHT as i32) / 2);
    let box_center = bottom_right + IVec2::new(-help_config.bar_size.x, help_config.bar_size.y) / 2;
    box_center
}

pub(super) fn setup_editor_help(mut commands: Commands, help_config: Res<EditorHelpConfig>) {
    let box_center = get_box_center(&help_config);
    commands
        .spawn((
            EditorHelpBox,
            SpatialBundle::from_transform(Transform::from_translation(
                box_center.as_vec2().extend(0.0),
            )),
            Name::new("editor_help_box"),
            menu_layer(),
        ))
        .with_children(|parent| {
            parent.spawn((
                EditorGrayHelpBox,
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::GRAY,
                        ..default()
                    },
                    transform: Transform {
                        translation: Vec3::new(0.0, 0.0, -1.0),
                        scale: Vec3::new(
                            help_config.box_size.x as f32,
                            help_config.box_size.y as f32,
                            1.0,
                        ),
                        ..default()
                    },
                    ..default()
                },
                menu_layer(),
            ));
            parent.spawn(HelpBoxKVBundle {
                kv: HelpBoxKV {
                    key: "Mode".to_string(),
                    value: "".to_string(),
                },
                text: Text2dBundle {
                    // transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                    transform: Transform {
                        translation: Vec3::new(0.0, 0.0, 1.0),
                        ..default()
                    },
                    text_anchor: bevy::sprite::Anchor::Center,
                    ..default()
                },
                render_layers: menu_layer(),
            });
        });

    // Help/command bar
    let bar_center = get_bar_center(&help_config);
    commands
        .spawn((
            HelpBarData {
                input: String::new(),
                captured: false,
                submitted: false,
                output: vec![],
            },
            SpatialBundle::from_transform(Transform::from_translation(
                bar_center.as_vec2().extend(0.0),
            )),
            Name::new("help_bar_data"),
        ))
        .with_children(|parent| {
            // Input bar
            let input_center = IVec2::new(0, -help_config.bar_size.y / 4);
            let input_width = help_config.bar_size.x;
            let input_height = help_config.bar_size.y / 2;
            parent.spawn((
                HelpBarInput,
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::GRAY,
                        ..default()
                    },
                    transform: Transform {
                        translation: input_center.extend(0).as_vec3(),
                        scale: Vec3::new(input_width as f32, input_height as f32, 1.0),
                        ..default()
                    },
                    ..default()
                },
                menu_layer(),
            ));
            parent.spawn((
                HelpBarInput,
                Text2dBundle {
                    transform: Transform::from_translation(input_center.extend(1).as_vec3()),
                    text_anchor: bevy::sprite::Anchor::Center,
                    ..default()
                },
                menu_layer(),
            ));
            // Output bar
            let output_center = IVec2::new(0, help_config.bar_size.y / 4);
            let output_width = help_config.bar_size.x;
            let output_height = help_config.bar_size.y / 2;
            parent.spawn((
                HelpBarOutput,
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::DARK_GRAY,
                        ..default()
                    },
                    transform: Transform {
                        translation: output_center.extend(0).as_vec3(),
                        scale: Vec3::new(output_width as f32, output_height as f32, 1.0),
                        ..default()
                    },
                    ..default()
                },
                menu_layer(),
            ));
            parent.spawn((
                HelpBarOutput,
                Text2dBundle {
                    transform: Transform::from_translation(output_center.extend(1).as_vec3()),
                    text_anchor: bevy::sprite::Anchor::Center,
                    ..default()
                },
                menu_layer(),
            ));
        });
}

pub(super) fn update_editor_help_box(
    mut kvs: Query<(&mut HelpBoxKV, &mut Text)>,
    gs: Res<GameState>,
    help_config: Res<EditorHelpConfig>,
    mut editor_help: Query<&mut Transform, With<EditorHelpBox>>,
    mut gray_box: Query<&mut Transform, (With<EditorGrayHelpBox>, Without<EditorHelpBox>)>,
) {
    let mode = gs.get_editing_mode();
    let center = get_box_center(&help_config);
    let width = help_config.box_size.x;
    let height = help_config.box_size.y;
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
                None => "TESTING".to_string(),
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
                font_size: help_config.box_font_size,
                ..default()
            },
        )]
    }
}

pub(super) fn update_editor_help_bar(
    help_config: Res<EditorHelpConfig>,
    mut help_bar: Query<(&mut HelpBarData, &mut Transform)>,
    mut input_background: Query<
        (&mut Transform, &mut Sprite),
        (
            With<HelpBarInput>,
            Without<HelpBarData>,
            Without<HelpBarOutput>,
        ),
    >,
    mut output_background: Query<
        &mut Transform,
        (
            With<Sprite>,
            With<HelpBarOutput>,
            Without<HelpBarData>,
            Without<HelpBarInput>,
        ),
    >,
    mut input_text: Query<
        (&mut Transform, &mut Text),
        (
            Without<Sprite>,
            With<HelpBarInput>,
            Without<HelpBarData>,
            Without<HelpBarOutput>,
        ),
    >,
    mut output_text: Query<
        (&mut Transform, &mut Text),
        (
            Without<Sprite>,
            With<HelpBarOutput>,
            Without<HelpBarData>,
            Without<HelpBarInput>,
        ),
    >,
) {
    let (
        Ok(mut help_bar),
        Ok(mut input_background),
        Ok(mut output_background),
        Ok(mut input_text),
        Ok(mut output_text),
    ) = (
        help_bar.get_single_mut(),
        input_background.get_single_mut(),
        output_background.get_single_mut(),
        input_text.get_single_mut(),
        output_text.get_single_mut(),
    )
    else {
        return;
    };
    // Update all the transforms
    let bar_center = get_bar_center(&help_config);
    let bar_width = help_config.bar_size.x;
    let bar_height = help_config.bar_size.y;
    let input_center = IVec2::new(0, -bar_height / 4);
    let input_width = bar_width;
    let input_height = bar_height / 2;
    let output_center = IVec2::new(0, bar_height / 4);
    let output_width = bar_width;
    let output_height = bar_height / 2;
    help_bar.1.translation.x = bar_center.x as f32;
    help_bar.1.translation.y = bar_center.y as f32;
    input_background.0.translation.x = input_center.x as f32;
    input_background.0.translation.y = input_center.y as f32;
    input_background.0.scale.x = input_width as f32;
    input_background.0.scale.y = input_height as f32;
    input_background.1.color = if help_bar.0.captured {
        Color::YELLOW
    } else {
        Color::GRAY
    };
    input_text.0.translation.x = input_center.x as f32;
    input_text.0.translation.y = input_center.y as f32;
    output_background.translation.x = output_center.x as f32;
    output_background.translation.y = output_center.y as f32;
    output_background.scale.x = output_width as f32;
    output_background.scale.y = output_height as f32;
    output_text.0.translation.x = output_center.x as f32;
    output_text.0.translation.y = output_center.y as f32;

    // Update the text
    input_text.1.sections = vec![TextSection::new(
        format!("{}", help_bar.0.input),
        TextStyle {
            font_size: help_config.bar_font_size,
            color: Color::BLACK,
            ..default()
        },
    )];
    output_text.1.sections = vec![TextSection::new(
        format!(
            "{}",
            help_bar
                .0
                .output
                .iter()
                .last()
                .cloned()
                .unwrap_or("(no output yet)".to_string())
        ),
        TextStyle {
            font_size: help_config.bar_font_size,
            ..default()
        },
    )];
}

pub(super) fn editor_help_input(
    mut help_bar: Query<&mut HelpBarData>,
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
    mut evr_char: EventReader<ReceivedCharacter>,
) {
    let Ok(mut help_bar) = help_bar.get_single_mut() else {
        return;
    };
    if !help_bar.captured {
        help_bar.captured = keyboard.pressed(KeyCode::Slash);
        evr_char.read();
    }
    if help_bar.captured {
        if keyboard.pressed(KeyCode::Escape) {
            help_bar.input = String::new();
            help_bar.captured = false;
        } else if keyboard.pressed(KeyCode::Enter) {
            help_bar.captured = false;
            help_bar.submitted = true;
        } else {
            let chars: Vec<&ReceivedCharacter> = evr_char.read().collect();
            for char in chars {
                if !["/", "\n", "\u{8}"].contains(&char.char.as_str()) {
                    help_bar.input += &char.char.to_string();
                }
                if char.char.as_str() == "\u{8}" {
                    // Backspace
                    help_bar.input.pop();
                }
            }
        }
        keyboard.reset_all();
    }
}

pub(super) fn read_editor_help_output(
    mut help_bar: Query<&mut HelpBarData>,
    mut events: EventReader<HelpBarEvent>,
) {
    let Ok(mut help_bar) = help_bar.get_single_mut() else {
        return;
    };
    for event in events.read() {
        help_bar.output.push(event.0.clone());
    }
}

pub(super) fn run_help_bar_command(
    world: &mut World,
    params: &mut SystemState<(
        Query<&mut HelpBarData>,
        EventWriter<HelpBarEvent>,
        EventWriter<SaveEditorEvent>,
        EventWriter<LoadEditorEvent>,
        EventWriter<ExportLevelEvent>,
        EventWriter<SetMetaState>,
        EventWriter<CreateStandaloneFieldEvent>,
        Query<&EStart>,
        Query<&EGoal>,
        Query<Entity, With<EditingSceneRoot>>,
    )>,
) {
    let (
        mut help_bar,
        mut event,
        mut save_editor_writer,
        mut load_editor_writer,
        mut export_level_writer,
        mut gs_writer,
        mut create_standalone_field_writer,
        estart_q,
        egoal_q,
        eroot,
    ) = params.get_mut(world);
    let Ok(mut help_bar) = help_bar.get_single_mut() else {
        return;
    };
    let Ok(eroot) = eroot.get_single() else {
        return;
    };
    if !help_bar.submitted {
        return;
    }
    let mut send_output = |msg: &str| {
        event.send(HelpBarEvent(msg.to_string()));
    };
    let mut get_one_nonempty_param = |input: &str, prefix: &str| {
        let Some(name) = input.strip_prefix(prefix) else {
            send_output(&format!("Invalid get_one_nonempty_param: {}", input));
            return None;
        };
        if name.len() == 0 {
            send_output(&format!("Invalid get_one_nonempty_param: empty param"));
            return None;
        }
        Some(name.to_string())
    };
    let get_two_f32s = |input: &str, prefix: &str| {
        // Hackity hack
        // Resisting the urge to waste 3 hours making this better
        let Some(combined) = input.strip_prefix(prefix) else {
            return None;
        };
        let parts = combined.split(" ");
        let v = parts.into_iter().collect::<Vec<_>>();
        if v.len() != 2 {
            return None;
        }
        let f1 = v[0].parse::<f32>();
        let f2 = v[1].parse::<f32>();
        match (f1, f2) {
            (Ok(f1), Ok(f2)) => Some((f1, f2)),
            _ => None,
        }
    };

    let input = help_bar.input.clone();
    help_bar.submitted = false;
    help_bar.input = String::new();

    if &input == "print out" {
        println!("Output:");
        for thing in help_bar.output.iter() {
            println!("{}", thing);
        }
        send_output("HelpBar output printed to terminal");
    } else if input.starts_with("save ") {
        match get_one_nonempty_param(&input, "save ") {
            Some(name) => {
                save_editor_writer.send(SaveEditorEvent(name));
            }
            None => send_output("Must save to a name"),
        }
    } else if input.starts_with("load ") {
        match get_one_nonempty_param(&input, "load ") {
            Some(name) => {
                load_editor_writer.send(LoadEditorEvent(name));
            }
            None => send_output("Must load from a name"),
        }
    } else if input.starts_with("field ") {
        let Some((x, y)) = get_two_f32s(&input, "field ") else {
            send_output("That doesn't look like two floats...");
            return;
        };
        create_standalone_field_writer.send(CreateStandaloneFieldEvent(Vec2::new(x, y)));
    } else if &input == "test" {
        if estart_q.iter().len() == 0 {
            send_output("You must spawn a start position before testing");
        } else if egoal_q.iter().len() == 0 {
            send_output("You must spawn a goal position before testing");
        } else {
            gs_writer.send(SetMetaState(EditorState::Testing.to_meta_state()));
        }
    } else if &input == "edit" {
        gs_writer.send(SetMetaState(EditingMode::Free.to_meta_state()));
    } else if input.starts_with("export ") {
        let Some(name) = input.strip_prefix("export ") else {
            send_output(&format!("Invalid export: {}", input));
            return;
        };
        if name.len() == 0 {
            send_output(&format!("Invalid export: name cannot be empty"));
            return;
        }

        export_level_writer.send(ExportLevelEvent(name.to_string()));
    } else if &input == "clear" {
        world.entity_mut(eroot).despawn_descendants();
    } else {
        send_output(&format!("INVALID COMMAND: {}", input));
    }
}

pub(super) fn destroy_editor_help(
    mut commands: Commands,
    help_box: Query<Entity, With<EditorHelpBox>>,
    help_bar: Query<Entity, With<HelpBarData>>,
) {
    for eid in help_box.iter().chain(help_bar.iter()) {
        commands.entity(eid).despawn_recursive();
    }
}
