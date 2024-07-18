use bevy::{ecs::system::SystemId, prelude::*, utils::HashMap};

use crate::{
    environment::{
        field::{Field, FieldBundle, FieldDrag, FieldStrength},
        goal::{GoalBundle, GoalSize},
        live_poly::LivePolyBundle,
        replenish::ReplenishBundle,
        rock::{RockBundle, RockKind},
        start::{StartBundle, StartSize},
    },
    ship::ShipBundle,
};

#[derive(Component)]
pub struct LevelRoot;

/// A representation of a level. Note that this functions both as saves of
/// editor state, and what is used when loading a level to play
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
pub struct LevelData {
    pub points: HashMap<u64, IVec2>,
    pub start: ExportedStart,
    pub goal: ExportedGoal,
    pub rocks: Vec<ExportedRock>,
    pub fields: Vec<ExportedField>,
    pub replenishes: Vec<ExportedReplenish>,
    // pub live_polys: Vec<ExportedLivePoly>,
}

fn spawn_level(In((parent_eid, level_data)): In<(Entity, LevelData)>, mut commands: Commands) {
    commands.entity(parent_eid).with_children(|parent| {
        let points = &level_data.points;
        let start_pos = points.get(&level_data.start.uid).unwrap();
        parent.spawn(ShipBundle::new(*start_pos));
        parent.spawn(level_data.start.rehydrate(points).unwrap());
        parent.spawn(level_data.goal.rehydrate(points).unwrap());
        for rock in level_data.rocks {
            parent.spawn(rock.rehydrate(points).unwrap());
        }
        for field in level_data.fields {
            parent.spawn(field.rehydrate(points).unwrap());
        }
        for replenish in level_data.replenishes {
            parent.spawn(replenish.rehydrate(points).unwrap());
        }
        parent.spawn(LivePolyBundle::new(vec![
            Vec2::new(-10000.0, -10000.0),
            Vec2::new(-10000.0, 10000.0),
            Vec2::new(10000.0, 10000.0),
            Vec2::new(10000.0, -10000.0),
        ]));
    });
    commands.entity(parent_eid).insert(LevelRoot);
}

#[derive(Resource, Clone)]
pub struct LevelDataOneshots {
    pub spawn_level: SystemId<(Entity, LevelData)>,
}

pub(super) fn register_level_data_oneshots(app: &mut App) {
    let oneshots = LevelDataOneshots {
        spawn_level: app.world.register_system(spawn_level),
    };
    app.insert_resource(oneshots);
}

trait Rehydrate<T> {
    fn rehydrate(self, points: &HashMap<u64, IVec2>) -> Result<T, String>;
}

#[derive(
    serde::Serialize, serde::Deserialize, bevy::reflect::TypePath, Debug, PartialEq, Clone, Default,
)]
pub struct ExportedStart {
    pub uid: u64,
}

#[derive(
    serde::Serialize, serde::Deserialize, bevy::reflect::TypePath, Debug, PartialEq, Clone, Default,
)]
pub struct ExportedGoal {
    pub uid: u64,
}

#[derive(
    serde::Serialize, serde::Deserialize, bevy::reflect::TypePath, Debug, PartialEq, Clone, Default,
)]
pub struct ExportedRock {
    pub points: Vec<u64>,
    pub kind: RockKind,
}

#[derive(
    serde::Serialize, serde::Deserialize, bevy::reflect::TypePath, Debug, PartialEq, Clone, Default,
)]
pub struct ExportedField {
    pub points: Vec<u64>,
    pub dir: Vec2,
    pub strength: FieldStrength,
    pub drag: FieldDrag,
}

#[derive(
    serde::Serialize, serde::Deserialize, bevy::reflect::TypePath, Debug, PartialEq, Clone, Default,
)]
pub struct ExportedReplenish {
    pub uid: u64,
}

#[derive(
    serde::Serialize, serde::Deserialize, bevy::reflect::TypePath, Debug, PartialEq, Clone, Default,
)]
pub struct ExportedLivePoly {
    pub points: Vec<u64>,
}

fn get_poses(uids: &[u64], points: &HashMap<u64, IVec2>) -> Result<Vec<IVec2>, String> {
    let mut result = vec![];
    for uid in uids {
        let pos = points.get(uid).ok_or("Bad uid")?;
        result.push(*pos);
    }
    Ok(result)
}

impl Rehydrate<StartBundle> for ExportedStart {
    fn rehydrate(self, points: &HashMap<u64, IVec2>) -> Result<StartBundle, String> {
        let pos = points.get(&self.uid).ok_or("Bad uid rehydrate start")?;
        Ok(StartBundle::new(StartSize::Medium, *pos))
    }
}

impl Rehydrate<GoalBundle> for ExportedGoal {
    fn rehydrate(self, points: &HashMap<u64, IVec2>) -> Result<GoalBundle, String> {
        let pos = points.get(&self.uid).ok_or("Bad uid rehydrate goal")?;
        Ok(GoalBundle::new(GoalSize::Medium, *pos))
    }
}

impl Rehydrate<RockBundle> for ExportedRock {
    fn rehydrate(self, points: &HashMap<u64, IVec2>) -> Result<RockBundle, String> {
        let poses = get_poses(&self.points, points)?;
        Ok(RockBundle::new(self.kind, poses))
    }
}

impl Rehydrate<FieldBundle> for ExportedField {
    fn rehydrate(self, points: &HashMap<u64, IVec2>) -> Result<FieldBundle, String> {
        let poses = get_poses(&self.points, points)?;
        Ok(FieldBundle::new(
            Field {
                dir: self.dir,
                strength: self.strength,
                drag: self.drag,
            },
            poses,
        ))
    }
}

impl Rehydrate<ReplenishBundle> for ExportedReplenish {
    fn rehydrate(self, points: &HashMap<u64, IVec2>) -> Result<ReplenishBundle, String> {
        let pos = points.get(&self.uid).ok_or("Bad uid rehydrate replenish")?;
        Ok(ReplenishBundle::new(*pos))
    }
}
