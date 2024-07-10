use bevy::{
    ecs::system::{SystemId, SystemState},
    prelude::*,
};

use crate::{
    camera::CameraMarker,
    editor::{
        field::EStandaloneField,
        planet::EPlanet,
        point::EPoint,
        replenish::EReplenish,
        segment::SegmentParents,
        start_goal::{EGoal, EStart},
    },
    environment::{
        field::{FieldDrag, FieldStrength},
        goal::{GoalBundle, GoalSize},
        live_poly::LivePolyBundle,
        rock::RockKind,
        segment::SegmentKind,
        start::{StartBundle, StartSize},
    },
    physics::dyno::IntMoveable,
    ship::ShipBundle,
    uid::{UId, UIdMarker, UIdTranslator},
};

/// For rehydrating exported level data (saves, intermediate editor output) into spawnable things
/// NOTE: _Spawnable_ things. This means you usually implement Rehydrate<SomeBundle>
pub trait Rehydrate<T> {
    fn rehydrate(self) -> T;
}

#[derive(
    serde::Serialize, serde::Deserialize, bevy::reflect::TypePath, Debug, PartialEq, Clone, Default,
)]
pub struct ExportedRock {
    pub kind: RockKind,
    pub points: Vec<IVec2>,
    pub z: i32,
}

#[derive(
    serde::Serialize, serde::Deserialize, bevy::reflect::TypePath, Debug, PartialEq, Clone, Default,
)]
pub struct ExportedField {
    pub points: Vec<IVec2>,
    pub dir: Vec2,
    pub strength: FieldStrength,
    pub drag: FieldDrag,
}

#[derive(
    serde::Serialize, serde::Deserialize, bevy::reflect::TypePath, Debug, PartialEq, Clone, Default,
)]
pub struct ExportedSegment {
    pub kind: SegmentKind,
    pub left_parent: IVec2,
    pub right_parent: IVec2,
}

#[derive(
    serde::Serialize, serde::Deserialize, bevy::reflect::TypePath, Debug, PartialEq, Clone, Default,
)]
pub struct ExportedReplenish {
    pub pos: IVec2,
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
    segments: Vec<ExportedSegment>,
    replenishes: Vec<ExportedReplenish>,
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
        Query<(&SegmentParents, &SegmentKind)>,
        Query<&GlobalTransform, With<EReplenish>>,
        Query<&EStandaloneField>,
        Res<UIdTranslator>,
    )>,
) -> LevelData {
    let (points_q, planets_q, estart, egoal, segments_q, replenish_q, esf_q, ut) =
        params.get(world);
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
    let mut segments = vec![];
    for (parents, kind) in segments_q.iter() {
        let Some(eid_left) = ut.get_entity(parents.left_uid) else {
            continue;
        };
        let Some(eid_right) = ut.get_entity(parents.right_uid) else {
            continue;
        };
        let Ok(left_point) = points_q.get(eid_left) else {
            continue;
        };
        let Ok(right_point) = points_q.get(eid_right) else {
            continue;
        };
        let left_parent = left_point.translation().truncate();
        let left_parent = IVec2::new(left_parent.x.round() as i32, left_parent.y.round() as i32);
        let right_parent = right_point.translation().truncate();
        let right_parent = IVec2::new(right_parent.x.round() as i32, right_parent.y.round() as i32);

        segments.push(ExportedSegment {
            kind: *kind,
            left_parent,
            right_parent,
        });
    }
    let mut replenishes = vec![];
    for replenish in replenish_q.iter() {
        replenishes.push(ExportedReplenish {
            pos: IVec2::new(
                replenish.translation().x.round() as i32,
                replenish.translation().y.round() as i32,
            ),
        });
    }
    for esf in esf_q.iter() {
        let point_eids = esf
            .field_points
            .iter()
            .map(|uid| ut.get_entity(*uid).unwrap())
            .collect::<Vec<_>>();
        let points = point_eids
            .into_iter()
            .map(|eid| {
                let tran = points_q.get(eid).unwrap().translation().truncate();
                IVec2::new(tran.x.round() as i32, tran.y.round() as i32)
            })
            .collect();
        fields.push(ExportedField {
            points,
            dir: esf.dir,
            strength: FieldStrength::Normal,
            drag: FieldDrag::Normal,
        })
    }
    LevelData {
        start,
        goal,
        rocks,
        fields,
        segments,
        replenishes,
    }
}

#[derive(Component)]
pub struct LevelRoot;

pub(super) fn spawn_level(
    In((uid, level_data, home)): In<(UId, LevelData, IVec2)>,
    mut commands: Commands,
    mut camera_q: Query<&mut IntMoveable, With<CameraMarker>>,
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
            let mut all_points = vec![];
            parent.spawn(ShipBundle::new(level_data.start));
            parent.spawn(StartBundle::new(StartSize::Medium, level_data.start));
            all_points.push(level_data.start.as_vec2());
            parent.spawn(GoalBundle::new(GoalSize::Medium, level_data.goal));
            all_points.push(level_data.goal.as_vec2());
            for rock in level_data.rocks {
                for point in rock.points.iter() {
                    all_points.push(point.as_vec2());
                }
                parent.spawn(rock.rehydrate());
            }
            for field in level_data.fields {
                for point in field.points.iter() {
                    all_points.push(point.as_vec2());
                }
                parent.spawn(field.rehydrate());
            }
            for segment in level_data.segments {
                parent.spawn(segment.rehydrate());
            }
            for repl in level_data.replenishes {
                all_points.push(repl.pos.as_vec2());
                parent.spawn(repl.rehydrate());
            }
            let live_poly = LivePolyBundle::new(all_points);
            parent.spawn(live_poly);
        });
    if let Ok(mut camera) = camera_q.get_single_mut() {
        camera.pos = home.extend(0) + level_data.start.extend(0);
    }
}
