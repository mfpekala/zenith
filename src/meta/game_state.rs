use bevy::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct MenuState;

#[derive(Clone, Copy, Debug)]
pub enum EditingMode {
    Free,
    CreatingRock(Entity),
    EditingRock(Entity),
}

#[derive(Clone, Copy, Debug)]
pub struct EditingState {
    pub mode: EditingMode,
    pub paused: bool,
}
impl EditingState {
    pub fn to_game_state(&self) -> GameState {
        GameState {
            meta: MetaState::Editor(EditorState::Editing(self.clone())),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum EditorState {
    Editing(EditingState),
    Testing,
}

#[derive(Clone, Copy, Debug)]
pub struct LevelState;

#[derive(Clone, Copy, Debug)]
pub enum MetaState {
    Menu(MenuState),
    Editor(EditorState),
    Level(LevelState),
}

#[derive(Resource, Clone, Debug)]
pub struct GameState {
    pub meta: MetaState,
}
impl GameState {
    pub fn is_in_menu(&self) -> bool {
        match self.meta {
            MetaState::Menu(_) => true,
            _ => false,
        }
    }

    pub fn is_in_editor(&self) -> bool {
        match self.meta {
            MetaState::Editor(_) => true,
            _ => false,
        }
    }

    pub fn is_in_level(&self) -> bool {
        match self.meta {
            MetaState::Level(_) => true,
            _ => false,
        }
    }
}

#[derive(Event)]
pub struct SetGameState(pub GameState);

fn translate_events(mut state_change: EventReader<SetGameState>, mut gs: ResMut<GameState>) {
    let Some(SetGameState(new_state)) = state_change.read().last() else {
        return;
    };
    println!("Setting state to {:?}", new_state.clone());
    *gs = new_state.clone();
}

fn set_initial_game_state(mut gs_writer: EventWriter<SetGameState>) {
    gs_writer.send(SetGameState(GameState {
        meta: MetaState::Menu(MenuState),
    }));
}

pub fn register_game_state(app: &mut App) {
    app.insert_resource(GameState {
        meta: MetaState::Level(LevelState),
    });
    app.add_event::<SetGameState>();
    app.add_systems(Startup, set_initial_game_state);
    app.add_systems(PostUpdate, translate_events);
}

// Helper functions for common state transitions

#[macro_export]
macro_rules! when_becomes_true {
    ($ref_fn: ident, $fname: ident) => {
        pub fn $fname(
            mut state_change: EventReader<crate::meta::game_state::SetGameState>,
            old_state: Res<GameState>,
        ) -> bool {
            match state_change.read().last() {
                Some(crate::meta::game_state::SetGameState(state)) => {
                    if $ref_fn(&old_state) {
                        // It's already true
                        return false;
                    }
                    $ref_fn(state)
                }
                None => false,
            }
        }
    };
}

#[macro_export]
macro_rules! when_becomes_false {
    ($ref_fn: ident, $fname: ident) => {
        pub fn $fname(
            mut state_change: EventReader<crate::meta::game_state::SetGameState>,
            old_state: Res<GameState>,
        ) -> bool {
            match state_change.read().last() {
                Some(crate::meta::game_state::SetGameState(state)) => {
                    if !$ref_fn(&old_state) {
                        // It's already false
                        return false;
                    }
                    !$ref_fn(state)
                }
                None => false,
            }
        }
    };
}

fn menu_transition_helper(gs: &GameState) -> bool {
    gs.is_in_menu()
}

fn editor_transition_helper(gs: &GameState) -> bool {
    gs.is_in_editor()
}

fn level_transition_helper(gs: &GameState) -> bool {
    gs.is_in_level()
}

when_becomes_true!(menu_transition_helper, entered_menu);
when_becomes_false!(menu_transition_helper, left_menu);
when_becomes_true!(editor_transition_helper, entered_editor);
when_becomes_false!(editor_transition_helper, left_editor);
when_becomes_true!(level_transition_helper, entered_level);
when_becomes_false!(level_transition_helper, left_level);

pub fn in_menu(gs: Res<GameState>) -> bool {
    gs.is_in_menu()
}

pub fn in_editor(gs: Res<GameState>) -> bool {
    gs.is_in_editor()
}

pub fn in_level(gs: Res<GameState>) -> bool {
    gs.is_in_level()
}
