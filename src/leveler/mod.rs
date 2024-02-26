use crate::{
    environment::rock::RockResources,
    meta::{
        game_state::{
            entered_level, pretranslate_events, GameState, MetaState, NextGameState, SetGameState,
        },
        level_data::{get_level_folder, LevelData},
    },
    when_becomes_true,
};
use bevy::prelude::*;

fn is_level_won_helper(gs: &GameState) -> bool {
    match &gs.meta {
        MetaState::Level(level_state) => level_state.is_won,
        _ => false,
    }
}

pub fn is_level_won(gs: Res<GameState>) -> bool {
    is_level_won_helper(&gs)
}
pub fn is_level_not_won(gs: Res<GameState>) -> bool {
    !is_level_won_helper(&gs)
}
when_becomes_true!(is_level_won_helper, entered_won_level);

pub fn setup_level(
    mut commands: Commands,
    gs: Res<NextGameState>,
    mut gs_writer: EventWriter<SetGameState>,
    rock_res: Res<RockResources>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let MetaState::Level(level_state) = &gs.meta else {
        error!("Setuping level but the next game state doesn't seem to be for level...");
        return;
    };
    let level_id = level_state.id.clone();
    let level_data =
        LevelData::load(get_level_folder().join(format!("{}.zenith", level_id))).unwrap();
    level_data.load_level(&mut commands, &rock_res.feature_map, &mut meshes);
    let mut next_level_state = level_state.clone();
    next_level_state.last_safe_location = level_data.starting_point;
    gs_writer.send(SetGameState(GameState {
        meta: MetaState::Level(next_level_state),
    }));
}

pub fn register_leveler(app: &mut App) {
    app.add_systems(
        Update,
        setup_level.run_if(entered_level).after(pretranslate_events),
    );
}
