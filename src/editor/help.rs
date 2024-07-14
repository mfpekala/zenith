use bevy::{prelude::*, render::view::RenderLayers, text::Text2dBounds};
use clap::{Arg, Command};

use crate::{
    drawing::layering::menu_layer,
    meta::{
        consts::MENU_GROWTH_F32,
        game_state::{EditingState, EditorState, GameState, SetMetaState},
    },
};

use super::{oneshots::EOneshots, transitions::HRootEid};

// THE STUFF TO CARE ABOUT

pub(super) fn spawn_help(In(()): In<()>, hroot: Res<HRootEid>, mut commands: Commands) {
    commands.entity(hroot.0).with_children(|parent| {
        let (help_bar, input_bg, input_text, output_bg, output_text) = HelpBarData::new_hierarchy();
        parent.spawn(help_bar).with_children(|bar| {
            bar.spawn(input_bg);
            bar.spawn(input_text);
            bar.spawn(output_bg);
            bar.spawn(output_text);
        });
    });
}

pub(super) fn submit_help_command(
    In(command): In<String>,
    gs: Res<GameState>,
    mut meta_writer: EventWriter<SetMetaState>,
) {
    let parts = format!("help {command}");
    let parts = parts.split_whitespace().collect::<Vec<_>>();
    let Ok(matches) = Command::new("help")
        .subcommand(Command::new("test"))
        .subcommand(Command::new("edit"))
        .subcommand(Command::new("field").arg(Arg::new("x")).arg(Arg::new("y")))
        .try_get_matches_from(parts)
    else {
        warn!("Invalid command1: {command}");
        return;
    };
    let Some(m) = matches.subcommand() else {
        warn!("Invalid command2: {command}");
        return;
    };
    let Some(editing_state) = gs.get_editor_state() else {
        warn!("GameState looks like {gs:?}, which is not an editing state");
        return;
    };
    match m {
        ("test", _) => {
            if let EditorState::Editing(_) = editing_state {
                // TODO: Verify that there exists a start
                meta_writer.send(SetMetaState(EditorState::Testing.to_meta_state()));
            }
        }
        ("edit", _) => {
            if let EditorState::Testing = editing_state {
                meta_writer.send(SetMetaState(
                    EditorState::Editing(EditingState::blank()).to_meta_state(),
                ));
            }
        }
        ("field", args) => {
            println!(
                "got field with {:?} {:?}",
                args.try_get_one::<String>("x"),
                args.try_get_one::<String>("y")
            );
        }
        _ => {
            warn!("Invalid command3: {command}");
            return;
        }
    }
}

// VERBOSE BULLSHIT
#[derive(Component, Debug, Reflect)]
pub(super) struct HelpBarData {
    pub input: String,
    pub captured: bool,
    pub output: Vec<String>,
}

#[derive(Component, Debug)]
pub(super) struct HelpBarInputBg;
#[derive(Bundle)]
struct HelpBarInputBgBundle {
    name: Name,
    marker: HelpBarInputBg,
    sprite: SpriteBundle,
    render_layers: RenderLayers,
}
impl HelpBarInputBgBundle {
    fn new() -> Self {
        Self {
            name: Name::new("input_bg"),
            marker: HelpBarInputBg,
            sprite: SpriteBundle {
                sprite: Sprite {
                    color: Color::GRAY,
                    ..default()
                },
                transform: Transform {
                    translation: Vec3::new(0.0, -5.0, -1.0) * MENU_GROWTH_F32,
                    scale: (Vec2::new(160.0, 10.0) * MENU_GROWTH_F32).extend(1.0),
                    ..default()
                },
                ..default()
            },
            render_layers: menu_layer(),
        }
    }
}
#[derive(Component, Debug)]
pub(super) struct HelpBarInputText;
#[derive(Bundle)]
pub(super) struct HelpBarInputTextBundle {
    name: Name,
    marker: HelpBarInputText,
    text: Text2dBundle,
    render_layers: RenderLayers,
}
impl HelpBarInputTextBundle {
    fn new() -> Self {
        Self {
            name: Name::new("input_text"),
            marker: HelpBarInputText,
            text: Text2dBundle {
                text: Text::from_section(
                    "",
                    TextStyle {
                        font_size: 48.0,
                        ..default()
                    },
                )
                .with_justify(JustifyText::Left),
                text_2d_bounds: Text2dBounds {
                    size: Vec2::new(160.0, 10.0) * MENU_GROWTH_F32,
                },
                transform: Transform::from_translation(Vec3::new(0.0, -5.0, 0.0) * MENU_GROWTH_F32),
                ..default()
            },
            render_layers: menu_layer(),
        }
    }
}

#[derive(Component, Debug)]
pub(super) struct HelpBarOutputBg;
#[derive(Bundle)]
struct HelpBarOutputBgBundle {
    name: Name,
    marker: HelpBarOutputBg,
    sprite: SpriteBundle,
    render_layers: RenderLayers,
}
impl HelpBarOutputBgBundle {
    fn new() -> Self {
        Self {
            name: Name::new("output_bg"),
            marker: HelpBarOutputBg,
            sprite: SpriteBundle {
                sprite: Sprite {
                    color: Color::DARK_GRAY,
                    ..default()
                },
                transform: Transform {
                    translation: Vec3::new(0.0, 5.0, -1.0) * MENU_GROWTH_F32,
                    scale: (Vec2::new(160.0, 10.0) * MENU_GROWTH_F32).extend(1.0),
                    ..default()
                },
                ..default()
            },
            render_layers: menu_layer(),
        }
    }
}

#[derive(Component, Debug)]
pub(super) struct HelpBarOutputText;
#[derive(Bundle)]
struct HelpBarOutputTextBundle {
    name: Name,
    marker: HelpBarOutputText,
    text: Text2dBundle,
    render_layers: RenderLayers,
}
impl HelpBarOutputTextBundle {
    fn new() -> Self {
        Self {
            name: Name::new("output_text"),
            marker: HelpBarOutputText,
            text: Text2dBundle {
                text: Text::from_section(
                    "",
                    TextStyle {
                        font_size: 48.0,
                        ..default()
                    },
                )
                .with_justify(JustifyText::Left),
                text_2d_bounds: Text2dBounds {
                    size: Vec2::new(160.0, 10.0) * MENU_GROWTH_F32,
                },
                transform: Transform::from_translation(Vec3::new(0.0, 5.0, 0.0) * MENU_GROWTH_F32),
                ..default()
            },
            render_layers: menu_layer(),
        }
    }
}

impl HelpBarData {
    fn new_hierarchy() -> (
        impl Bundle,
        HelpBarInputBgBundle,
        HelpBarInputTextBundle,
        HelpBarOutputBgBundle,
        HelpBarOutputTextBundle,
    ) {
        let me = (
            Name::new("help_bar_data"),
            HelpBarData {
                input: String::new(),
                captured: false,
                output: vec![],
            },
            SpatialBundle::from_transform(Transform::from_translation(
                Vec3::new(80.0, -80.0, 0.0) * MENU_GROWTH_F32,
            )),
        );
        (
            me,
            HelpBarInputBgBundle::new(),
            HelpBarInputTextBundle::new(),
            HelpBarOutputBgBundle::new(),
            HelpBarOutputTextBundle::new(),
        )
    }
}

pub(super) fn update_editor_help_bar(
    help_bar: Query<&HelpBarData>,
    mut input_background: Query<&mut Sprite, With<HelpBarInputBg>>,
    mut input_text: Query<&mut Text, With<HelpBarInputText>>,
    mut output_text: Query<&mut Text, (With<HelpBarOutputText>, Without<HelpBarInputText>)>,
) {
    let (Ok(help_bar), Ok(mut input_background), Ok(mut input_text), Ok(mut output_text)) = (
        help_bar.get_single(),
        input_background.get_single_mut(),
        input_text.get_single_mut(),
        output_text.get_single_mut(),
    ) else {
        return;
    };
    input_background.color = if help_bar.captured {
        Color::YELLOW
    } else {
        Color::GRAY
    };

    // Update the text
    input_text.sections = vec![TextSection::new(
        format!("{}", help_bar.input),
        TextStyle {
            font_size: 48.0,
            color: Color::BLACK,
            ..default()
        },
    )];
    output_text.sections = vec![TextSection::new(
        format!(
            "{}",
            help_bar
                .output
                .iter()
                .last()
                .cloned()
                .unwrap_or("(no output yet)".to_string())
        ),
        TextStyle {
            font_size: 48.0,
            ..default()
        },
    )];
}

pub(super) fn editor_help_input(
    mut help_bar: Query<&mut HelpBarData>,
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
    mut evr_char: EventReader<ReceivedCharacter>,
    oneshots: Res<EOneshots>,
    mut commands: Commands,
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
            help_bar.captured = false;
            help_bar.input = String::new();
        } else if keyboard.pressed(KeyCode::Enter) {
            commands.run_system_with_input(oneshots.submit_help_command, help_bar.input.clone());
            help_bar.captured = false;
            help_bar.input = String::new();
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
