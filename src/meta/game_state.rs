use bevy::prelude::*;

#[derive(Clone, Copy, Debug)]
pub enum MenuState {
    Title,
    SaveFile,
    ConstellationSelect,
}

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

#[derive(Clone, Debug)]
pub struct LevelState {
    pub id: String,
    pub next_id: Option<String>,
    pub is_settled: bool,
    pub is_won: bool,
    pub last_safe_location: Vec2,
    pub num_shots: i32,
}
impl LevelState {
    pub fn fresh_from_id(id: String) -> Self {
        Self {
            id,
            next_id: None,
            is_settled: false,
            is_won: false,
            last_safe_location: Vec2::ZERO,
            num_shots: 0,
        }
    }
}

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

    pub fn get_level_state(&self) -> Option<LevelState> {
        match &self.meta {
            MetaState::Level(state) => Some(state.clone()),
            _ => None,
        }
    }

    pub fn into_next(self) -> NextGameState {
        NextGameState { meta: self.meta }
    }
}

#[derive(Resource, Clone, Debug)]
pub struct NextGameState {
    pub meta: MetaState,
}
impl NextGameState {
    pub fn into_game_state(self) -> GameState {
        GameState { meta: self.meta }
    }
}

#[derive(Event)]
pub struct SetGameState(pub GameState);

pub fn pretranslate_events(
    mut state_change: EventReader<SetGameState>,
    mut next_gs: ResMut<NextGameState>,
) {
    let Some(SetGameState(new_state)) = state_change.read().last() else {
        return;
    };
    *next_gs = new_state.clone().into_next();
}

fn translate_events(mut state_change: EventReader<SetGameState>, mut gs: ResMut<GameState>) {
    let Some(SetGameState(new_state)) = state_change.read().last() else {
        return;
    };
    *gs = new_state.clone();
}

fn set_initial_game_state(mut gs_writer: EventWriter<SetGameState>) {
    gs_writer.send(SetGameState(GameState {
        meta: MetaState::Menu(MenuState::Title),
    }));
}

pub fn register_game_state(app: &mut App) {
    let initial_state = MetaState::Level(LevelState {
        id: "".to_string(),
        next_id: None,
        is_settled: false,
        is_won: false,
        last_safe_location: Vec2::ZERO,
        num_shots: 0,
    });
    app.insert_resource(GameState {
        meta: initial_state.clone(),
    });
    app.insert_resource(NextGameState {
        meta: initial_state,
    });
    app.add_event::<SetGameState>();
    app.add_systems(Startup, set_initial_game_state);
    app.add_systems(Update, pretranslate_events);
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
