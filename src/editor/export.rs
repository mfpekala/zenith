use bevy::{prelude::*, utils::HashMap};

use crate::{
    meta::level_data::{
        ExportedField, ExportedGoal, ExportedReplenish, ExportedRock, ExportedStart, LevelData,
    },
    physics::dyno::IntMoveable,
};

use super::{
    efield::EField,
    egoal::EGoal,
    epoint::{EPoint, EPointGroup},
    ereplenish::EReplenish,
    erock::ERock,
    estart::EStart,
};

pub fn freeze_level_data(
    In(()): In<()>,
    points_q: Query<(Entity, &IntMoveable), With<EPoint>>,
    start_q: Query<Entity, With<EStart>>,
    goal_q: Query<Entity, With<EGoal>>,
    rocks_q: Query<(&EPointGroup, &ERock)>,
    fields_q: Query<(&EPointGroup, &EField)>,
    replenishes_q: Query<Entity, With<EReplenish>>,
) -> Result<LevelData, String> {
    // Freeze the points
    let mut eid2u64 = HashMap::<Entity, u64>::new();
    let mut points = HashMap::<u64, IVec2>::new();
    let mut id = 0_u64;
    for (eid, mv) in &points_q {
        eid2u64.insert(eid, id);
        points.insert(id, mv.get_ipos().truncate());
        id += 1;
    }
    // Freeze the start/goal
    let start_eid = start_q.get_single().map_err(|e| format!("{e:?}"))?;
    let start_uid = eid2u64.get(&start_eid).ok_or("Bad start_eid".to_string())?;
    let exported_start = ExportedStart { uid: *start_uid };
    let goal_eid = goal_q.get_single().map_err(|e| format!("{e:?}"))?;
    let goal_uid = eid2u64.get(&goal_eid).ok_or("Bad goal_eid".to_string())?;
    let exported_goal = ExportedGoal { uid: *goal_uid };
    // Freeze the rocks
    let mut exported_rocks = Vec::<ExportedRock>::new();
    for (pg, rock) in &rocks_q {
        let mut u64s = vec![];
        for eid in &pg.eids {
            let u64 = eid2u64.get(eid).ok_or("Bad rock eid")?;
            u64s.push(*u64);
        }
        exported_rocks.push(ExportedRock {
            points: u64s,
            kind: rock.kind,
        })
    }
    // Freeze the fields
    let mut exported_fields = Vec::<ExportedField>::new();
    for (pg, field) in &fields_q {
        let mut u64s = vec![];
        for eid in &pg.eids {
            let u64 = eid2u64.get(eid).ok_or("Bad rock eid")?;
            u64s.push(*u64);
        }
        exported_fields.push(ExportedField {
            points: u64s,
            dir: field.dir,
            strength: default(),
            drag: default(),
        })
    }
    // Freeze the replenishes
    let mut exported_replenishes = Vec::<ExportedReplenish>::new();
    for eid in &replenishes_q {
        let uid = eid2u64.get(&eid).ok_or("Bad replenish eid".to_string())?;
        exported_replenishes.push(ExportedReplenish { uid: *uid });
    }
    Ok(LevelData {
        points,
        start: exported_start,
        goal: exported_goal,
        rocks: exported_rocks,
        fields: exported_fields,
        replenishes: exported_replenishes,
    })
}
