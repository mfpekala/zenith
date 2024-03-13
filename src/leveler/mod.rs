use crate::{
    environment::{
        field::Field as eField,
        goal::{Goal, GoalGet},
        rock::{Rock, RockResources},
        starting_point::StartingPoint,
    },
    meta::{
        game_state::{entered_level, GameState, LevelState, MetaState, SetGameState},
        level_data::{get_level_folder, LevelData},
    },
    ship::{Ship, SpawnShipId},
    when_becomes_true,
};
use bevy::{ecs::system::SystemId, prelude::*};

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

fn setup_helper(
    level_id: String,
    commands: &mut Commands,
    rock_res: &Res<RockResources>,
    meshes: &mut ResMut<Assets<Mesh>>,
    gs_writer: &mut EventWriter<SetGameState>,
    spawn_ship_id: SystemId<(IVec2, f32)>,
) {
    let level_data =
        LevelData::load(get_level_folder().join(format!("{}.zenith", level_id))).unwrap();
    level_data.load_level(commands, meshes, rock_res, spawn_ship_id);
    let ipos = IVec2 {
        x: level_data.starting_point.x as i32,
        y: level_data.starting_point.x as i32,
    };
    let next_level_state = LevelState {
        id: level_id.clone(),
        next_id: level_data.next_level.clone(),
        is_settled: false,
        is_won: false,
        last_safe_location: ipos,
        num_shots: 0,
    };
    gs_writer.send(SetGameState(GameState {
        meta: MetaState::Level(next_level_state),
    }));
}

pub fn setup_level(
    mut commands: Commands,
    gs: Res<GameState>,
    mut gs_writer: EventWriter<SetGameState>,
    rock_res: Res<RockResources>,
    mut meshes: ResMut<Assets<Mesh>>,
    spawn_ship_id: Res<SpawnShipId>,
) {
    let MetaState::Level(level_state) = &gs.meta else {
        error!("Setuping level but the next game state doesn't seem to be for level...");
        return;
    };
    let level_id = level_state.id.clone();
    setup_helper(
        level_id,
        &mut commands,
        &rock_res,
        &mut meshes,
        &mut gs_writer,
        spawn_ship_id.0.clone(),
    );
}

pub fn progress_level(
    mut gs_reader: EventReader<GoalGet>,
    mut commands: Commands,
    gs: Res<GameState>,
    rocks: Query<Entity, With<Rock>>,
    ships: Query<Entity, With<Ship>>,
    fields: Query<Entity, With<eField>>,
    start: Query<Entity, With<StartingPoint>>,
    goal: Query<Entity, With<Goal>>,
    mut gs_writer: EventWriter<SetGameState>,
    rock_res: Res<RockResources>,
    mut meshes: ResMut<Assets<Mesh>>,
    spawn_ship_id: Res<SpawnShipId>,
) {
    let Some(_) = gs_reader.read().last() else {
        return;
    };
    // Despawn everything
    for id in rocks
        .iter()
        .chain(ships.iter())
        .chain(fields.iter())
        .chain(start.iter())
        .chain(goal.iter())
    {
        commands.entity(id).despawn_recursive();
    }
    let Some(level_state) = gs.get_level_state() else {
        panic!("Bad level data loading on progress");
    };
    let Some(next_id) = level_state.next_id else {
        panic!("Out of levels :/");
    };
    setup_helper(
        next_id,
        &mut commands,
        &rock_res,
        &mut meshes,
        &mut gs_writer,
        spawn_ship_id.0.clone(),
    );
}

pub fn register_leveler(app: &mut App) {
    app.add_systems(Update, setup_level.run_if(entered_level));
    app.add_systems(Update, progress_level);
}
