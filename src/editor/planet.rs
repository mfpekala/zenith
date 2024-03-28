use bevy::prelude::*;

use crate::{
    drawing::{bordered_mesh::BorderedMesh, layering::sprite_layer, sprite_mat::SpriteMaterial},
    input::MouseState,
    meta::game_state::{EditingMode, GameState, SetGameState},
    physics::dyno::IntMoveable,
};

use super::point::Point;

pub(super) struct EPlanetField {
    pub field_points: Vec<Entity>,
    dir: Vec2,
}

#[derive(Component, Default)]
pub(super) struct EPlanet {
    pub rock_points: Vec<Entity>,
    pub fields: Vec<EPlanetField>,
}

#[derive(Bundle)]
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
        let entity = commands
            .spawn(EPlanetBundle {
                eplanet: EPlanet::default(),
                spatial: SpatialBundle::from_transform(Transform::from_translation(
                    pos.as_vec2().extend(0.0),
                )),
                moveable: IntMoveable::new(pos.extend(0)),
            })
            .with_children(|parent| {
                BorderedMesh::spawn_easy(
                    parent,
                    asset_server,
                    meshes,
                    mats,
                    vec![
                        IVec2::new(-20, -20),
                        IVec2::new(-20, 20),
                        IVec2::new(20, 20),
                        IVec2::new(20, -20),
                    ],
                    ("textures/play_inner.png", UVec2::new(36, 36)),
                    Some(("textures/play_outer.png", UVec2::new(36, 36))),
                    Some(3.0),
                    sprite_layer(),
                );
            })
            .id();
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
    points: Query<(&Point, &Parent)>,
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
    if mode != EditingMode::Free {
        return;
    }
    if keyboard.just_pressed(KeyCode::KeyF) {
        match mode {
            EditingMode::Free => (),
            EditingMode::CreatingPlanet(_) | EditingMode::EditingPlanet(_) => {
                gs_writer.send(SetGameState(EditingMode::Free.to_game_state()));
            }
        }
    }
}

pub(super) fn drive_planet_meshes(
    points: Query<&IntMoveable, With<Point>>,
    eplanets: Query<(&EPlanet, &Children)>,
    mut bms: Query<&mut BorderedMesh>,
) {
    for (eplanet, children) in eplanets.iter() {
        let mut mesh_points = vec![];
        for pid in eplanet.rock_points.iter() {
            if let Ok(mv) = points.get(*pid) {
                mesh_points.push(mv.pos.truncate());
            }
        }

        for child in children {
            let Ok(mut bm) = bms.get_mut(*child) else {
                continue;
            };
            bm.points = mesh_points;
            break;
        }
    }
}
