use bevy::prelude::*;

use crate::{
    drawing::layering::menu_layer,
    meta::{
        consts::{MENU_HEIGHT, MENU_WIDTH},
        game_state::{GameState, MetaState, PrevGameState, SetPaused},
    },
};

use super::{
    button::{MenuButton, MenuButtonBundle},
    placement::GameRelativePlacement,
};

pub(super) fn did_pause_start(old_state: Res<PrevGameState>, new_state: Res<GameState>) -> bool {
    !old_state.paused && new_state.paused
}

pub(super) fn did_pause_end(old_state: Res<PrevGameState>, new_state: Res<GameState>) -> bool {
    old_state.paused && !new_state.paused
}

pub fn is_paused(state: Res<GameState>) -> bool {
    state.paused
}

pub(super) fn start_pause(
    mut pause_writer: EventWriter<SetPaused>,
    gs: Res<GameState>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) && !gs.is_in_menu() {
        pause_writer.send(SetPaused(true));
    }
}

#[derive(Component)]
pub(super) struct PauseRoot;
impl PauseRoot {
    fn new_root(name: &str) -> impl Bundle {
        (
            PauseRoot,
            SpatialBundle::default(),
            Name::new(format!("pause_root_{name}")),
        )
    }
}

pub(super) fn setup_pause(mut gs: ResMut<GameState>, mut commands: Commands) {
    if !gs.paused {
        // Shouldn't happen
        return;
    }
    let spawn_background = |parent: &mut ChildBuilder, color: Color| {
        parent.spawn((
            Name::new("paused_dark_box"),
            SpriteBundle {
                sprite: Sprite { color, ..default() },
                transform: Transform {
                    scale: Vec3::new(MENU_WIDTH as f32, MENU_HEIGHT as f32, 1.0),
                    translation: Vec3::new(0.0, 0.0, 10.0),
                    ..default()
                },
                ..default()
            },
            menu_layer(),
        ));
    };
    match &gs.meta {
        MetaState::Menu(_) => {
            gs.paused = false;
            return;
        }
        MetaState::Level(level_state) => {
            commands
                .spawn(PauseRoot::new_root("level"))
                .with_children(|parent| {
                    spawn_background(parent, Color::rgba(0.2, 0.2, 0.2, 0.8));
                    parent.spawn(MenuButtonBundle::new(
                        MenuButton::basic("test", "Test"),
                        GameRelativePlacement::new(IVec3::new(0, 0, 12), 1.0),
                    ));
                });
        }
        MetaState::Editor(editor_state) => {}
    }
}

pub(super) fn update_pause(
    mut set_paused: EventWriter<SetPaused>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        // You can always press escape to exit a pause
        set_paused.send(SetPaused(false));
        return;
    }
}

pub(super) fn destroy_pause(pause_root: Query<Entity, With<PauseRoot>>, mut commands: Commands) {
    for eid in pause_root.iter() {
        commands.entity(eid).despawn_recursive();
    }
}
