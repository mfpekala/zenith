use bevy::{prelude::*, utils::HashMap};

#[derive(Debug, Clone)]
pub struct LevelMetaData {
    pub id: String,
    pub title: String,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GalaxyKind {
    #[default]
    Basic,
    Springy,
}
impl std::fmt::Display for GalaxyKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone)]
pub struct GalaxyMetaData {
    pub title: String,
    pub description: String,
    pub levels: Vec<LevelMetaData>,
}

// TODO: Yeah so this ends up just being stored on code segment.
// probably not terrible but probably should shove into a file at some point
impl GalaxyKind {
    pub fn all() -> Vec<Self> {
        vec![Self::Basic, Self::Springy]
    }

    pub fn rank(&self) -> u32 {
        match self {
            Self::Basic => 0,
            Self::Springy => 1,
        }
    }

    pub fn next(&self) -> Option<Self> {
        match self {
            Self::Basic => Some(Self::Springy),
            Self::Springy => None,
        }
    }

    pub fn prev(&self) -> Option<Self> {
        match self {
            Self::Basic => None,
            Self::Springy => Some(Self::Basic),
        }
    }

    pub fn to_levels(&self) -> Vec<LevelMetaData> {
        match self {
            Self::Basic => vec![
                LevelMetaData {
                    id: "basic_1".to_string(),
                    title: "First level".to_string(),
                    description: "Just testing 1".to_string(),
                },
                LevelMetaData {
                    id: "basic_2".to_string(),
                    title: "Second level".to_string(),
                    description: "Just testing 2".to_string(),
                },
            ],
            Self::Springy => vec![
                LevelMetaData {
                    id: "springy_1".to_string(),
                    title: "Spring intro".to_string(),
                    description: "Introducing the player to springs".to_string(),
                },
                LevelMetaData {
                    id: "springy_2".to_string(),
                    title: "Springs go brrr".to_string(),
                    description: "Yeah, so, springs".to_string(),
                },
            ],
        }
    }

    pub fn get_next_level_id(&self, level_id: &str) -> Option<String> {
        let mut it = self.to_levels().into_iter().map(|meta| meta.id);
        while let Some(val) = it.next() {
            if val.as_str() == level_id {
                break;
            }
        }
        it.next()
    }

    pub fn to_meta_data(&self) -> GalaxyMetaData {
        let (title, description) = match self {
            Self::Basic => ("Basic", "A basic, test galaxy"),
            Self::Springy => ("Spring", "For learning about springs"),
        };
        GalaxyMetaData {
            title: title.to_string(),
            description: description.to_string(),
            levels: self.to_levels(),
        }
    }
}

/// A handy struct for passing around info about whether a galaxy is completed
#[derive(
    serde::Serialize,
    serde::Deserialize,
    bevy::asset::Asset,
    Debug,
    PartialEq,
    Clone,
    Resource,
    Default,
    Reflect,
)]
pub struct GalaxyProgress {
    /// Has the player ever completed this galaxy?
    pub completed: bool,
    /// In the current epoch of the user playing this galaxy, what is the next level they should play?
    /// I.e., initially Some(<first level of galaxy>), and after completing becomes None
    pub next_level: Option<String>,
}
impl GalaxyProgress {
    /// In the players current epoch, returns (num_complete, num_in_galaxy)
    pub fn portion_completed(&self, kind: GalaxyKind) -> (u32, u32) {
        let Some(next_level) = self.next_level.as_ref() else {
            let as_u32 = kind.to_levels().len() as u32;
            return (as_u32, as_u32);
        };
        let mut rank = 0;
        for (ix, level) in kind.to_levels().into_iter().enumerate() {
            if &level.id == next_level {
                rank = ix as u32;
                break;
            }
        }
        (rank, kind.to_levels().len() as u32)
    }

    /// Attempts to mark a level as complete. The level must match `next_level` and exist as expected
    pub fn try_mark_completed(&mut self, kind: GalaxyKind, level_id: String) -> bool {
        let Some(old_next_level) = self.next_level.clone() else {
            warn!(
                "Tried to mark completed with no next_level in galaxy {}",
                kind
            );
            return false;
        };
        if old_next_level != level_id {
            warn!(
                "Tried to mark level {} as completed, when next_level says {} in galaxy {}",
                level_id, old_next_level, kind
            );
            return false;
        }
        let next = kind.get_next_level_id(&level_id);
        self.completed = self.completed || next.is_none();
        self.next_level = next;
        true
    }
}

/// Maps galaxy (enum as string) to (completed, id_of_level_on)
#[derive(
    Component,
    serde::Serialize,
    serde::Deserialize,
    bevy::asset::Asset,
    Debug,
    PartialEq,
    Clone,
    Resource,
    Default,
    Reflect,
)]
pub struct GameProgress {
    galaxy_map: HashMap<String, GalaxyProgress>,
}
impl GameProgress {
    /// Gets the (completed, active_level) status for a given galaxy
    pub fn get_galaxy_progress(&self, kind: GalaxyKind) -> GalaxyProgress {
        self.galaxy_map.get(&kind.to_string()).unwrap().clone()
    }

    /// Returns the earliest incomplete galaxy, i.e., the galaxy the player is actively playing
    pub fn first_incomplete_galaxy(&self) -> GalaxyKind {
        for kind in GalaxyKind::all() {
            let progress = self.get_galaxy_progress(kind);
            if !progress.completed {
                return kind;
            }
        }
        GalaxyKind::Basic
    }

    /// Returns true if this galaxy is playable, a.k.a if it should be selectable from the
    /// galaxy overworld. Translates to: complete OR first incomplete
    pub fn is_playable(&self, kind: GalaxyKind) -> bool {
        let progress = self.get_galaxy_progress(kind);
        progress.completed || kind == self.first_incomplete_galaxy()
    }
}

/// Marks a GameProgress as being active.
/// Basically, the game will load both save files, but only one of them will be given
/// this component. So whenever it needs to be queried, just do Query<&GameProgress, With<ActiveSaveFile>>
#[derive(Component)]
pub struct ActiveSaveFile;

#[derive(Component)]
pub(super) struct TempGameProgressLoading {
    a: Handle<GameProgress>,
    b: Handle<GameProgress>,
}

pub(super) fn initialize_game_progress(asset_server: Res<AssetServer>, mut commands: Commands) {
    let a_handle: Handle<GameProgress> = asset_server.load("saves/A.progress.ron");
    let b_handle: Handle<GameProgress> = asset_server.load("saves/B.progress.ron");
    commands.spawn((
        TempGameProgressLoading {
            a: a_handle,
            b: b_handle,
        },
        Name::new("temp_game_progress_loading"),
    ));
}

pub(super) fn is_progress_initializing(temps: Query<(Entity, &TempGameProgressLoading)>) -> bool {
    temps.iter().len() > 0
}

pub(super) fn continue_initializing_game_progress(
    progresses: Res<Assets<GameProgress>>,
    temps: Query<(Entity, &TempGameProgressLoading)>,
    mut commands: Commands,
) {
    let Ok((temp_eid, temp_handle)) = temps.get_single() else {
        return;
    };
    match (
        progresses.get(temp_handle.a.id()),
        progresses.get(temp_handle.b.id()),
    ) {
        (Some(a_data), Some(b_data)) => {
            commands.spawn((a_data.clone(), Name::new("game_progress_a")));
            commands.spawn((b_data.clone(), Name::new("game_progress_b")));
            commands.entity(temp_eid).despawn_recursive();
        }
        _ => (),
    }
}
