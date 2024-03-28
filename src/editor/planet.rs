use bevy::{prelude::*, utils::petgraph::visit::NodeRef};

use crate::{
    drawing::{
        bordered_mesh::BorderedMesh,
        layering::sprite_layer,
        mesh::{outline_points, ScrollSprite},
        sprite_mat::SpriteMaterial,
    },
    editor::point::EPointKind,
    input::MouseState,
    meta::game_state::{EditingMode, GameState, SetGameState},
    physics::dyno::IntMoveable,
};

use super::point::{EPoint, EPointBundle};

pub(super) struct EPlanetField {
    pub field_points: Vec<Entity>,
    pub mesh_id: Entity,
    dir: Vec2,
}

#[derive(Component, Default)]
pub(super) struct EPlanet {
    pub rock_points: Vec<Entity>,
    pub rock_mesh_id: Option<Entity>,
    pub fields: Vec<EPlanetField>,
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
pub(super) fn redo_field(
    mut commands: Commands,
    mut eplanets: Query<&mut EPlanet>,
    points: Query<(Entity, &IntMoveable), With<EPoint>>,
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
        let fp1 = new_poses[ix];
        let fp2 = new_poses[next_ix];
        let parent1 = points.get(eplanet.rock_points[ix]).unwrap();
        let parent2 = points.get(eplanet.rock_points[next_ix]).unwrap();
        let mut fp1_id = planet_id;
        let mut fp2_id = planet_id;
        commands.entity(parent1.0).with_children(|parent| {
            fp1_id = EPointBundle::spawn(
                parent,
                &asset_server,
                fp1 - parent1.1.pos.truncate(),
                EPointKind::Field,
            );
        });
        commands.entity(parent2.0).with_children(|parent| {
            fp2_id = EPointBundle::spawn(
                parent,
                &asset_server,
                fp2 - parent2.1.pos.truncate(),
                EPointKind::Field,
            );
        });
        let mesh_points = vec![parent2.1.pos.truncate(), parent1.1.pos.truncate(), fp1, fp2];
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
                // This order preserves: 0,1 are rock, 2,3 are field
                field_points: vec![parent2.id(), parent1.id(), fp1_id, fp2_id],
                mesh_id,
                dir: Vec2::ONE,
            };
            eplanet.fields.push(field);
        });
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
                    }
                }
            }
            if let Ok(mut bm) = bms.get_mut(field.mesh_id) {
                bm.points = mesh_points;
                bm.scroll = Vec2::new(1.0 / 24.0, 0.0);
            }
        }
    }
}
