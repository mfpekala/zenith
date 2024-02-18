use crate::meta::{game_state::in_editor, level::get_level_folder};
use bevy::prelude::*;
use std::{fs::File, io::Write};

fn watch_for_save(keys: Res<Input<KeyCode>>) {
    if !keys.pressed(KeyCode::SuperLeft) || !keys.pressed(KeyCode::S) {
        // Don't save
        return;
    }
    // For now just test
    let mut fout = File::create(get_level_folder().join("editing.zenith")).unwrap();
    write!(fout, "mark in the doc").unwrap();
}

pub fn register_saver(app: &mut App) {
    app.add_systems(Update, watch_for_save.run_if(in_editor));
}
