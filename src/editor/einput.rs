use bevy::prelude::*;

use crate::{
    input::MouseState,
    meta::game_state::{EditingMode, GameState},
};

use super::{
    eoneshots::EOneshots,
    epoint::{EPoint, ESelected},
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
    if let EditingMode::Free = emode {
        if keyboard.just_pressed(KeyCode::KeyP) {
            commands.run_system(oneshots.spawn_rock);
            return;
        }
        if keyboard.just_pressed(KeyCode::KeyF) {
            commands.run_system(oneshots.spawn_field);
            return;
        }
        if keyboard.just_pressed(KeyCode::KeyR) {
            commands.run_system_with_input(oneshots.spawn_replenish, mouse.world_pos);
            return;
        }
        if keyboard.just_pressed(KeyCode::BracketLeft) {
            commands.run_system_with_input(oneshots.spawn_start, mouse.world_pos);
            return;
        }
        if keyboard.just_pressed(KeyCode::BracketRight) {
            commands.run_system_with_input(oneshots.spawn_goal, mouse.world_pos);
            return;
        }
        if keyboard.just_pressed(KeyCode::KeyL) {
            commands.run_system(oneshots.spawn_live_poly);
            return;
        }
    }
}
