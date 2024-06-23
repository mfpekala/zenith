use bevy::prelude::*;

use super::progress::GalaxyKind;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MenuState {
    Title,
    ConstellationSelect,
    GalaxyOverworld,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EditingMode {
    Free,
    CreatingPlanet(Entity),
    EditingPlanet(Entity),
}
impl EditingMode {
    pub fn to_meta_state(&self) -> MetaState {
        MetaState::Editor(EditorState::Editing(EditingState { mode: *self }))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EditingState {
    pub mode: EditingMode,
}
impl EditingState {
    pub fn blank() -> Self {
        Self {
            mode: EditingMode::Free,
        }
    }

    pub fn to_meta_state(&self) -> MetaState {
        MetaState::Editor(EditorState::Editing(self.clone()))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EditorState {
    Editing(EditingState),
    Testing,
}
impl EditorState {
    pub fn get_editing_mode(&self) -> Option<EditingMode> {
        match *self {
            Self::Editing(state) => match state.mode {
                EditingMode::Free => Some(EditingMode::Free),
                EditingMode::CreatingPlanet(id) => Some(EditingMode::CreatingPlanet(id)),
                EditingMode::EditingPlanet(id) => Some(EditingMode::EditingPlanet(id)),
            },
            _ => None,
        }
    }

    pub fn to_meta_state(&self) -> MetaState {
        MetaState::Editor(self.clone())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LevelState {
    pub kind: GalaxyKind,
    pub id: String,
    pub is_won: bool,
    pub num_shots: i32,
}
impl LevelState {
    pub fn from_galaxy_n_level(kind: GalaxyKind, level_id: String) -> Self {
        Self {
            kind,
            id: level_id,
            is_won: false,
            num_shots: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum MetaState {
    Menu(MenuState),
    Editor(EditorState),
    Level(LevelState),
}
impl MetaState {
    pub fn get_level_state(&self) -> Option<LevelState> {
        match &self {
            MetaState::Level(state) => Some(state.clone()),
            _ => None,
        }
    }
}

/// What kind of pause are we in. Contains kind of duplicate data as meta,
/// but probably fine because it gets implementational simplicity
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PauseState {
    Level,
    Editor,
    Settings { prev_level: bool, prev_menu: bool },
}

#[derive(Resource, Clone, Debug, PartialEq)]
pub struct GameState {
    pub meta: MetaState,
    pub pause: Option<PauseState>,
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
        self.meta.get_level_state()
    }

    pub fn into_prev(self) -> PrevGameState {
        PrevGameState {
            meta: self.meta,
            pause: self.pause,
        }
    }

    pub fn get_editing_mode(&self) -> Option<EditingMode> {
        match self.meta {
            MetaState::Editor(state) => state.get_editing_mode(),
            _ => None,
        }
    }
}

#[derive(Resource, Clone, Debug)]
pub struct PrevGameState {
    pub meta: MetaState,
    pub pause: Option<PauseState>,
}
impl PrevGameState {
    pub fn into_game_state(self) -> GameState {
        GameState {
            meta: self.meta,
            pause: self.pause,
        }
    }
}

#[derive(Event)]
pub struct SetMetaState(pub MetaState);

#[derive(Event)]
pub struct SetPaused(pub Option<PauseState>);

fn translate_events(
    mut state_change: EventReader<SetMetaState>,
    mut paused_change: EventReader<SetPaused>,
    mut prev_gs: ResMut<PrevGameState>,
    mut gs: ResMut<GameState>,
) {
    *prev_gs = gs.clone().into_prev();
    if let Some(SetMetaState(new_state)) = state_change.read().last() {
        gs.meta = new_state.clone();
    };
    if let Some(SetPaused(new_paused)) = paused_change.read().last() {
        gs.pause = *new_paused;
    };
}

fn set_initial_game_state(mut gs_writer: EventWriter<SetMetaState>) {
    gs_writer.send(SetMetaState(MetaState::Menu(MenuState::Title)));

    // gs_writer.send(SetMetaState(MetaState::Level(
    //     LevelState::from_galaxy_n_level(GalaxyKind::Basic, "basic_1".into()),
    // )));
}

pub fn register_game_state(app: &mut App) {
    let initial_state = MetaState::Level(LevelState {
        id: "".to_string(),
        kind: GalaxyKind::default(),
        is_won: false,
        num_shots: 0,
    });
    app.insert_resource(GameState {
        meta: initial_state.clone(),
        pause: None,
    });
    app.insert_resource(PrevGameState {
        meta: initial_state,
        pause: None,
    });
    app.add_event::<SetMetaState>();
    app.add_event::<SetPaused>();
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
