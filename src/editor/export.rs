use std::{
    fs::File,
    io::{Read, Write},
};

use bevy::{ecs::system::SystemState, prelude::*, utils::HashMap};

use crate::{
    meta::level_data::{
        ExportedField, ExportedGoal, ExportedLivePoly, ExportedReplenish, ExportedRock,
        ExportedStart, LevelData,
    },
    physics::dyno::IntMoveable,
};

use super::{
    efield::{EField, EFieldBundle},
    egoal::{EGoal, EGoalBundle},
    elive_poly::ELivePoly,
    eoneshots::EOneshots,
    epoint::{EPoint, EPointBundle, EPointGroup, ShinyThingBundle},
    ereplenish::{EReplenish, EReplenishBundle},
    erock::{ERock, ERockBundle},
    estart::{EStart, EStartBundle},
    transitions::ERootEid,
};

pub fn freeze_level_data(
    In(()): In<()>,
    points_q: Query<(Entity, &IntMoveable), With<EPoint>>,
    start_q: Query<Entity, With<EStart>>,
    goal_q: Query<Entity, With<EGoal>>,
    rocks_q: Query<(&EPointGroup, &ERock)>,
    fields_q: Query<(&EPointGroup, &EField)>,
    replenishes_q: Query<Entity, With<EReplenish>>,
    live_polys_q: Query<&EPointGroup, With<ELivePoly>>,
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
    // Freeze the live polys
    let mut exported_live_polys = Vec::<ExportedLivePoly>::new();
    for pg in &live_polys_q {
        let mut u64s = vec![];
        for eid in &pg.eids {
            let u64 = eid2u64.get(eid).ok_or("Bad rock eid")?;
            u64s.push(*u64);
        }
        exported_live_polys.push(ExportedLivePoly { points: u64s });
    }
    Ok(LevelData {
        points,
        start: exported_start,
        goal: exported_goal,
        rocks: exported_rocks,
        fields: exported_fields,
        replenishes: exported_replenishes,
        live_polys: exported_live_polys,
    })
}

pub(super) fn save_level(
    In(name): In<String>,
    world: &mut World,
    params: &mut SystemState<(Res<EOneshots>,)>,
) {
    let (eoneshots,) = params.get_mut(world);
    let eoneshots = eoneshots.clone();
    match world.run_system(eoneshots.freeze_level_data) {
        Ok(Ok(level_data)) => {
            let Ok(mut file) = File::create(format!("assets/levels/{}.level.ron", name)) else {
                warn!("Failed to open file to export to");
                return;
            };
            let Ok(level_string) = ron::to_string(&level_data) else {
                warn!("Failed to serialize level_data to ron string");
                return;
            };
            match file.write_all(level_string.as_bytes()) {
                Ok(_) => {
                    info!("Level exported successfully");
                }
                Err(e) => {
                    warn!("Failed to export level, couldn't write to file: {e:?}");
                }
            };
        }
        Ok(Err(s)) => {
            warn!("I fucked up: Failed to freeze level data: {s:?}");
        }
        Err(e) => {
            warn!("Intrinsic: Failed to freeze level data (system): {e:?}");
        }
    }
}

/// Unfreezes level data TO THE EDITOR. Use a different system for loading to play. (That system uses resources instead of fs.)
/// I think.
pub(super) fn load_level(In(name): In<String>, mut commands: Commands, eroot: Res<ERootEid>) {
    let Ok(mut file) = File::open(format!("assets/levels/{}.level.ron", name)) else {
        warn!("Failed to open file to unfreeze from");
        return;
    };
    let mut s = String::new();
    let Ok(_) = file.read_to_string(&mut s) else {
        warn!("Failed to read string from file");
        return;
    };
    let Ok(level_data) = ron::from_str::<LevelData>(&s) else {
        warn!("Failed to parse level_data from string");
        return;
    };
    commands.entity(eroot.0).despawn_descendants();
    let mut special_spawned_map = HashMap::<u64, (Entity, IVec2)>::new();
    let mut basic_spawned_map = HashMap::<u64, Entity>::new();
    commands.entity(eroot.0).with_children(|parent| {
        parent.spawn(ShinyThingBundle::new());
        for (uid, pos) in level_data.points {
            let eid = parent.spawn(EPointBundle::new(pos)).id();
            special_spawned_map.insert(uid, (eid, pos));
            basic_spawned_map.insert(uid, eid);
        }
        // Basic things (points don't have custom anims)
        for rock in level_data.rocks {
            let erock = rock.rehydrate_edit(&basic_spawned_map).unwrap();
            parent.spawn(erock);
        }
        for field in level_data.fields {
            let efield = field.rehydrate_edit(&basic_spawned_map).unwrap();
            parent.spawn(efield);
        }
    });
    // Special things (points that have custom anims)
    let spawned_start = special_spawned_map[&level_data.start.uid];
    commands
        .entity(spawned_start.0)
        .insert(EStartBundle::new(spawned_start.1));
    let spawned_goal = special_spawned_map[&level_data.goal.uid];
    commands
        .entity(spawned_goal.0)
        .insert(EGoalBundle::new(spawned_goal.1));
    for replenish in level_data.replenishes {
        let spawned_replenish = special_spawned_map[&replenish.uid];
        commands
            .entity(spawned_replenish.0)
            .insert(EReplenishBundle::new(spawned_replenish.1));
    }
}

trait RehydrateEdit<T> {
    fn rehydrate_edit(self, spawned_map: &HashMap<u64, Entity>) -> Result<T, String>;
}

fn get_point_group(
    points: &[u64],
    spawned_map: &HashMap<u64, Entity>,
    minimum: u32,
) -> Result<EPointGroup, String> {
    let mut result = EPointGroup::default();
    result.minimum = minimum;
    for point in points {
        let eid = spawned_map
            .get(point)
            .ok_or("Bad uid in get_point_group".to_string())?;
        result.eids.push(*eid);
        result.poses.push(IVec2::ZERO);
    }
    Ok(result)
}

impl RehydrateEdit<ERockBundle> for ExportedRock {
    fn rehydrate_edit(self, spawned_map: &HashMap<u64, Entity>) -> Result<ERockBundle, String> {
        let pg = get_point_group(&self.points, spawned_map, 3)?;
        Ok(ERockBundle::new(self.kind, pg))
    }
}

impl RehydrateEdit<EFieldBundle> for ExportedField {
    fn rehydrate_edit(self, spawned_map: &HashMap<u64, Entity>) -> Result<EFieldBundle, String> {
        let pg = get_point_group(&self.points, spawned_map, 3)?;
        Ok(EFieldBundle::new(self.dir, pg))
    }
}
