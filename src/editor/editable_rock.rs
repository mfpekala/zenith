use super::{
    draggable::{handle_draggables, Draggable},
    editable_point::{destroy_points, EditablePoint},
    is_editing,
};
use crate::{
    drawing::hollow::{CircleMarker, HollowDrawable},
    environment::{
        field::Field,
        rock::{Rock, RockFeatures, RockResources, RockType},
    },
    input::MouseState,
    math::MathLine,
    meta::game_state::{EditingMode, EditorState, GameState, MetaState},
};
use bevy::prelude::*;

#[derive(Component)]
pub struct EditableRock {
    pub closed: bool,
    pub gravity_strength: Option<f32>,
    pub gravity_reach_point: Option<Entity>,
    pub points: Vec<Entity>,
}
impl EditableRock {
    pub fn to_rock(
        &self,
        epoints: &Query<&Transform, With<EditablePoint>>,
        offset: Vec2,
        features: RockFeatures,
    ) -> Rock {
        let mut points = vec![];
        for pid in self.points.iter() {
            let tran = epoints.get(pid.clone()).unwrap();
            points.push(tran.translation.truncate() - offset);
        }
        Rock::new(points, features)
    }

    pub fn despawn(&mut self, my_pid: Entity, commands: &mut Commands) {
        for pid in self.points.iter() {
            commands.entity(pid.clone()).despawn_recursive();
        }
        if let Some(rpid) = self.gravity_reach_point {
            commands.entity(rpid).despawn_recursive();
        }
        self.points = vec![];
        self.gravity_reach_point = None;
        commands.entity(my_pid).despawn_recursive();
    }

    /// Helper function to return a vector of exterior points including the reach point if it exists
    pub fn get_all_ext_points(&self) -> Vec<Entity> {
        match self.gravity_reach_point {
            Some(pid) => {
                let mut tmp = self.points.clone();
                tmp.extend([pid].into_iter());
                tmp
            }
            None => self.points.clone(),
        }
    }
}

#[derive(Bundle)]
pub struct EditableRockBundle {
    pub erock: EditableRock,
    pub editable_point: EditablePoint,
    pub draggable: Draggable,
    pub circle: CircleMarker,
    pub spatial: SpatialBundle,
}
impl EditableRockBundle {
    pub fn from_single_point(id: Entity, pos: Vec2) -> Self {
        Self {
            erock: EditableRock {
                closed: false,
                gravity_reach_point: None,
                gravity_strength: None,
                points: vec![id],
            },
            editable_point: EditablePoint { is_focused: true },
            draggable: Draggable::new(10.0),
            circle: CircleMarker::new(10.0, Color::SEA_GREEN),
            spatial: SpatialBundle::from_transform(Transform::from_translation(pos.extend(0.0))),
        }
    }
}

fn update_centers(
    mut erocks: Query<(&EditableRock, &Draggable, &mut Transform), With<EditableRock>>,
    mut epoints: Query<&mut Transform, (With<EditablePoint>, Without<EditableRock>)>,
    mouse_state: Res<MouseState>,
) {
    for (erock, edrag, mut etran) in erocks.iter_mut() {
        if edrag.is_dragging {
            // We are moving the whole rock
            let diff = mouse_state.world_pos - etran.translation.truncate();
            etran.translation += diff.extend(0.0);
            for pid in erock.points.iter() {
                let Ok(mut ptran) = epoints.get_mut(pid.clone()) else {
                    continue;
                };
                ptran.translation += diff.extend(0.0);
            }
            if let Some(pid) = erock.gravity_reach_point {
                let Ok(mut ptran) = epoints.get_mut(pid.clone()) else {
                    continue;
                };
                ptran.translation += diff.extend(0.0);
            }
        } else {
            // Otherwise correct the center
            let mut total = Vec2::ZERO;
            let mut count = 0;
            for pid in erock.points.iter() {
                let Ok(ptran) = epoints.get(pid.clone()) else {
                    continue;
                };
                total += ptran.translation.truncate();
                count += 1;
            }
            if count > 0 {
                total = total / (count as f32);
            }
            etran.translation = total.extend(0.0);
        }
    }
}

fn enable_only_active_rock(
    gs: Res<GameState>,
    erocks: Query<(Entity, &EditableRock)>,
    mut epoints: Query<
        (&mut EditablePoint, &mut Draggable),
        (With<EditablePoint>, Without<EditableRock>),
    >,
    mut center_points: Query<&mut EditablePoint, With<EditableRock>>,
) {
    let MetaState::Editor(EditorState::Editing(state)) = gs.meta else {
        return;
    };
    let rid = match state.mode {
        EditingMode::CreatingRock(rid) | EditingMode::EditingRock(rid) => Some(rid),
        _ => None,
    };
    for (id, rock) in erocks.iter() {
        for pid in rock.get_all_ext_points() {
            let Ok((mut epoint, mut edrag)) = epoints.get_mut(pid) else {
                continue;
            };
            if Some(id) == rid {
                // Yay! The chosen rock
                epoint.is_focused = true;
                edrag.enabled = true;
            } else {
                // Nope
                epoint.is_focused = false;
                edrag.enabled = false;
            }
        }
        // Finally mark the center accordingly (NOTE: It's always enabled so it can be focused)
        let Ok(mut cpoint) = center_points.get_mut(id) else {
            continue;
        };
        if Some(id) == rid {
            cpoint.is_focused = true;
        } else {
            cpoint.is_focused = false;
        }
    }
}

fn snap_reach_point_to_line(
    gs: Res<GameState>,
    mut erocks: Query<&mut EditableRock>,
    mut epoints: Query<&mut Transform, (With<EditablePoint>, Without<EditableRock>)>,
) {
    let MetaState::Editor(EditorState::Editing(state)) = gs.meta else {
        return;
    };
    let EditingMode::EditingRock(rid) = state.mode else {
        return;
    };
    let Ok(editing_rock) = erocks.get_mut(rid) else {
        return;
    };
    let Some(gpid) = editing_rock.gravity_reach_point else {
        return;
    };

    let pointn1 = epoints
        .get(editing_rock.points.last().unwrap().clone())
        .unwrap()
        .translation
        .truncate();
    let point0 = epoints
        .get(editing_rock.points.first().unwrap().clone())
        .unwrap()
        .translation
        .truncate();
    let pointp1 = epoints
        .get(editing_rock.points[1])
        .unwrap()
        .translation
        .truncate();
    let mut gravity_point = epoints.get_mut(gpid).unwrap();
    let diff1 = point0 - pointn1;
    let diff2 = pointp1 - point0;
    let perp = (diff1.normalize() + diff2.normalize()).normalize().perp();
    let line = MathLine {
        p1: point0,
        p2: point0 + perp,
    };
    let closest = line.closest_point_on_line(&gravity_point.translation.truncate());
    gravity_point.translation = closest.extend(0.0);
}

fn draw_editable_rocks(
    erocks: Query<(&EditableRock, &Transform)>,
    epoints: Query<&Transform, With<EditablePoint>>,
    mut gz: Gizmos,
    rock_resources: Res<RockResources>,
) {
    for (rock, tran) in erocks.iter() {
        if rock.points.len() < 3 {
            continue;
        }
        // Draw the standard lines
        for ix in 0..rock.points.len().saturating_sub(1) {
            let Ok(this_tran) = epoints.get(rock.points[ix]) else {
                continue;
            };
            let Ok(next_tran) = epoints.get(rock.points[ix + 1]) else {
                continue;
            };
            gz.line_2d(
                this_tran.translation.truncate(),
                next_tran.translation.truncate(),
                Color::TURQUOISE,
            );
        }
        // If closed, draw the last line
        if rock.closed {
            let first_id = rock.points.first().unwrap();
            let first_tran = epoints.get(first_id.clone()).unwrap();
            let last_id = rock.points.last().unwrap();
            let last_tran = epoints.get(last_id.clone()).unwrap();
            gz.line_2d(
                last_tran.translation.truncate(),
                first_tran.translation.truncate(),
                Color::TURQUOISE,
            );
        }
        // If has a gravity point, draw the gravity bounds
        if let Some(rp) = rock.gravity_reach_point {
            let first_point = epoints.get(rock.points[0]).unwrap();
            let Ok(rp_point) = epoints.get(rp) else {
                continue;
            };
            let dist = first_point
                .translation
                .truncate()
                .distance(rp_point.translation.truncate());
            let as_rock = rock.to_rock(
                &epoints,
                tran.translation.truncate(),
                rock_resources.get_type(RockType::Normal),
            );
            let show_fields = Field::uniform_around_rock(&as_rock, dist, 1.0);
            for field in show_fields {
                field.draw_hollow(tran.translation.truncate(), &mut gz);
            }
        }
    }
}

pub fn register_editable_rocks(app: &mut App) {
    app.add_systems(
        Update,
        update_centers.run_if(is_editing).before(handle_draggables),
    );
    app.add_systems(
        Update,
        draw_editable_rocks
            .run_if(is_editing)
            .after(handle_draggables), // .after(destroy_points),
    );
    app.add_systems(
        Update,
        snap_reach_point_to_line
            .run_if(is_editing)
            .after(handle_draggables)
            .after(destroy_points),
    );
    app.add_systems(
        Update,
        enable_only_active_rock
            .run_if(is_editing)
            .after(handle_draggables),
    );
}
