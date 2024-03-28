use crate::{
    meta::{
        game_state::{entered_editor, in_editor, left_editor, EditorState, GameState, MetaState},
        level_data::LevelData,
    },
    when_becomes_false, when_becomes_true,
};
use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;

use self::{
    help::{
        setup_editor_help, setup_editor_help_config, teardown_editor_help, update_editor_help,
        update_editor_help_config, EditorHelpConfig,
    },
    planet::{drive_planet_meshes, planet_state_input},
    point::{delete_points, hover_points, move_points, select_points, spawn_points},
};

pub mod help;
pub mod input;
pub mod planet;
pub mod point;

fn is_editing_helper(gs: &GameState) -> bool {
    match gs.meta {
        MetaState::Editor(editor_state) => match editor_state {
            EditorState::Editing(_) => true,
            _ => false,
        },
        _ => false,
    }
}
pub fn is_editing(gs: Res<GameState>) -> bool {
    is_editing_helper(&gs)
}
when_becomes_true!(is_editing_helper, entered_editing);
when_becomes_false!(is_editing_helper, left_editing);

fn is_testing_helper(gs: &GameState) -> bool {
    match gs.meta {
        MetaState::Editor(editor_state) => match editor_state {
            EditorState::Testing => true,
            _ => false,
        },
        _ => false,
    }
}
pub fn is_testing(gs: Res<GameState>) -> bool {
    is_testing_helper(&gs)
}
when_becomes_true!(is_testing_helper, entered_testing);
when_becomes_false!(is_testing_helper, left_testing);

#[derive(Component)]
struct LevelEditingHandle(pub Handle<LevelData>);

#[derive(Resource, bevy::asset::Asset, bevy::reflect::TypePath)]
struct LevelEditingData(pub LevelData);

fn setup_editor(mut commands: Commands, asset_server: Res<AssetServer>) {
    let handle = asset_server.load::<LevelData>("levels/editing.level.ron");
    commands.spawn(LevelEditingHandle(handle));
}

fn teardown_editor(mut commands: Commands, handle: Query<Entity, With<LevelEditingHandle>>) {
    if let Ok(id) = handle.get_single() {
        commands.entity(id).despawn_recursive();
    }
}

fn watch_level_editing_asset(
    handle: Query<&LevelEditingHandle>,
    asset: Res<Assets<LevelData>>,
    mut res: ResMut<LevelEditingData>,
) {
    let Ok(handle) = handle.get_single() else {
        return;
    };
    if let Some(data) = asset.get(handle.0.id()) {
        if *data != res.0 {
            res.0 = data.clone();
            println!("updated data to: {:?}", data);
        }
    }
}

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        // Meta-editor stuff
        app.insert_resource(LevelEditingData(LevelData::default()));
        app.add_plugins(RonAssetPlugin::<LevelData>::new(&["level.ron"]));
        app.add_systems(Update, setup_editor.run_if(entered_editor));
        app.add_systems(Update, teardown_editor.run_if(left_editor));
        app.add_systems(Update, watch_level_editing_asset.run_if(in_editor));

        // Help system
        app.add_plugins(RonAssetPlugin::<EditorHelpConfig>::new(&[
            "editor_help.ron",
        ]));
        app.add_systems(Startup, setup_editor_help_config);
        app.add_systems(Update, update_editor_help_config);
        app.add_systems(Update, setup_editor_help.run_if(entered_editor));
        app.add_systems(Update, update_editor_help.run_if(in_editor));
        app.add_systems(Update, teardown_editor_help.run_if(left_editor));

        // Points
        app.add_systems(
            Update,
            (
                hover_points,
                spawn_points,
                select_points,
                delete_points,
                move_points,
            )
                .chain()
                .run_if(is_editing),
        );

        // Planets
        app.add_systems(
            Update,
            (planet_state_input, drive_planet_meshes)
                .chain()
                .run_if(is_editing)
                .after(move_points), // should be after the last thing in point chain
        );
    }
}
