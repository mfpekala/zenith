use bevy::prelude::*;

use crate::{
    input::watch_mouse,
    meta::game_state::{entered_editor, left_editor},
};
use transitions::{in_editing, in_testing, ERootEid, HRootEid, TRootEid};

pub mod epoint;
mod help;
mod input;
mod oneshots;
mod transitions;

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        // Epoint
        app.add_systems(
            Update,
            (
                epoint::hover_points,
                epoint::select_points,
                epoint::move_points,
                epoint::animate_points,
            )
                .chain()
                .after(watch_mouse)
                .run_if(in_editing),
        );

        // Help
        app.add_systems(Update, help::editor_help_input);
        app.add_systems(Update, help::update_editor_help_bar);

        // Inpu
        app.add_systems(
            Update,
            input::watch_dramatic_editing_input
                .after(watch_mouse)
                .run_if(in_editing),
        );

        // Oneshots
        oneshots::register_oneshots(app);

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
