use super::{
    draggable::Draggable,
    editable_goal::EditableGoal,
    editable_point::{destroy_points, EditablePoint, EditablePointBundle},
    editable_rock::{EditableRock, EditableRockBundle},
    editable_starting_point::EditableStartingPoint,
    entered_editing, entered_testing, is_editing, is_editing_helper, is_testing_helper,
    left_editing, left_testing,
};
use crate::{
    camera::CameraMode,
    environment::{
        field::Field,
        goal::{Goal, GoalBundle},
        planet::spawn_planet,
        rock::{Rock, RockBundle,  RockResources},
    },
    input::{MouseState, SetCameraModeEvent},
    meta::game_state::{
        in_editor, EditingMode, EditingState, EditorState, GameState, MetaState, SetGameState,
    },
    physics::Dyno,
    ship::ShipBundle,
};
use bevy::prelude::*;

pub fn editing_state_machine(
    gs: Res<GameState>,
    mouse_state: Res<MouseState>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    mut editable_rocks: Query<&mut EditableRock>,
    center_points: Query<(Entity, &Transform, &Draggable), With<EditableRock>>,
    exterior_points: Query<
        (Entity, &Transform, &Draggable),
        (With<EditablePoint>, Without<EditableRock>),
    >,
    mut gs_writer: EventWriter<SetGameState>,
) {
    let editing_state = match gs.meta {
        MetaState::Editor(EditorState::Editing(state)) => state,
        _ => return,
    };
    match editing_state.mode {
        EditingMode::Free => {
            if mouse_buttons.just_pressed(MouseButton::Right) {
                // If we right click, start creating a new rock
                let point_id = commands
                    .spawn(EditablePointBundle::new(mouse_state.world_pos))
                    .id();
                let editable_rock_id = commands
                    .spawn(EditableRockBundle::from_single_point(
                        point_id,
                        mouse_state.world_pos,
                    ))
                    .id();
                let new_editing_state = EditingState {
                    mode: EditingMode::CreatingRock(editable_rock_id),
                    paused: false,
                };
                gs_writer.send(SetGameState(new_editing_state.to_game_state()));
            } else if mouse_buttons.just_pressed(MouseButton::Left) {
                // If we left click, see if there's an existing rock to focus on
                for (cid, ctran, cdrag) in center_points.iter() {
                    if cdrag.is_mouse_over(ctran.translation.truncate(), &mouse_state) {
                        // Start editing this rock
                        let new_editing_state = EditingState {
                            mode: EditingMode::EditingRock(cid),
                            paused: false,
                        };
                        gs_writer.send(SetGameState(new_editing_state.to_game_state()));
                        return;
                    }
                }
            }
        }
        EditingMode::CreatingRock(id) => {
            let Ok(mut editing_rock) = editable_rocks.get_mut(id) else {
                // The rock doesn't exist anymore
                let new_editing_state = EditingState {
                    mode: EditingMode::Free,
                    paused: false,
                };
                gs_writer.send(SetGameState(new_editing_state.to_game_state()));
                return;
            };
            let Some(first_ext_id) = editing_rock.points.get(0) else {
                // The rock doesn't exist anymore
                let new_editing_state = EditingState {
                    mode: EditingMode::Free,
                    paused: false,
                };
                gs_writer.send(SetGameState(new_editing_state.to_game_state()));
                return;
            };
            let (_, first_ext_tran, first_ext_drag) = exterior_points.get(*first_ext_id).unwrap();
            if mouse_buttons.just_pressed(MouseButton::Right) {
                if first_ext_drag.is_mouse_over(first_ext_tran.translation.truncate(), &mouse_state)
                {
                    if editing_rock.points.len() < 2 {
                        // All rocks need at least two points
                        return;
                    }
                    // If we right click the first point, close it and go to editing
                    editing_rock.closed = true;
                    let new_editing_state = EditingState {
                        mode: EditingMode::EditingRock(id),
                        paused: false,
                    };
                    let Ok((_, first_tran, _)) = exterior_points.get(editing_rock.points[0]) else {
                        return;
                    };
                    let Ok((_, last_tran, _)) = exterior_points.get(editing_rock.points[1]) else {
                        return;
                    };
                    // This logic calculates where to place the gravity_point_loc
                    let normal = (last_tran.translation.truncate()
                        - first_tran.translation.truncate())
                    .perp()
                    .normalize();
                    let gravity_point_loc = first_tran.translation.truncate() + normal * 50.0;
                    let gpl_id = commands
                        .spawn(EditablePointBundle::new(gravity_point_loc))
                        .id();
                    editing_rock.gravity_reach_point = Some(gpl_id);
                    gs_writer.send(SetGameState(new_editing_state.to_game_state()));
                } else {
                    // If we right click anywhere else, just make a new point
                    let point_id = commands
                        .spawn(EditablePointBundle::new(mouse_state.world_pos))
                        .id();
                    editing_rock.points.push(point_id);
                }
            }
        }
        EditingMode::EditingRock(id) => {
            let Ok(editing_rock) = editable_rocks.get(id) else {
                // The focused rock just got deleted, go back to free
                let new_editing_state = EditingState {
                    mode: EditingMode::Free,
                    paused: false,
                };
                gs_writer.send(SetGameState(new_editing_state.to_game_state()));
                return;
            };
            let (_, editing_tran, editing_draggable) = center_points.get(id).unwrap();
            if mouse_buttons.just_pressed(MouseButton::Left) {
                // If this click was on any of my points, stay in this state
                if editing_draggable
                    .is_mouse_over(editing_tran.translation.truncate(), &mouse_state)
                {
                    return;
                }
                for ext_id in editing_rock.points.iter() {
                    let (_, ex_tran, ex_drag) = exterior_points.get(ext_id.clone()).unwrap();
                    if ex_drag.is_mouse_over(ex_tran.translation.truncate(), &mouse_state) {
                        return;
                    }
                }
                if let Some(reach_id) = editing_rock.gravity_reach_point {
                    let (_, ex_tran, ex_drag) = exterior_points.get(reach_id).unwrap();
                    if ex_drag.is_mouse_over(ex_tran.translation.truncate(), &mouse_state) {
                        return;
                    }
                }
                // Otherwise, default back to free
                let new_editing_state = EditingState {
                    mode: EditingMode::Free,
                    paused: false,
                };
                gs_writer.send(SetGameState(new_editing_state.to_game_state()));
            }
        }
    }
}

fn watch_for_edit_test_switch(
    keys: Res<ButtonInput<KeyCode>>,
    gs: Res<GameState>,
    mut gs_writer: EventWriter<SetGameState>,
) {
    let mut set_action: Option<SetGameState> = None;
    if keys.just_pressed(KeyCode::Enter) {
        if !is_testing_helper(&gs) {
            set_action = Some(SetGameState(GameState {
                meta: MetaState::Editor(EditorState::Testing),
            }));
        }
    } else if keys.just_pressed(KeyCode::Escape) {
        if !is_editing_helper(&gs) {
            set_action = Some(SetGameState(GameState {
                meta: MetaState::Editor(EditorState::Editing(EditingState {
                    mode: EditingMode::Free,
                    paused: false,
                })),
            }))
        }
    }
    if let Some(event) = set_action {
        gs_writer.send(event);
    }
}

fn start_testing(
    mut commands: Commands,
    mut camera_switch_writer: EventWriter<SetCameraModeEvent>,
    erocks: Query<(&EditableRock, &Transform)>,
    epoints: Query<&Transform, With<EditablePoint>>,
    rock_resources: Res<RockResources>,
    estart: Query<&Transform, With<EditableStartingPoint>>,
    egoal: Query<&Transform, With<EditableGoal>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    camera_switch_writer.send(SetCameraModeEvent {
        mode: CameraMode::Follow,
    });
    let ship = ShipBundle::new(estart.single().translation.truncate(), 16.0);
    commands.spawn(ship);
    GoalBundle::spawn(egoal.single().translation.truncate(), &mut commands);
    for (erock, tran) in erocks.iter() {
        let (rock, reach) =
            erock.to_rock_n_reach(&epoints, tran.translation.truncate(), &rock_resources);
        let base_pos = tran.translation.truncate();
        match reach {
            Some(reach) => {
                spawn_planet(&mut commands, base_pos, rock, reach, 0.06, &mut meshes);
            }
            None => {
                RockBundle::spawn(&mut commands, base_pos, rock, &mut meshes);
            }
        }
    }
}

fn stop_testing(
    dyno_ids: Query<Entity, With<Dyno>>,
    goal_ids: Query<Entity, With<Goal>>,
    rock_ids: Query<Entity, With<Rock>>,
    field_ids: Query<Entity, With<Field>>,
    mut commands: Commands,
) {
    for id in dyno_ids.iter() {
        commands.entity(id).despawn_recursive();
    }
    for id in goal_ids.iter() {
        commands.entity(id).despawn_recursive();
    }
    for id in rock_ids.iter() {
        commands.entity(id).despawn_recursive();
    }
    for id in field_ids.iter() {
        commands.entity(id).despawn_recursive();
    }
}

fn start_editing(
    mut _commands: Commands,
    mut camera_switch_writer: EventWriter<SetCameraModeEvent>,
    mut gs_writer: EventWriter<SetGameState>,
) {
    // Start in free editing mode
    let new_editing_state = EditingState {
        mode: EditingMode::Free,
        paused: false,
    };
    gs_writer.send(SetGameState(new_editing_state.to_game_state()));
    // Free the camera too
    camera_switch_writer.send(SetCameraModeEvent {
        mode: CameraMode::Free,
    });
}

fn stop_editing(mut _commands: Commands) {}

pub fn register_editor_state_machine(app: &mut App) {
    app.add_systems(PreUpdate, watch_for_edit_test_switch.run_if(in_editor));
    app.add_systems(Update, start_testing.run_if(entered_testing));
    app.add_systems(Update, stop_testing.run_if(left_testing));
    app.add_systems(Update, start_editing.run_if(entered_editing));
    app.add_systems(Update, stop_editing.run_if(left_editing));
    app.add_systems(
        Update,
        editing_state_machine
            .run_if(is_editing)
            .after(destroy_points),
    );
}
