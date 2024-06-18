use crate::{
    drawing::effects::{ScreenEffect, ScreenEffectManager},
    meta::{
        game_state::{GameState, PrevGameState},
        level_data::{LevelData, LevelDataOneshots, LevelRoot},
    },
};
use bevy::prelude::*;

#[derive(Component)]
pub(super) struct ActivelyLoading(Handle<LevelData>);

pub(super) fn did_level_change(old_state: Res<PrevGameState>, new_state: Res<GameState>) -> bool {
    let Some(current_level) = new_state.get_level_state() else {
        return false;
    };
    match old_state.meta.get_level_state() {
        Some(old_level) => old_level.id != current_level.id,
        None => true,
    }
}

pub(super) fn is_actively_loading_level(q: Query<&ActivelyLoading>) -> bool {
    q.iter().len() > 0
}

pub(super) fn start_load(
    mut commands: Commands,
    gs: Res<GameState>,
    asset_server: Res<AssetServer>,
) {
    let level_id = gs.get_level_state().unwrap().id;
    let handle = asset_server.load(format!("levels/{level_id}.level.ron"));
    commands.spawn((
        Name::new(format!("active_level_load_{level_id}")),
        ActivelyLoading(handle),
    ));
}

pub(super) fn actively_load(
    level_oneshots: Res<LevelDataOneshots>,
    actively_loadings: Query<(Entity, &ActivelyLoading)>,
    level_ass: Res<Assets<LevelData>>,
    level_roots: Query<Entity, With<LevelRoot>>,
    mut commands: Commands,
    mut screen_effects: ResMut<ScreenEffectManager>,
) {
    let spawn_id = level_oneshots.spawn_level_id.clone();
    let mut kill_eids = vec![];
    let mut level_data = None;
    for (eid, al) in actively_loadings.iter() {
        if let Some(data) = level_ass.get(&al.0) {
            kill_eids.push(eid);
            level_data = Some(data.clone());
        }
    }
    if level_data.is_some() {
        for root_eid in level_roots.iter() {
            kill_eids.push(root_eid);
        }
    }
    for kill_eid in kill_eids {
        commands.entity(kill_eid).despawn_recursive();
    }
    if let Some(level_data) = level_data {
        commands.run_system_with_input(spawn_id, (1, level_data, IVec2::ZERO));
        screen_effects.queue_effect(ScreenEffect::UnfadeToBlack);
    }
}

pub fn destroy_level(level_roots: Query<Entity, With<LevelRoot>>, mut commands: Commands) {
    for eid in level_roots.iter() {
        commands.entity(eid).despawn_recursive();
    }
}
