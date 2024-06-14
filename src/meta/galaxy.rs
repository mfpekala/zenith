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
        vec![Self::Basic]
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
    serde::Serialize,
    serde::Deserialize,
    bevy::asset::Asset,
    bevy::reflect::TypePath,
    Debug,
    PartialEq,
    Clone,
    Resource,
    Default,
)]
pub struct GalaxyProgress {
    pub map: HashMap<String, (bool, String)>,
}
