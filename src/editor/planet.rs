use std::cmp::Ordering;

use bevy::{
    prelude::*,
    utils::hashbrown::{HashMap, HashSet},
};
use serde::{Deserialize, Serialize};

use crate::{
    drawing::{
        bordered_mesh::BorderedMesh, layering::sprite_layer, mesh::outline_points,
        sprite_mat::SpriteMaterial,
    },
    editor::point::EPointKind,
    input::MouseState,
    math::MathLine,
    meta::game_state::{EditingMode, GameState, SetGameState},
    physics::dyno::IntMoveable,
};

use super::point::{ChangeEPointKind, EPoint, EPointBundle};

#[derive(Component, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub(super) struct EPlanetField {
    pub field_points: Vec<Entity>,
    pub mesh_id: Entity,
    dir: Vec2,
}

#[derive(Component, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub(super) struct EPlanet {
    pub rock_points: Vec<Entity>,
    pub rock_mesh_id: Option<Entity>,
    pub wild_points: Vec<Entity>,
    pub fields: Vec<EPlanetField>,
}

#[derive(Component, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub(super) struct PendingField {
    groups: Vec<u32>,
}

#[derive(Bundle, Default)]
pub(super) struct EPlanetBundle {
    eplanet: EPlanet,
    spatial: SpatialBundle,
    moveable: IntMoveable,
}
impl EPlanetBundle {
    pub fn spawn(
        commands: &mut Commands,
        pos: IVec2,
        asset_server: &Res<AssetServer>,
        mats: &mut ResMut<Assets<SpriteMaterial>>,
        meshes: &mut ResMut<Assets<Mesh>>,
    ) -> Entity {
        let mut rock_mesh_id = None;
        let entity = commands
            .spawn(EPlanetBundle::default())
            .with_children(|parent| {
                let new_rock_mesh_id = BorderedMesh::spawn_easy(
                    parent,
                    asset_server,
                    meshes,
                    mats,
                    vec![],
                    ("textures/play_inner.png", UVec2::new(36, 36)),
                    Some(("textures/play_outer.png", UVec2::new(36, 36))),
                    Some(3.0),
                    sprite_layer(),
                    0,
                );
                rock_mesh_id = Some(new_rock_mesh_id);
            })
            .id();
        commands.entity(entity).remove::<EPlanetBundle>();
        commands.entity(entity).insert(EPlanetBundle {
            eplanet: EPlanet {
                rock_mesh_id,
                ..default()
            },
            spatial: SpatialBundle {
                transform: Transform::from_translation(pos.as_vec2().extend(0.0)),
                visibility: Visibility::Visible,
                ..default()
            },
            moveable: IntMoveable::new(pos.extend(0)),
        });
        entity
    }
}

/// Handle transitions between free, creating, editing
/// NOTE: Except the creating -> editing transition, that's handled in spawn points
pub(super) fn planet_state_input(
    mut commands: Commands,
    gs: Res<GameState>,
    mut gs_writer: EventWriter<SetGameState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_state: Res<MouseState>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    asset_server: Res<AssetServer>,
    mut mats: ResMut<Assets<SpriteMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    points: Query<(&EPoint, &Parent)>,
    eplanets: Query<Entity, With<EPlanet>>,
) {
    let Some(mode) = gs.get_editing_mode() else {
        return;
    };
    let num_points_hovered = points.iter().filter(|(p, _)| p.is_hovered).count();
    match mode {
        EditingMode::Free => {
            if keyboard.just_pressed(KeyCode::KeyP) {
                let id = EPlanetBundle::spawn(
                    &mut commands,
                    mouse_state.world_pos,
                    &asset_server,
                    &mut mats,
                    &mut meshes,
                );
                gs_writer.send(SetGameState(
                    EditingMode::CreatingPlanet(id).to_game_state(),
                ));
            } else if mouse_buttons.just_pressed(MouseButton::Left) {
                for (point, parent) in points.iter() {
                    if point.is_hovered && eplanets.get(parent.get()).is_ok() {
                        gs_writer.send(SetGameState(
                            EditingMode::EditingPlanet(parent.get()).to_game_state(),
                        ));
                        return;
                    }
                }
            }
        }
        EditingMode::CreatingPlanet(_) | EditingMode::EditingPlanet(_) => {
            if mouse_buttons.just_pressed(MouseButton::Left) {
                if num_points_hovered == 0 {
                    gs_writer.send(SetGameState(EditingMode::Free.to_game_state()));
                }
            }
        }
    }
}

/// On cmd+f, redo the field of the active planet
/// NOTE: Must be editing the planet
pub(super) fn redo_fields(
    mut commands: Commands,
    mut eplanets: Query<&mut EPlanet>,
    points: Query<(Entity, &IntMoveable, &mut EPoint)>,
    gs: Res<GameState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    asset_server: Res<AssetServer>,
) {
    let planet_id = match gs.get_editing_mode() {
        Some(EditingMode::EditingPlanet(id)) => id,
        _ => return,
    };
    if !((keyboard.pressed(KeyCode::SuperLeft) && keyboard.just_pressed(KeyCode::KeyF))
        || (keyboard.just_pressed(KeyCode::SuperLeft) && keyboard.pressed(KeyCode::KeyF)))
    {
        return;
    }
    // Despawn and clear the old fields
    let mut eplanet = eplanets.get_mut(planet_id).unwrap();
    let mut despawned = HashSet::new();
    for field in eplanet.fields.iter() {
        for id in field.field_points.iter() {
            if points.get(*id).is_err() || eplanet.rock_points.iter().any(|rid| rid == id) {
                // Ignore the rock points of the field
                continue;
            }
            if despawned.contains(id) {
                continue;
            }
            commands.entity(*id).despawn_recursive();
            despawned.insert(*id);
        }
        commands.entity(field.mesh_id).despawn_recursive();
    }
    eplanet.fields.clear();
    // Make the new points
    let poses: Vec<Vec2> = eplanet
        .rock_points
        .iter()
        .map(|id| points.get(*id).unwrap().1.pos.truncate().as_vec2())
        .collect();
    let new_poses: Vec<IVec2> = outline_points(&poses, 60.0)
        .into_iter()
        .map(|pos| IVec2::new(pos.x.round() as i32, pos.y.round() as i32))
        .collect();

    for ix in 0..new_poses.len() {
        let next_ix = (ix + 1).rem_euclid(new_poses.len());
        let fp = new_poses[ix];
        let parent = points.get(eplanet.rock_points[ix]).unwrap();
        let mut fp_id = planet_id;
        // TODO: This actually spawns two points per point
        commands.entity(planet_id).with_children(|parent| {
            fp_id = EPointBundle::spawn(parent, &asset_server, fp, EPointKind::Wild);
        });
        commands.entity(fp_id).insert(PendingField {
            groups: vec![ix as u32, next_ix as u32],
        });
        commands.entity(parent.0).insert(PendingField {
            groups: vec![ix as u32, next_ix as u32],
        });
    }
}

/// To make new fields, you mark all the points that should be part of that field
/// on the editing rock with Some([vec of u32s for which fields it's a part of])
/// This function goes in and resolves all that data, spitting out new fields
/// on the active editing rock as it goes
pub(super) fn resolve_pending_fields(
    mut commands: Commands,
    gs: Res<GameState>,
    mut eplanets: Query<(&mut EPlanet, &IntMoveable)>,
    stable_points: Query<&IntMoveable, Without<PendingField>>,
    mut points: Query<
        (
            Entity,
            &mut EPoint,
            &mut IntMoveable,
            &PendingField,
            &Parent,
        ),
        Without<EPlanet>,
    >,
    asset_server: Res<AssetServer>,
    mut mats: ResMut<Assets<SpriteMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let planet_id = match gs.get_editing_mode() {
        Some(EditingMode::EditingPlanet(id)) => id,
        _ => return,
    };
    let Ok(mut eplanet) = eplanets.get_mut(planet_id) else {
        return;
    };

    // Construct the groupmap (fields)
    let mut rock_points = HashMap::new();
    let mut group_map = HashMap::<u32, Vec<(Entity, EPoint, IntMoveable, Entity)>>::new();
    for (id, epoint, mv, pf, parent) in points.iter() {
        for group in pf.groups.iter() {
            if group_map.contains_key(group) {
                let existing = group_map.get_mut(group).unwrap();
                existing.push((id, epoint.clone(), mv.clone(), parent.get()));
            } else {
                group_map.insert(*group, vec![(id, epoint.clone(), mv.clone(), parent.get())]);
            }
        }
        if epoint.kind == EPointKind::Rock {
            rock_points.insert(id, mv.pos.truncate());
        }
    }

    // Spawn a mesh for each field, and add it to the planet
    for (_, items) in group_map.into_iter() {
        let mut points_n_ids = vec![];
        let mut center = Vec2::ZERO;
        let to_ang = |thing: Vec2| (thing.x as f32).atan2(thing.y as f32);
        for (id, epoint, mv, parent) in items {
            match epoint.kind {
                EPointKind::Rock | EPointKind::Wild => {
                    let pos = mv.pos.truncate();
                    points_n_ids.push((id, pos));
                    center += mv.pos.truncate().as_vec2();
                }
                EPointKind::Field => {
                    let mv = match points.get(parent) {
                        Ok(thing) => thing.2,
                        Err(_) => stable_points.get(parent).unwrap(),
                    };
                    let pos = mv.pos.truncate();
                    points_n_ids.push((id, pos));
                    center += mv.pos.truncate().as_vec2();
                }
            }
        }
        center /= points_n_ids.len() as f32;
        points_n_ids.sort_by(|a, b| {
            let a_ang = to_ang(a.1.as_vec2() - center);
            let b_ang = to_ang(b.1.as_vec2() - center);
            if a_ang < b_ang {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        });
        let mesh_points = points_n_ids
            .clone()
            .into_iter()
            .map(|thing| thing.1)
            .collect();
        let id_order: Vec<Entity> = points_n_ids
            .clone()
            .into_iter()
            .map(|thing| thing.0)
            .collect();
        commands.entity(id_order[0]).insert(UpdateFieldGravity); // Triggers the field's gravity to update on the next frame
        commands.entity(planet_id).with_children(|parent| {
            let mesh_id = BorderedMesh::spawn_easy(
                parent,
                &asset_server,
                &mut meshes,
                &mut mats,
                mesh_points,
                ("sprites/field/field_bg.png", UVec2::new(12, 12)),
                None,
                None,
                sprite_layer(),
                -1,
            );
            let field = EPlanetField {
                field_points: id_order,
                mesh_id,
                dir: Vec2::ZERO,
            };
            eplanet.0.fields.push(field);
        });
    }

    // Cleanup all the groups and turn wild points into field points
    for (id, mut point, mut mv, _, parent) in points.iter_mut() {
        commands.entity(id).remove::<PendingField>();
        if point.kind == EPointKind::Wild {
            let mut min_dist = i32::MAX;
            let mut new_dad = planet_id;
            let mut dad_pos = IVec2::ZERO;
            for (id, pos) in rock_points.iter() {
                let dist = mv.pos.truncate().distance_squared(*pos);
                if dist < min_dist {
                    min_dist = dist;
                    new_dad = *id;
                    dad_pos = *pos;
                }
            }
            // adoption????!@??!??
            commands.entity(parent.get()).remove_children(&[id]);
            commands.entity(new_dad).push_children(&[id]);
            point.kind = EPointKind::Field;
            eplanet.0.wild_points.retain(|p| *p != id);
            let old_pos = mv.pos;
            mv.pos = old_pos - dad_pos.extend(0);
            commands
                .entity(id)
                .insert(ChangeEPointKind(EPointKind::Field));
        }
    }
}

/// On cmd + / cmd -, nudge all field points closer/further from their parent
pub(super) fn nudge_fields(
    eplanets: Query<&EPlanet>,
    mut points: Query<(&EPoint, &mut IntMoveable)>,
    gs: Res<GameState>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if !keyboard.pressed(KeyCode::SuperLeft)
        || (!keyboard.pressed(KeyCode::Comma) && !keyboard.pressed(KeyCode::Period))
    {
        return;
    }
    let Some(mode) = gs.get_editing_mode() else {
        return;
    };
    let EditingMode::EditingPlanet(planet_id) = mode else {
        return;
    };
    let Ok(eplanet) = eplanets.get(planet_id) else {
        return;
    };
    for field in eplanet.fields.iter() {
        for id in field.field_points.iter() {
            let Ok((point, mut mv)) = points.get_mut(*id) else {
                continue;
            };
            if point.kind != EPointKind::Field {
                continue;
            }
            let change = mv.pos.as_vec3().truncate().normalize();
            let mult = if keyboard.pressed(KeyCode::Period) {
                1.0
            } else {
                -1.0
            };
            let hmm = mv.pos.truncate().as_vec2() + mv.rem;
            if hmm.length_squared() < 16.0 {
                if mult < 0.0 {
                    continue;
                }
            }
            mv.rem += mult * change;
        }
    }
}

#[derive(Component, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub(super) struct FeralEPoint;

/// If multiple points from the same field are selected, delete that field
/// Turns points into wild points if needed
pub(super) fn remove_field(
    mut eplanets: Query<&mut EPlanet>,
    points: Query<(Entity, &EPoint)>,
    gs: Res<GameState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
) {
    let Some(mode) = gs.get_editing_mode() else {
        return;
    };
    let EditingMode::EditingPlanet(planet_id) = mode else {
        return;
    };
    let Ok(mut eplanet) = eplanets.get_mut(planet_id) else {
        return;
    };
    if !keyboard.just_pressed(KeyCode::KeyC) {
        return;
    }
    let selected_ids: Vec<Entity> = points
        .iter()
        .filter(|p| p.1.is_selected)
        .map(|p| p.0)
        .collect();
    let mut maybe_feral = HashSet::new();
    for field in eplanet.fields.iter_mut() {
        if selected_ids.iter().all(|p| field.field_points.contains(p)) {
            commands.entity(field.mesh_id).despawn_recursive();
            for id in field.field_points.iter() {
                maybe_feral.insert(*id);
            }
            // hackety hack
            field.mesh_id = planet_id;
        }
    }
    eplanet.fields.retain(|field| field.mesh_id != planet_id);
    for id in maybe_feral.into_iter() {
        let point = points.get(id).unwrap();
        if point.1.kind != EPointKind::Field {
            continue;
        }
        if !eplanet
            .fields
            .iter()
            .any(|field| field.field_points.contains(&id))
        {
            commands.entity(id).insert(FeralEPoint);
        }
    }
}

/// Actually makes old field points wild (adjusting transform and spawning thing to change sprite)
pub(super) fn handle_feral_points(
    stable: Query<(&IntMoveable, &Parent), (Without<FeralEPoint>, With<EPoint>)>,
    mut points: Query<(Entity, &mut EPoint, &mut IntMoveable, &Parent), With<FeralEPoint>>,
    mut commands: Commands,
) {
    for mut feral_point in points.iter_mut() {
        let parent = stable.get(feral_point.3.get()).unwrap();
        feral_point.2.pos += parent.0.pos;
        feral_point.1.kind = EPointKind::Wild;
        commands
            .entity(feral_point.3.get())
            .remove_children(&[feral_point.0]);
        commands
            .entity(parent.1.get())
            .push_children(&[feral_point.0]);
        commands
            .entity(feral_point.0)
            .insert(ChangeEPointKind(EPointKind::Wild));
        commands.entity(feral_point.0).remove::<FeralEPoint>();
    }
}

/// Makes a new field from the selected points
pub(super) fn make_new_field(
    eplanets: Query<&EPlanet>,
    points: Query<(Entity, &EPoint)>,
    gs: Res<GameState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
) {
    let Some(mode) = gs.get_editing_mode() else {
        return;
    };
    let EditingMode::EditingPlanet(planet_id) = mode else {
        return;
    };
    if !keyboard.just_pressed(KeyCode::KeyF) {
        return;
    }
    let selected_info: Vec<(Entity, EPoint)> = points
        .iter()
        .filter(|p| p.1.is_selected)
        .map(|thing| (thing.0, thing.1.clone()))
        .collect();
    let valid = selected_info
        .iter()
        .any(|p| p.1.kind == EPointKind::Wild || p.1.kind == EPointKind::Field)
        && selected_info.iter().any(|p| p.1.kind == EPointKind::Rock);
    if !valid {
        return;
    }
    for (id, _) in selected_info {
        commands.entity(id).insert(PendingField { groups: vec![0] });
    }
}

/// Any point in a field that has this component should update it's gravity
#[derive(Component, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub(super) struct UpdateFieldGravity;

/// Updates the gravity direction of fields.
/// Fields with 0 rock points - do nothing
/// Fields with 1 rock point - get average of field points, direct towards rock point
/// Fields with 2 rock points - perpendicular, assuming clockwise like normal
/// Fields with 3+ rock points - get line of best fit. Then pick perp dir using center of field points
pub(super) fn update_field_gravity(
    mut eplanets: Query<&mut EPlanet>,
    points: Query<(
        Entity,
        &EPoint,
        &IntMoveable,
        Option<&UpdateFieldGravity>,
        &Parent,
    )>,
    gs: Res<GameState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
) {
    let Some(mode) = gs.get_editing_mode() else {
        return;
    };
    let EditingMode::EditingPlanet(planet_id) = mode else {
        return;
    };
    let Ok(mut eplanet) = eplanets.get_mut(planet_id) else {
        return;
    };
    // First do the input
    if keyboard.just_pressed(KeyCode::KeyG) {
        if keyboard.pressed(KeyCode::ShiftLeft) {
            for field in eplanet.fields.iter() {
                commands
                    .entity(field.field_points[0])
                    .insert(UpdateFieldGravity);
            }
        } else {
            for point in points.iter() {
                if point.1.is_selected {
                    commands.entity(point.0).insert(UpdateFieldGravity);
                }
            }
        }
    }

    // Then actually do the updating
    let mut affected_points = HashSet::<Entity>::new();
    for point in points.iter() {
        if point.3.is_some() {
            affected_points.insert(point.0);
        }
    }
    for field in eplanet.fields.iter_mut() {
        let adjusting = field
            .field_points
            .iter()
            .any(|id| affected_points.contains(id));
        if !adjusting {
            continue;
        }
        for id in field.field_points.iter() {
            commands.entity(*id).remove::<UpdateFieldGravity>();
        }
        let mut rock_points = vec![];
        let mut field_points = vec![];
        for point in field.field_points.iter() {
            let point = points.get(*point).unwrap();
            if point.1.kind == EPointKind::Rock {
                rock_points.push((point.2.pos.truncate()).as_vec2());
            } else {
                let parent = points.get(point.4.get()).unwrap();
                field_points.push((parent.2.pos.truncate() + point.2.pos.truncate()).as_vec2());
            }
        }
        if rock_points.len() == 0 {
            continue;
        }
        let field_center = field_points
            .clone()
            .into_iter()
            .reduce(|acc, e| acc + e)
            .unwrap()
            / field_points.len() as f32;
        let rock_center = rock_points
            .clone()
            .into_iter()
            .reduce(|acc, e| acc + e)
            .unwrap()
            / rock_points.len() as f32;
        if rock_points.len() == 1 {
            field.dir = (rock_points[0] - field_center).normalize_or_zero();
        } else if rock_points.len() == 2 {
            let pall = rock_points[1] - rock_points[0];
            let perp = Vec2::new(-pall.y, pall.x);
            field.dir = perp.normalize_or_zero();
            let check_diff = rock_center - field_center;
            if field.dir.dot(check_diff) < 0.0 {
                field.dir *= -1.0;
            }
        } else {
            let line = MathLine::slope_fit_points(&rock_points);
            let perp = Vec2::new(line.rise(), -line.run());
            field.dir = perp.normalize_or_zero();
            let check_diff = rock_center - field_center;
            if field.dir.dot(check_diff) < 0.0 {
                field.dir *= -1.0;
            }
        }
    }
}

pub(super) fn drive_planet_meshes(
    points: Query<(&EPoint, &IntMoveable, &Parent)>,
    eplanets: Query<&EPlanet>,
    mut bms: Query<&mut BorderedMesh>,
) {
    for eplanet in eplanets.iter() {
        // First update the rock mesh
        let mut mesh_points = vec![];
        for pid in eplanet.rock_points.iter() {
            if let Ok((_, mv, _)) = points.get(*pid) {
                mesh_points.push(mv.pos.truncate());
            }
        }
        if let Some(id) = eplanet.rock_mesh_id {
            if let Ok(mut bm) = bms.get_mut(id) {
                bm.points = mesh_points;
            }
        }

        // Then update each of the field meshes
        for field in eplanet.fields.iter() {
            let mut mesh_points = vec![];
            for pid in field.field_points.iter() {
                if let Ok((epoint, mv, parent)) = points.get(*pid) {
                    match epoint.kind {
                        EPointKind::Rock => {
                            mesh_points.push(mv.pos.truncate());
                        }
                        EPointKind::Field => {
                            let (_, parent_mv, _) = points.get(parent.get()).unwrap();
                            mesh_points.push(parent_mv.pos.truncate() + mv.pos.truncate());
                        }
                        EPointKind::Wild => (),
                    }
                }
            }
            if let Ok(mut bm) = bms.get_mut(field.mesh_id) {
                bm.points = mesh_points;
                bm.scroll = field.dir / 4.0;
            }
        }
    }
}

pub(super) fn draw_field_parents(
    mut gzs: Gizmos,
    points: Query<(&EPoint, &GlobalTransform, &Parent)>,
) {
    for (epoint, gt, parent) in points.iter() {
        if epoint.kind == EPointKind::Field {
            let Ok((_, pgt, _)) = points.get(parent.get()) else {
                continue;
            };
            gzs.line_2d(
                gt.translation().truncate(),
                pgt.translation().truncate(),
                Color::WHITE,
            );
        }
    }
}
