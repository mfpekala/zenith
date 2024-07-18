use bevy::{prelude::*, render::view::RenderLayers, text::Text2dBounds, utils::HashMap};
use clap::{Arg, Command};

use crate::{
    drawing::layering::menu_layer,
    meta::{
        consts::MENU_GROWTH_F32,
        game_state::{EditingMode, EditingState, EditorState, GameState, SetMetaState},
    },
};

use super::{eoneshots::EOneshots, transitions::HRootEid};

// THE STUFF TO CARE ABOUT

pub(super) fn spawn_help(In(()): In<()>, hroot: Res<HRootEid>, mut commands: Commands) {
    commands.entity(hroot.0).with_children(|parent| {
        let (help_bar, input_bg, input_text, output_bg, output_text) = HelpBarData::new_hierarchy();
        parent.spawn(help_bar).with_children(|bar| {
            bar.spawn(input_bg).with_children(|input| {
                input.spawn(input_text);
            });
            bar.spawn(output_bg).with_children(|output| {
                output.spawn(output_text);
            });
        });
        HelpBoxData::spawn_hierarchy(parent);
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

pub(super) fn update_help_box(
    gs: Res<GameState>,
    mut help_box_q: Query<(&mut HelpBoxData, &mut Visibility)>,
    mut help_box_fg_q: Query<&mut HelpTextFg, With<HelpBoxFg>>,
) {
    let Ok((mut help_box, mut visibility)) = help_box_q.get_single_mut() else {
        return;
    };
    let Ok(mut help_box_fg) = help_box_fg_q.get_single_mut() else {
        return;
    };
    let Some(editing_mode) = gs.get_editing_mode() else {
        return;
    };
    match editing_mode {
        EditingMode::Free => {
            help_box.pairs.remove("mode");
        }
        EditingMode::CreatingRock(eid) => {
            help_box
                .pairs
                .insert("mode".into(), format!("crock({eid:?})"));
        }
        EditingMode::EditingRock(eid) => {
            help_box
                .pairs
                .insert("mode".into(), format!("erock({eid:?})"));
        }
        EditingMode::CreatingField(eid) => {
            help_box
                .pairs
                .insert("mode".into(), format!("cfield({eid:?})"));
        }
        EditingMode::EditingField(eid) => {
            help_box
                .pairs
                .insert("mode".into(), format!("efield({eid:?})"));
        }
    }
    help_box_fg.content = String::new();
    for (key, value) in help_box.pairs.iter() {
        help_box_fg.content.push_str(&format!("{key}: {value}"));
    }
    *visibility = if help_box_fg.content.len() > 0 {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
}

// VERBOSE BULLSHIT

/// Specific data for managing the input bar
#[derive(Component, Debug, Reflect)]
pub(super) struct HelpBarData {
    pub input: String,
    pub captured: bool,
    pub output: Vec<String>,
}

/// Common piece used for displaying the background of a text box
#[derive(Component, Debug, Reflect)]
pub(super) struct HelpTextBg {
    pub color: Color,
    /// Center pos IN SCREEN SPACE (not menu space)
    pub pos: IVec3,
    /// Dimensions of the box IN SCREEN SPACE (not menu space)
    pub dims: UVec2,
}
impl HelpTextBg {
    fn new(color: Color, pos: IVec3, dims: UVec2) -> Self {
        Self { color, pos, dims }
    }

    fn to_bundle(self) -> HelpTextBgBundle {
        HelpTextBgBundle::new(self)
    }
}
#[derive(Bundle)]
struct HelpTextBgBundle {
    driver: HelpTextBg,
    sprite: SpriteBundle,
    render_layers: RenderLayers,
}
impl HelpTextBgBundle {
    fn new(driver: HelpTextBg) -> Self {
        Self {
            sprite: SpriteBundle {
                sprite: Sprite {
                    color: driver.color,
                    custom_size: Some(driver.dims.as_vec2()),
                    ..default()
                },
                transform: Transform {
                    translation: driver.pos.as_vec3() * MENU_GROWTH_F32,
                    ..default()
                },
                ..default()
            },
            render_layers: menu_layer(),
            driver,
        }
    }
}

/// Common piece used for displaying the actual text of a text box
#[derive(Component, Debug, Reflect)]
pub(super) struct HelpTextFg {
    pub color: Color,
    pub font_size: f32,
    pub content: String,
}
impl HelpTextFg {
    fn new(color: Color, font_size: f32, content: String) -> Self {
        Self {
            color,
            font_size,
            content,
        }
    }

    fn to_bundle(self) -> HelpTextFgBundle {
        HelpTextFgBundle::new(self)
    }
}
#[derive(Bundle)]
struct HelpTextFgBundle {
    driver: HelpTextFg,
    text: Text2dBundle,
    render_layers: RenderLayers,
}
impl HelpTextFgBundle {
    fn new(driver: HelpTextFg) -> Self {
        Self {
            text: Text2dBundle {
                text: Text::from_section(
                    &driver.content,
                    TextStyle {
                        font_size: driver.font_size,
                        color: driver.color,
                        ..default()
                    },
                )
                .with_justify(JustifyText::Left),
                transform: Transform::from_translation(Vec3::Z),
                ..default()
            },
            render_layers: menu_layer(),
            driver,
        }
    }
}

#[derive(Component, Debug)]
pub(super) struct HelpBarInputBg;
#[derive(Bundle)]
struct HelpBarInputBgBundle {
    name: Name,
    marker: HelpBarInputBg,
    bg: HelpTextBgBundle,
}
impl HelpBarInputBgBundle {
    fn new() -> Self {
        Self {
            name: Name::new("input_bg"),
            marker: HelpBarInputBg,
            bg: HelpTextBg::new(Color::GRAY, IVec3::new(0, -5, -1), UVec2::new(160, 10))
                .to_bundle(),
        }
    }
}
#[derive(Component, Debug)]
pub(super) struct HelpBarInputText;
#[derive(Bundle)]
pub(super) struct HelpBarInputTextBundle {
    name: Name,
    marker: HelpBarInputText,
    fg: HelpTextFgBundle,
}
impl HelpBarInputTextBundle {
    fn new() -> Self {
        Self {
            name: Name::new("input_text"),
            marker: HelpBarInputText,
            fg: HelpTextFg::new(Color::BLACK, 48.0, "".into()).to_bundle(),
        }
    }
}

#[derive(Component, Debug)]
pub(super) struct HelpBarOutputBg;
#[derive(Bundle)]
struct HelpBarOutputBgBundle {
    name: Name,
    marker: HelpBarOutputBg,
    bg: HelpTextBgBundle,
}
impl HelpBarOutputBgBundle {
    fn new() -> Self {
        Self {
            name: Name::new("output_bg"),
            marker: HelpBarOutputBg,
            bg: HelpTextBg::new(Color::DARK_GRAY, IVec3::new(0, 5, -1), UVec2::new(160, 10))
                .to_bundle(),
        }
    }
}

#[derive(Component, Debug)]
pub(super) struct HelpBarOutputText;
#[derive(Bundle)]
struct HelpBarOutputTextBundle {
    name: Name,
    marker: HelpBarOutputText,
    fg: HelpTextFgBundle,
}
impl HelpBarOutputTextBundle {
    fn new() -> Self {
        Self {
            name: Name::new("output_text"),
            marker: HelpBarOutputText,
            fg: HelpTextFg::new(Color::WHITE, 48.0, "".into()).to_bundle(),
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

#[derive(Component, Debug)]
pub(super) struct HelpBoxBg;
#[derive(Component, Debug)]
pub(super) struct HelpBoxFg;

/// Key/value data that is shown in the bottom left corner of the screen
#[derive(Component, Debug, Reflect)]
pub(super) struct HelpBoxData {
    pairs: HashMap<String, String>,
}
impl HelpBoxData {
    fn spawn_hierarchy(commands: &mut ChildBuilder) {
        commands
            .spawn((
                Name::new("help_box_data"),
                HelpBoxData { pairs: default() },
                SpatialBundle::from_transform(Transform::from_translation(
                    Vec3::new(-130.0, -75.0, 0.0) * MENU_GROWTH_F32,
                )),
            ))
            .with_children(|parent| {
                parent
                    .spawn((
                        Name::new("help_box_bg"),
                        HelpBoxBg,
                        HelpTextBg::new(Color::ANTIQUE_WHITE, IVec3::ZERO, UVec2::new(60, 30))
                            .to_bundle(),
                    ))
                    .with_children(|bg| {
                        bg.spawn((
                            Name::new("help_box_fg"),
                            HelpBoxFg,
                            HelpTextFg::new(Color::BLACK, 36.0, "".into()).to_bundle(),
                        ));
                    });
            });
    }
}

/// Common logic for updating text fg and bg
pub(super) fn update_editor_texts(
    mut bgs: Query<(&HelpTextBg, &mut Sprite, &mut Transform)>,
    mut fgs: Query<(&HelpTextFg, &Parent, &mut Text, &mut Text2dBounds)>,
) {
    for (bg, mut sprite, mut tran) in bgs.iter_mut() {
        sprite.color = bg.color;
        sprite.custom_size = Some(bg.dims.as_vec2() * MENU_GROWTH_F32);
        tran.translation = bg.pos.as_vec3() * MENU_GROWTH_F32;
    }

    for (fg, parent, mut text, mut bounds) in fgs.iter_mut() {
        let bg = bgs.get(parent.get()).unwrap().0;
        text.sections = vec![TextSection::new(
            &fg.content,
            TextStyle {
                font_size: fg.font_size,
                color: fg.color,
                ..default()
            },
        )];
        bounds.size = bg.dims.as_vec2() * MENU_GROWTH_F32;
    }
}

pub(super) fn update_editor_help_bar(
    help_bar: Query<&HelpBarData>,
    mut input_bg: Query<&mut HelpTextBg, With<HelpBarInputBg>>,
    mut input_text: Query<&mut HelpTextFg, With<HelpBarInputText>>,
    mut output_text: Query<&mut HelpTextFg, (With<HelpBarOutputText>, Without<HelpBarInputText>)>,
) {
    let (Ok(help_bar), Ok(mut input_bg), Ok(mut input_text), Ok(mut output_text)) = (
        help_bar.get_single(),
        input_bg.get_single_mut(),
        input_text.get_single_mut(),
        output_text.get_single_mut(),
    ) else {
        return;
    };
    input_bg.color = if help_bar.captured {
        Color::YELLOW
    } else {
        Color::GRAY
    };

    // Update the text
    input_text.content = format!("{}", help_bar.input);
    output_text.content = format!(
        "{}",
        help_bar
            .output
            .iter()
            .last()
            .cloned()
            .unwrap_or("(no output yet)".to_string())
    );
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
