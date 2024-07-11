use bevy::prelude::*;

use crate::meta::game_state::{entered_editor, left_editor};
use transitions::{in_editing, in_testing};

mod oneshots;
mod transitions;

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        // Oneshots
        oneshots::register_oneshots(app);

        // Transitions
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
