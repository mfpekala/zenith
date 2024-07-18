use bevy::prelude::*;

use crate::{
    input::MouseState,
    meta::game_state::{EditingMode, GameState},
};

use super::{
    epoint::{EPoint, ESelected},
    oneshots::EOneshots,
};

/// Watches for "dramatic" editing input which will create or delete things
/// Inputs that do movement/adjustment are handled within individual update functions
pub(super) fn watch_dramatic_editing_input(
    mut commands: Commands,
    gs: Res<GameState>,
    oneshots: Res<EOneshots>,
    mouse: Res<MouseState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    selected_points: Query<Entity, (With<EPoint>, With<ESelected>)>,
) {
    // NOTE: We return after doing any "real work" to maintain the nice-to-have
    // invariant that only one dramatic thing happens per frame
    let Some(emode) = gs.get_editing_mode() else {
        warn!("In order to watch editor input, we must have an editing mode. Gs: {gs:?}");
        return;
    };
    if mouse.button_input.just_pressed(MouseButton::Right) {
        commands.run_system_with_input(oneshots.spawn_point, (emode, mouse.world_pos));
        return;
    }
    if keyboard.just_pressed(KeyCode::Backspace) {
        let eids = selected_points.iter().collect();
        commands.run_system_with_input(oneshots.delete_points, eids);
        return;
    }
    if keyboard.just_pressed(KeyCode::KeyP) {
        if let EditingMode::Free = emode {
            commands.run_system(oneshots.spawn_rock);
            return;
        }
    }
    if keyboard.just_pressed(KeyCode::KeyF) {
        if let EditingMode::Free = emode {
            commands.run_system(oneshots.spawn_field);
            return;
        }
    }
}
