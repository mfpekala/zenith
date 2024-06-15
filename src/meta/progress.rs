use bevy::{prelude::*, utils::HashMap};

#[derive(Debug, Clone)]
pub struct LevelMetaData {
    pub id: String,
    pub title: String,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GalaxyKind {
    Basic,
    Springy,
}
impl std::fmt::Display for GalaxyKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl GalaxyKind {
    pub fn rank(&self) -> u32 {
        match self {
            Self::Basic => 0,
            Self::Springy => 1,
        }
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
    galaxy_map: HashMap<String, (bool, String)>,
}
impl GameProgress {
    /// Gets the (completed, active_level) status for a given galaxy
    pub fn get_galaxy_progress(&self, kind: GalaxyKind) -> (bool, String) {
        self.galaxy_map.get(&kind.to_string()).unwrap().clone()
    }

    /// Returns the earliest incomplete galaxy, i.e., the galaxy the player is actively playing
    pub fn first_incomplete_galaxy(&self) -> GalaxyKind {
        for kind in GalaxyKind::all() {
            let (completed, _) = self.get_galaxy_progress(kind);
            if !completed {
                return kind;
            }
        }
        GalaxyKind::Basic
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
