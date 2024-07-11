use crate::{
    camera::{camera_movement, CameraMode},
    environment::{
        background::{BgKind, BgManager},
        field::{FieldDrag, FieldStrength},
        rock::RockKind,
        segment::SegmentKind,
    },
    input::{watch_camera_input, SetCameraModeEvent},
    menu::paused::start_pause,
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
use field::{CreateStandaloneFieldEvent, EStandaloneField};
use save::{export_level, ExportLevelEvent};
use serde::{Deserialize, Serialize};

use self::{
    help::{
        destroy_editor_help, editor_help_input, read_editor_help_output, run_help_bar_command,
        setup_editor_help, setup_editor_help_config, update_editor_help_bar,
        update_editor_help_box, update_editor_help_config, EditorHelpConfig, HelpBarEvent,
    },
    planet::{
        change_planet_rock_kind, cleanup_degen_fields, draw_field_parents, drive_planet_meshes,
        handle_feral_points, make_new_field, nudge_fields, planet_state_input, redo_fields,
        remove_field, resolve_pending_fields, update_field_gravity, EPlanet,
    },
    point::{
        delete_points, hover_points, move_points, point_select_shortcuts, select_points,
        set_point_selection_order, spawn_points, update_point_sprites, EPoint,
    },
    replenish::{spawn_replenish, EReplenish},
    save::{
        connect_parents, fix_after_load, load_editor, save_editor, FuckySceneResource,
        LoadEditorEvent, SaveEditorEvent, SaveMarker,
    },
    segment::{create_segment, kill_segments, position_segments, SegmentParents},
    start_goal::{
        spawn_or_update_start_goal, start_goal_drag, EGoal, EStart, EStartGoalDiameter,
        EStartGoalDragOffset,
    },
    testing::{start_testing, stop_testing},
};

pub mod field;
pub mod help;
pub mod oneshots;
pub mod planet;
pub mod point;
pub mod replenish;
pub mod save;
pub mod segment;
pub mod start_goal;
pub mod testing;

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

#[derive(Component, Clone, Debug, PartialEq, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
struct EditingSceneRoot;

#[derive(Component)]
struct LevelEditingHandle(pub Handle<LevelData>);

#[derive(Resource, bevy::asset::Asset, bevy::reflect::TypePath)]
struct LevelEditingData(pub LevelData);

fn setup_editor(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut set_event: EventWriter<SetCameraModeEvent>,
    mut bg_manager: ResMut<BgManager>,
) {
    let handle = asset_server.load::<LevelData>("levels/editing/template.level.ron");
    commands.spawn((
        LevelEditingHandle(handle),
        Name::new("level_editing_handle"),
    ));
    commands.spawn((
        EditingSceneRoot,
        SpatialBundle::default(),
        Name::new("editing_root"),
    ));
    set_event.send(SetCameraModeEvent {
        mode: CameraMode::Free,
    });
    bg_manager.set_kind(BgKind::ParallaxStars(300));
}

fn destroy_editor(
    mut commands: Commands,
    handle: Query<Entity, With<LevelEditingHandle>>,
    root: Query<Entity, With<EditingSceneRoot>>,
) {
    if let Ok(id) = handle.get_single() {
        commands.entity(id).despawn_recursive();
    }
    for eid in root.iter() {
        commands.entity(eid).despawn_recursive();
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
        }
    }
}

pub struct OldEditorPlugin;

impl Plugin for OldEditorPlugin {
    fn build(&self, _app: &mut App) {
        //     // Meta-editor stuff
        //     app.insert_resource(LevelEditingData(LevelData::default()));
        //     app.add_plugins(RonAssetPlugin::<LevelData>::new(&["level.ron"]));
        //     app.add_systems(Update, setup_editor.run_if(entered_editor));
        //     app.add_systems(Update, destroy_editor.run_if(left_editor));
        //     app.add_systems(Update, watch_level_editing_asset.run_if(in_editor));

        //     // Save system
        //     app.insert_resource(FuckySceneResource::default());
        //     app.add_event::<SaveEditorEvent>();
        //     app.add_event::<LoadEditorEvent>();
        //     app.add_event::<ExportLevelEvent>();
        //     app.add_systems(Update, save_editor.run_if(in_editor));
        //     app.add_systems(Update, load_editor.run_if(in_editor));
        //     app.add_systems(Update, export_level.run_if(in_editor));
        //     app.add_systems(Update, connect_parents.run_if(in_editor));
        //     app.add_systems(Update, fix_after_load.run_if(in_editor));
        //     app.register_type::<EditingSceneRoot>();
        //     app.register_type::<EPlanet>();
        //     app.register_type::<EPoint>();
        //     app.register_type::<EStart>();
        //     app.register_type::<EGoal>();
        //     app.register_type::<EReplenish>();
        //     app.register_type::<SegmentKind>();
        //     app.register_type::<EStartGoalDragOffset>();
        //     app.register_type::<EStartGoalDiameter>();
        //     app.register_type::<RockKind>();
        //     app.register_type::<FieldStrength>();
        //     app.register_type::<FieldDrag>();
        //     app.register_type::<SegmentParents>();
        //     app.register_type::<IntMoveable>();
        //     app.register_type::<UIdMarker>();
        //     app.register_type::<SaveMarker>();
        //     app.register_type::<EStandaloneField>();

        //     // Help system
        //     app.add_plugins(RonAssetPlugin::<EditorHelpConfig>::new(&[
        //         "editor_help.ron",
        //     ]));
        //     app.add_event::<HelpBarEvent>();
        //     app.add_systems(Startup, setup_editor_help_config);
        //     app.add_systems(Update, update_editor_help_config);
        //     app.add_systems(Update, setup_editor_help.run_if(entered_editor));
        //     app.add_systems(Update, update_editor_help_box.run_if(in_editor));
        //     app.add_systems(Update, update_editor_help_bar.run_if(in_editor));
        //     app.add_systems(Update, read_editor_help_output.run_if(in_editor));
        //     app.add_systems(Update, destroy_editor_help.run_if(left_editor));
        //     app.add_systems(Update, run_help_bar_command.run_if(in_editor));
        //     app.add_systems(
        //         Update,
        //         editor_help_input
        //             .run_if(in_editor)
        //             .before(watch_camera_input)
        //             .before(start_pause),
        //     );

        //     // Oneshots
        //     oneshots::register_oneshots(app);

        //     // Fields
        //     app.add_event::<CreateStandaloneFieldEvent>();
        //     app.add_systems(
        //         Update,
        //         (
        //             field::create_new_standalone_field,
        //             field::update_standalone_fields,
        //         ),
        //     );

        //     // Points
        //     app.add_systems(
        //         Update,
        //         (
        //             hover_points,
        //             spawn_points,
        //             select_points,
        //             point_select_shortcuts,
        //             set_point_selection_order,
        //             delete_points,
        //             move_points,
        //             update_point_sprites,
        //         )
        //             .chain()
        //             .run_if(is_editing)
        //             .after(editor_help_input), // After this so that it can clear keyboard input if captured
        //     );

        //     // Planets
        //     app.add_systems(
        //         Update,
        //         (
        //             planet_state_input,
        //             redo_fields,
        //             resolve_pending_fields,
        //             nudge_fields,
        //             remove_field,
        //             cleanup_degen_fields,
        //             handle_feral_points,
        //             make_new_field,
        //             update_field_gravity,
        //             drive_planet_meshes,
        //             draw_field_parents,
        //             change_planet_rock_kind,
        //         )
        //             .chain()
        //             .run_if(is_editing)
        //             .after(move_points), // should be after the last thing in point chain
        //     );

        //     // Start n' goal
        //     app.add_systems(
        //         Update,
        //         (spawn_or_update_start_goal, start_goal_drag)
        //             .chain()
        //             .run_if(is_editing)
        //             .after(draw_field_parents),
        //     );

        //     // Segments
        //     app.add_systems(
        //         Update,
        //         (create_segment, kill_segments, position_segments).run_if(is_editing),
        //     );

        //     // Replenish
        //     app.add_systems(
        //         Update,
        //         spawn_replenish.run_if(is_editing).after(editor_help_input),
        //     );

        //     // Testing
        //     app.add_systems(Update, start_testing.run_if(entered_testing));
        //     app.add_systems(
        //         Update,
        //         stop_testing.run_if(left_testing).after(camera_movement),
        //     );
    }
}
