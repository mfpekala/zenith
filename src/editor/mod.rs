use bevy::prelude::*;

use crate::{
    input::{watch_camera_input, watch_mouse},
    meta::game_state::{entered_editor, left_editor},
};
use transitions::{in_editing, in_testing, ERootEid, HRootEid, TRootEid};

pub(self) mod efield;
pub(self) mod egoal;
mod einput;
pub(self) mod epoint;
pub(self) mod ereplenish;
pub(self) mod erock;
pub(self) mod estart;
mod help;
mod eoneshots;
mod transitions;

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        // EField
        app.register_type::<efield::EField>();
        app.add_systems(
            Update,
            (efield::update_fields, efield::animate_fields)
                .chain()
                .after(epoint::cleanup_points),
        );

        // EInput
        app.add_systems(
            Update,
            einput::watch_dramatic_editing_input
                .after(watch_mouse)
                .run_if(in_editing),
        );

        // EPoint
        app.register_type::<epoint::EPoint>();
        app.register_type::<epoint::EPointGroup>();
        app.register_type::<epoint::EHovered>();
        app.register_type::<epoint::ESelected>();
        app.add_systems(
            Update,
            (
                epoint::hover_points,
                epoint::select_points,
                epoint::move_points,
                epoint::animate_points,
                epoint::cleanup_points,
                epoint::tag_shiny,
                epoint::update_shiny_thing,
            )
                .chain()
                .after(watch_mouse)
                .run_if(in_editing),
        );

        // ERock
        app.register_type::<erock::ERock>();
        app.add_systems(
            Update,
            (erock::update_rocks, erock::animate_rocks)
                .chain()
                .after(epoint::cleanup_points),
        );

        // Help
        app.register_type::<help::HelpBarData>();
        app.add_systems(Update, help::update_editor_texts);
        app.add_systems(Update, help::editor_help_input.before(watch_camera_input));
        app.add_systems(Update, help::update_editor_help_bar);
        app.add_systems(Update, help::update_help_box);

        // Oneshots
        eoneshots::register_oneshots(app);

        // Transitions
        app.insert_resource(ERootEid(Entity::PLACEHOLDER));
        app.insert_resource(TRootEid(Entity::PLACEHOLDER));
        app.insert_resource(HRootEid(Entity::PLACEHOLDER));
        app.add_systems(Update, transitions::setup_editor.run_if(entered_editor));
        app.add_systems(Update, transitions::destroy_editor.run_if(left_editor));
        app.add_systems(
            Update,
            transitions::setup_editing.run_if(transitions::entered_editing),
        );
        app.add_systems(
            Update,
            transitions::destroy_editing.run_if(transitions::left_editing),
        );
        app.add_systems(
            Update,
            transitions::setup_testing.run_if(transitions::entered_testing),
        );
        app.add_systems(
            Update,
            transitions::destroy_testing.run_if(transitions::left_testing),
        );
    }
}
