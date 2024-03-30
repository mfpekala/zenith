use std::cmp::Ordering;

use bevy::{prelude::*, utils::hashbrown::HashMap};

use crate::{
    drawing::{
        bordered_mesh::BorderedMesh, layering::sprite_layer, mesh::outline_points,
        sprite_mat::SpriteMaterial,
    },
    editor::point::EPointKind,
    input::MouseState,
    meta::game_state::{EditingMode, GameState, SetGameState},
    physics::dyno::IntMoveable,
};

use super::point::{ChangeEPointKind, EPoint, EPointBundle};

pub(super) struct EPlanetField {
    pub field_points: Vec<Entity>,
    pub mesh_id: Entity,
    dir: Vec2,
}

#[derive(Component, Default)]
pub(super) struct EPlanet {
    pub rock_points: Vec<Entity>,
    pub rock_mesh_id: Option<Entity>,
    pub wild_points: Vec<Entity>,
    pub fields: Vec<EPlanetField>,
}

#[derive(Component)]
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
            spatial: SpatialBundle::from_transform(Transform::from_translation(
                pos.as_vec2().extend(0.0),
            )),
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
    mut points: Query<(Entity, &IntMoveable, &mut EPoint)>,
    gs: Res<GameState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    asset_server: Res<AssetServer>,
    mut mats: ResMut<Assets<SpriteMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
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
    for field in eplanet.fields.iter() {
        for id in field.field_points.iter() {
            if points.get(*id).is_err() || eplanet.rock_points.iter().any(|rid| rid == id) {
                // Ignore the rock points of the field
                continue;
            }
            commands.entity(*id).despawn_recursive();
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
    let new_poses: Vec<IVec2> = outline_points(&poses, 10.0)
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
        // let mesh_points = vec![parent2.1.pos.truncate(), parent1.1.pos.truncate(), fp1, fp2];
        // commands.entity(planet_id).with_children(|parent| {
        //     let mesh_id = BorderedMesh::spawn_easy(
        //         parent,
        //         &asset_server,
        //         &mut meshes,
        //         &mut mats,
        //         mesh_points,
        //         ("sprites/field/field_bg.png", UVec2::new(12, 12)),
        //         None,
        //         None,
        //         sprite_layer(),
        //     );
        //     let field = EPlanetField {
        //         // This order preserves: 0,1 are rock, 2,3 are field
        //         field_points: vec![parent2.id(), parent1.id(), fp1_id, fp2_id],
        //         mesh_id,
        //         dir: Vec2::ONE,
        //     };
        //     eplanet.fields.push(field);
        // });
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
    let mut eplanet = eplanets.get_mut(planet_id).unwrap();

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
                    let (_, _, mv, _, _) = points.get(parent).unwrap();
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
        let id_order = points_n_ids
            .clone()
            .into_iter()
            .map(|thing| thing.0)
            .collect();
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
            );
            let field = EPlanetField {
                field_points: id_order,
                mesh_id,
                dir: Vec2::ONE,
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
            let old_pos = mv.pos;
            mv.pos = old_pos - dad_pos.extend(0);
            commands
                .entity(id)
                .insert(ChangeEPointKind(EPointKind::Field));
        }
    }

    // Don't have enough time to implement so just scaffolding now

    // Get the editing planet

    // Be lazy: just start by making one pass through all points to get a list of
    // all relevant u32s

    // For each such u32...
    // Get it's position (relative to PLANET (if the point is already a field point, this is diff))
    // Make the mesh and EPlanetField struct. Add to eplanet

    // DO THIS AT THE END SEPARATELY
    // If the point is wild, get the nearest rock point, and make it a field point
    // ^ If it's already a field point, we don't need to do anything
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
            mv.rem += mult * change;
        }
    }
}

/// If multiple points from the same field are selected, delete that field
/// Turns points into wild points if needed
pub(super) fn remove_field() {}

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
                bm.scroll = Vec2::new(6.0 / 24.0, 6.0 / 24.0);
            }
        }
    }
}
