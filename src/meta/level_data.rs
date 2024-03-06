use crate::{
    editor::{
        editable_goal::EditableGoalBundle, editable_point::EditablePointBundle,
        editable_rock::EditableRockBundle, editable_starting_point::EditableStartingPointBundle,
    },
    environment::{
        field::{Field, FieldBundle},
        goal::GoalBundle,
        rock::{Rock, RockBundle, RockFeatures, RockKind},
        starting_point::StartingPointBundle,
    },
};
use bevy::{ecs::system::SystemId, prelude::*, utils::HashMap};
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

pub fn get_level_folder() -> PathBuf {
    Path::new("assets/levels").into()
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SaveableRock {
    pub kind: RockKind,
    pub points: Vec<Vec2>,
    pub reach: Option<f32>,
}
impl SaveableRock {
    pub fn spawn_editable_rock(&self, commands: &mut Commands) {
        let mut center = Vec2::ZERO;
        let mut epoint_ids: Vec<Entity> = vec![];
        let num_points = self.points.len();
        let gravity_reach_point = match self.reach {
            Some(dist) => {
                let point0 = self.points.first().unwrap().clone();
                let pointn1 = self.points.last().unwrap().clone();
                let pointp1 = self.points[1];
                let diff1 = point0 - pointn1;
                let diff2 = pointp1 - point0;
                let perp = (diff1.normalize() + diff2.normalize()).normalize().perp();
                let grp_pos = point0 + perp * dist;
                Some(commands.spawn(EditablePointBundle::new(grp_pos)).id())
            }
            None => None,
        };
        for point in self.points.iter() {
            let point_id = commands.spawn(EditablePointBundle::new(*point)).id();
            epoint_ids.push(point_id);
            center += *point;
        }
        let center = center / (num_points as f32);
        let gravity_strength = Some(0.06);
        let erock = EditableRockBundle::from_points(
            center,
            epoint_ids,
            gravity_reach_point,
            gravity_strength,
            self.kind.clone(),
        );
        commands.spawn(erock);
    }

    pub fn spawn_rock(
        &self,
        commands: &mut Commands,
        feature_map: &HashMap<RockKind, RockFeatures>,
        meshes: &mut ResMut<Assets<Mesh>>,
    ) {
        if self.points.len() <= 0 {
            return;
        }
        let mut center = Vec2::ZERO;
        for point in self.points.iter() {
            center += *point;
        }
        center /= self.points.len() as f32;
        let rock = Rock {
            points: self
                .points
                .clone()
                .into_iter()
                .map(|p| p - center)
                .collect(),
            kind: self.kind.clone(),
            features: feature_map.get(&self.kind).unwrap().clone(),
        };
        // TODO: Fix the editor so it doesn't tie rocks to fields as much
        RockBundle::spawn(commands, center, rock, meshes);
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SaveableField {
    pub points: Vec<Vec2>,
    pub strength: f32,
    pub dir: Vec2,
    pub drag: f32,
}
impl SaveableField {
    pub fn spawn_editable_field(&self, _commands: &mut Commands) {
        unimplemented!();
    }

    pub fn spawn_field(&self, commands: &mut Commands) {
        let (field, pos) = Field::from_saveable(&self);
        FieldBundle::spawn(commands, pos, field);
    }
}

/// All the data that exists about a level.
/// Just the data that needs to be used to load/play the level
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct LevelData {
    pub next_level: Option<String>,
    pub starting_point: Vec2,
    pub goal_point: Vec2,
    pub rocks: Vec<SaveableRock>,
    pub fields: Vec<SaveableField>,
}
impl LevelData {
    pub fn blank() -> Self {
        Self {
            next_level: None,
            starting_point: Vec2 { x: -30.0, y: 0.0 },
            goal_point: Vec2 { x: 30.0, y: 0.0 },
            rocks: vec![],
            fields: vec![],
        }
    }

    pub fn load(file: PathBuf) -> Option<Self> {
        let mut fin = File::open(file).unwrap();
        let mut as_string = String::new();
        fin.read_to_string(&mut as_string).ok();
        match serde_json::from_str(&as_string) {
            Ok(ld) => Some(ld),
            _ => None,
        }
    }

    pub fn load_editor(&self, commands: &mut Commands) {
        commands.spawn(EditableStartingPointBundle::new(self.starting_point));
        commands.spawn(EditableGoalBundle::new(self.goal_point));
        for rock in self.rocks.iter() {
            rock.spawn_editable_rock(commands);
        }
    }

    pub fn load_level(
        &self,
        commands: &mut Commands,
        feature_map: &HashMap<RockKind, RockFeatures>,
        meshes: &mut ResMut<Assets<Mesh>>,
        spawn_ship_id: SystemId<(Vec2, f32)>,
    ) {
        for srock in self.rocks.iter() {
            srock.spawn_rock(commands, feature_map, meshes);
        }
        for field in self.fields.iter() {
            field.spawn_field(commands);
        }
        commands.spawn(StartingPointBundle::new(self.starting_point));
        GoalBundle::spawn(self.goal_point, commands);
        commands.run_system_with_input(spawn_ship_id, (self.starting_point, 15.0));
    }
}
