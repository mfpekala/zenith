use bevy::prelude::*;

#[derive(Clone, Debug)]
pub struct MenuState;

#[derive(Clone, Debug)]
pub struct EditorState {
    pub is_testing: bool,
}

#[derive(Clone, Debug)]
pub struct LevelState;

/// NOTE: This is just all possible states we care about, will layer in a stack as needed
/// In other words, the stack structure that will likely exist in practice is not reflected
/// in this (flat) enum
#[derive(Clone, Debug)]
pub enum MetaState {
    Menu(MenuState),
    Editor(EditorState),
    Level(LevelState),
}

#[derive(Resource, Clone, Debug)]
pub struct GameState {
    pub meta: MetaState,
}

#[derive(Event)]
pub struct SetGameState(GameState);

#[macro_export]
macro_rules! when_becomes_true {
    ($ref_fn: ident, $fname: ident) => {
        fn $fname(mut state_change: EventReader<SetGameState>) -> bool {
            match state_change.read().last() {
                Some(SetGameState(state)) => $ref_fn(state),
                None => false,
            }
        }
    };
}

#[macro_export]
macro_rules! when_becomes_false {
    ($ref_fn: ident, $fname: ident) => {
        fn $fname(mut state_change: EventReader<SetGameState>) -> bool {
            match state_change.read().last() {
                Some(SetGameState(state)) => !$ref_fn(state),
                None => false,
            }
        }
    };
}

pub fn translate_events(mut state_change: EventReader<SetGameState>, mut gs: ResMut<GameState>) {
    let Some(SetGameState(new_state)) = state_change.read().last() else {
        return;
    };
    *gs = new_state.clone();
}

pub fn register_game_state(app: &mut App) {
    app.insert_resource(GameState {
        meta: MetaState::Menu(MenuState),
    });
    app.add_event::<SetGameState>();
    app.add_systems(Update, translate_events);
}
