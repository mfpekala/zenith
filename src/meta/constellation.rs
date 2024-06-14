use bevy::{prelude::*, utils::HashMap};

#[derive(Debug, Clone)]
pub struct LevelMetaData {
    pub id: String,
    pub title: String,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstellationKind {
    Basic,
    Springy,
}
impl std::fmt::Display for ConstellationKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone)]
pub struct ConstellationMetaData {
    pub title: String,
    pub description: String,
    pub levels: Vec<LevelMetaData>,
}

// TODO: Yeah so this ends up just being stored on code segment.
// probably not terrible but probably should shove into a file at some point
impl ConstellationKind {
    pub fn all() -> Vec<Self> {
        vec![Self::Basic]
    }

    pub fn to_levels(&self) -> Vec<LevelMetaData> {
        match self {
            Self::Basic => vec![
                LevelMetaData {
                    id: "cbasic_1".to_string(),
                    title: "First level".to_string(),
                    description: "Just testing 1".to_string(),
                },
                LevelMetaData {
                    id: "cbasic_2".to_string(),
                    title: "Second level".to_string(),
                    description: "Just testing 2".to_string(),
                },
            ],
            Self::Springy => vec![
                LevelMetaData {
                    id: "cspringy_1".to_string(),
                    title: "Spring intro".to_string(),
                    description: "Introducing the player to springs".to_string(),
                },
                LevelMetaData {
                    id: "cspringy_2".to_string(),
                    title: "Springs go brrr".to_string(),
                    description: "Yeah, so, springs".to_string(),
                },
            ],
        }
    }

    pub fn to_meta_data(&self) -> ConstellationMetaData {
        let (title, description) = match self {
            Self::Basic => ("Basic", "A basic, test constellation"),
            Self::Springy => ("Spring", "For learning about springs"),
        };
        ConstellationMetaData {
            title: title.to_string(),
            description: description.to_string(),
            levels: self.to_levels(),
        }
    }
}

/// Maps constellation (enum as string) to (completed, id_of_level_on)
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
pub struct ConstellationProgress {
    pub map: HashMap<String, (bool, String)>,
}
