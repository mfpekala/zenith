use crate::{
    drawing::{
        bordered_mesh::{BorderMeshType, BorderedMatData, BorderedMesh},
        mesh::{ScrollSprite, SpriteInfo},
    },
    meta::{
        game_state::{entered_editor, in_editor, left_editor, EditorState, GameState, MetaState},
        level_data::LevelData,
    },
    physics::dyno::IntMoveable,
    uid::UIdMarker,
    when_becomes_false, when_becomes_true,
};
use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;

use self::{
    help::{
        editor_help_input, read_editor_help_output, run_help_bar_command, setup_editor_help,
        setup_editor_help_config, teardown_editor_help, update_editor_help_bar,
        update_editor_help_box, update_editor_help_config, EditorHelpConfig, HelpBarEvent,
    },
    planet::{
        debug_planets, draw_field_parents, drive_planet_meshes, fix_dangling_mesh_ids,
        handle_feral_points, make_new_field, nudge_fields, planet_state_input, redo_fields,
        remove_field, resolve_pending_fields, update_field_gravity, EPlanet,
    },
    point::{
        delete_points, hover_points, move_points, point_select_shortcuts, select_points,
        spawn_points, update_point_sprites, EPoint,
    },
    save::{
        cleanup_load, connect_parents, fix_after_load, load_editor, resolve_holes, save_editor,
        CleanupLoadEvent, LoadEditorEvent, SaveEditorEvent,
    },
    start_goal::{spawn_or_update_start_goal, start_goal_drag},
};

pub mod help;
pub mod input;
pub mod planet;
pub mod point;
pub mod save;
pub mod start_goal;

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

#[derive(Component, Debug)]
struct EditingSceneRoot;

#[derive(Component)]
struct LevelEditingHandle(pub Handle<LevelData>);

#[derive(Resource, bevy::asset::Asset, bevy::reflect::TypePath)]
struct LevelEditingData(pub LevelData);

fn setup_editor(mut commands: Commands, asset_server: Res<AssetServer>) {
    let handle = asset_server.load::<LevelData>("levels/editing.level.ron");
    commands.spawn(LevelEditingHandle(handle));
    commands.spawn((
        EditingSceneRoot,
        SpatialBundle::default(),
        Name::new("EditingRoot"),
    ));
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

        // Save system
        app.add_event::<SaveEditorEvent>();
        app.add_event::<LoadEditorEvent>();
        app.add_event::<CleanupLoadEvent>();
        app.add_systems(Update, save_editor.run_if(in_editor));
        app.add_systems(Update, load_editor.run_if(in_editor));
        app.add_systems(Update, connect_parents.run_if(in_editor));
        app.add_systems(Update, resolve_holes.run_if(in_editor));
        app.add_systems(Update, fix_after_load.run_if(in_editor));
        app.add_systems(Update, cleanup_load.run_if(in_editor));
        app.register_type::<EPlanet>();
        app.register_type::<EPoint>();
        app.register_type::<IntMoveable>();
        app.register_type::<BorderedMesh>();
        app.register_type::<BorderMeshType>();
        app.register_type::<SpriteInfo>();
        app.register_type::<ScrollSprite>();
        app.register_type::<BorderedMatData>();
        app.register_type::<UIdMarker>();

        // Help system
        app.add_plugins(RonAssetPlugin::<EditorHelpConfig>::new(&[
            "editor_help.ron",
        ]));
        app.add_event::<HelpBarEvent>();
        app.add_systems(Startup, setup_editor_help_config);
        app.add_systems(Update, update_editor_help_config);
        app.add_systems(Update, setup_editor_help.run_if(entered_editor));
        app.add_systems(Update, update_editor_help_box.run_if(in_editor));
        app.add_systems(Update, update_editor_help_bar.run_if(in_editor));
        app.add_systems(Update, read_editor_help_output.run_if(in_editor));
        app.add_systems(Update, teardown_editor_help.run_if(left_editor));
        app.add_systems(Update, run_help_bar_command.run_if(in_editor));
        app.add_systems(Update, editor_help_input.run_if(in_editor));

        // Points
        app.add_systems(
            Update,
            (
                hover_points,
                spawn_points,
                select_points,
                point_select_shortcuts,
                delete_points,
                move_points,
                update_point_sprites,
            )
                .chain()
                .run_if(is_editing)
                .after(editor_help_input), // After this so that it can clear keyboard input if captured
        );

        // Planets
        app.add_systems(
            Update,
            (
                fix_dangling_mesh_ids,
                planet_state_input,
                redo_fields,
                resolve_pending_fields,
                nudge_fields,
                remove_field,
                handle_feral_points,
                make_new_field,
                update_field_gravity,
                drive_planet_meshes,
                draw_field_parents,
                debug_planets,
            )
                .chain()
                .run_if(is_editing)
                .after(move_points), // should be after the last thing in point chain
        );

        // Start n' goal
        app.add_systems(
            Update,
            (spawn_or_update_start_goal, start_goal_drag)
                .chain()
                .run_if(is_editing)
                .after(draw_field_parents),
        );
    }
}
