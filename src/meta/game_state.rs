use bevy::prelude::*;

#[derive(Clone, Copy, Debug)]
pub enum MenuState {
    Title,
    ConstellationSelect,
    GalaxyOverworld,
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
    pub last_safe_location: IVec2,
    pub num_shots: i32,
}
impl LevelState {
    pub fn fresh_from_id(id: String) -> Self {
        Self {
            id,
            next_id: None,
            is_settled: false,
            is_won: false,
            last_safe_location: IVec2::ZERO,
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

    pub fn into_prev(self) -> PrevGameState {
        PrevGameState { meta: self.meta }
    }
}

#[derive(Resource, Clone, Debug)]
pub struct PrevGameState {
    pub meta: MetaState,
}
impl PrevGameState {
    pub fn into_game_state(self) -> GameState {
        GameState { meta: self.meta }
    }
}

#[derive(Event)]
pub struct SetGameState(pub GameState);

fn translate_events(
    mut state_change: EventReader<SetGameState>,
    mut prev_gs: ResMut<PrevGameState>,
    mut gs: ResMut<GameState>,
) {
    *prev_gs = gs.clone().into_prev();
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
        last_safe_location: IVec2::ZERO,
        num_shots: 0,
    });
    app.insert_resource(GameState {
        meta: initial_state.clone(),
    });
    app.insert_resource(PrevGameState {
        meta: initial_state,
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
            old_state: Res<crate::meta::game_state::PrevGameState>,
            new_state: Res<crate::meta::game_state::GameState>,
        ) -> bool {
            let old_as_gs = old_state.clone().into_game_state();
            !$ref_fn(&old_as_gs) && $ref_fn(&new_state)
        }
    };
}

#[macro_export]
macro_rules! when_becomes_false {
    ($ref_fn: ident, $fname: ident) => {
        pub fn $fname(
            old_state: Res<crate::meta::game_state::PrevGameState>,
            new_state: Res<crate::meta::game_state::GameState>,
        ) -> bool {
            let old_as_gs = old_state.clone().into_game_state();
            $ref_fn(&old_as_gs) && !$ref_fn(&new_state)
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
