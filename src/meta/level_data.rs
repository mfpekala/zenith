use bevy::{
    ecs::system::{SystemId, SystemState},
    prelude::*,
};

use crate::{
    editor::{
        planet::EPlanet,
        point::EPoint,
        start_goal::{EGoal, EStart},
    },
    environment::{
        field::{FieldDrag, FieldStrength},
        goal::{GoalBundle, GoalSize},
        rock::RockKind,
        start::{StartBundle, StartSize},
    },
    ship::ShipBundle,
    uid::{UId, UIdMarker, UIdTranslator},
};

/// For rehydrating exported level data (saves, intermediate editor output) into spawnable things
/// NOTE: _Spawnable_ things. This means you usually implement Rehydrate<SomeBundle>
pub trait Rehydrate<T> {
    fn rehydrate(self) -> T;
}

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
pub struct ExportedRock {
    pub kind: RockKind,
    pub points: Vec<IVec2>,
    pub z: i32,
}

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
pub struct ExportedField {
    pub points: Vec<IVec2>,
    pub dir: Vec2,
    pub strength: FieldStrength,
    pub drag: FieldDrag,
}

/// All the data that exists about a level.
/// Just the data that needs to be used to load/play the level
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
    start: IVec2,
    goal: IVec2,
    rocks: Vec<ExportedRock>,
    fields: Vec<ExportedField>,
}

/// A struct that contains SystemIds for systems relating to exporting/loading levels
#[derive(Resource, Clone)]
pub struct LevelDataOneshots {
    pub crystallize_level_data_id: SystemId<(), LevelData>,
    pub spawn_level_id: SystemId<(u64, LevelData, IVec2)>,
}

pub(super) fn crystallize_level_data(
    world: &mut World,
    params: &mut SystemState<(
        Query<&GlobalTransform, With<EPoint>>,
        Query<&EPlanet>,
        Query<&GlobalTransform, With<EStart>>,
        Query<&GlobalTransform, With<EGoal>>,
        Res<UIdTranslator>,
    )>,
) -> LevelData {
    let (points_q, planets_q, estart, egoal, ut) = params.get(world);
    let start = match estart.get_single() {
        Ok(gt) => IVec2::new(
            gt.translation().x.round() as i32,
            gt.translation().y.round() as i32,
        ),
        _ => IVec2::ZERO,
    };
    let goal = match egoal.get_single() {
        Ok(gt) => IVec2::new(
            gt.translation().x.round() as i32,
            gt.translation().y.round() as i32,
        ),
        _ => IVec2::ZERO,
    };
    let rocks = planets_q
        .iter()
        .map(|planet| ExportedRock {
            kind: planet.rock_kind,
            points: planet
                .rock_points
                .iter()
                .map(|uid| {
                    let eid = ut.get_entity(*uid).unwrap();
                    let pos = points_q.get(eid).unwrap();
                    let pos = pos.translation().truncate();
                    IVec2::new(pos.x.round() as i32, pos.y.round() as i32)
                })
                .collect(),
            z: 0,
        })
        .collect();
    let mut fields = vec![];
    for planet in planets_q.iter() {
        for field in planet.fields.iter() {
            let points = field
                .field_points
                .iter()
                .map(|uid| {
                    let eid = ut.get_entity(*uid).unwrap();
                    let pos = points_q.get(eid).unwrap();
                    let pos = pos.translation().truncate();
                    IVec2::new(pos.x.round() as i32, pos.y.round() as i32)
                })
                .collect();
            fields.push(ExportedField {
                points,
                dir: field.dir,
                strength: FieldStrength::Normal,
                drag: FieldDrag::Normal,
            })
        }
    }
    LevelData {
        start,
        goal,
        rocks,
        fields,
    }
}

#[derive(Component)]
pub struct LevelRoot;

pub(super) fn spawn_level(
    In((uid, level_data, home)): In<(UId, LevelData, IVec2)>,
    mut commands: Commands,
) {
    commands
        .spawn((
            UIdMarker(uid),
            SpatialBundle::from_transform(Transform::from_translation(home.as_vec2().extend(0.0))),
            Name::new(format!("LevelRoot({})", uid)),
            LevelRoot,
        ))
        .with_children(|parent| {
            // TODO: Sexier level entrance
            parent.spawn(ShipBundle::new(level_data.start));
            parent.spawn(StartBundle::new(StartSize::Medium, level_data.start));
            parent.spawn(GoalBundle::new(GoalSize::Medium, level_data.goal));
            for rock in level_data.rocks {
                parent.spawn(rock.rehydrate());
            }
            for field in level_data.fields {
                parent.spawn(field.rehydrate());
            }
        });
}
