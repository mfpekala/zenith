use bevy::{prelude::*, render::view::RenderLayers};

use crate::{
    drawing::layering::sprite_layer,
    input::MouseState,
    meta::game_state::{
        EditingMode, EditingState, EditorState, GameState, MetaState, SetGameState,
    },
    physics::dyno::IntMoveable,
};

use super::planet::EPlanet;

#[derive(Component)]
pub struct Point {
    pub size: u32,
    pub is_hovered: bool,
    pub is_selected: bool,
    pub drag_offset: Option<IVec2>,
}
impl Point {
    pub fn new() -> Self {
        Self {
            size: 4,
            is_hovered: false,
            is_selected: false,
            drag_offset: None,
        }
    }
}

#[derive(Component)]
pub(super) struct SelectMarker;

#[derive(Bundle)]
pub struct PointBundle {
    pub point: Point,
    pub moveable: IntMoveable,
    pub sprite: SpriteBundle,
    pub render_layer: RenderLayers,
}
impl PointBundle {
    fn border_growth() -> f32 {
        1.5
    }

    fn new(pos: IVec2, size: u32, color: Color) -> Self {
        Self {
            point: Point::new(),
            moveable: IntMoveable::new(pos.extend(2)),
            sprite: SpriteBundle {
                sprite: Sprite { color, ..default() },
                transform: Transform {
                    scale: (Vec3::ONE * size as f32),
                    translation: pos.as_vec2().extend(0.1),
                    ..default()
                },
                ..default()
            },
            render_layer: sprite_layer(),
        }
    }

    pub fn spawn(commands: &mut ChildBuilder, pos: IVec2) -> Entity {
        let size = 4;
        commands
            .spawn(Self::new(pos, size, Color::ANTIQUE_WHITE))
            .with_children(|parent| {
                parent.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::DARK_GRAY,
                            ..default()
                        },
                        transform: Transform {
                            scale: Vec3::ONE * Self::border_growth(),
                            translation: Vec2::ZERO.extend(-0.1),
                            ..default()
                        },
                        ..default()
                    },
                    sprite_layer(),
                ));
            })
            .id()
    }
}

// Simply mark (in each point) whether it is hovered
pub(super) fn hover_points(
    mouse_state: Res<MouseState>,
    mut points: Query<(&GlobalTransform, &mut Point)>,
) {
    for (gt, mut point) in points.iter_mut() {
        let overlap_x = mouse_state
            .world_pos
            .x
            .abs_diff(gt.translation().x.round() as i32)
            < point.size;
        let overlap_y = mouse_state
            .world_pos
            .y
            .abs_diff(gt.translation().y.round() as i32)
            < point.size;
        point.is_hovered = overlap_x && overlap_y;
    }
}

/// What this "really" does is handle right click, which means variations of spawning points
pub(super) fn spawn_points(
    mut commands: Commands,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mouse_state: Res<MouseState>,
    gs: Res<GameState>,
    mut eplanets: Query<(&mut EPlanet, &IntMoveable)>,
    points: Query<(&Point, &IntMoveable)>,
    mut gs_writer: EventWriter<SetGameState>,
) {
    let Some(mode) = gs.get_editing_mode() else {
        return;
    };
    if !mouse_buttons.just_pressed(MouseButton::Right) {
        // See? it just handles right clicks
        return;
    }
    match mode {
        EditingMode::Free => {
            // For now do nothing here
        }
        EditingMode::CreatingPlanet(planet_id) => {
            // Either closes the rock or places a new point
            let (mut eplanet, mv) = eplanets.get_mut(planet_id).unwrap();
            let closing = eplanet.rock_points.len() > 2
                && points.get(eplanet.rock_points[0]).unwrap().0.is_hovered;
            if closing {
                gs_writer.send(SetGameState(GameState {
                    meta: MetaState::Editor(EditorState::Editing(EditingState {
                        mode: EditingMode::EditingPlanet(planet_id),
                    })),
                }));
            } else {
                commands.entity(planet_id).with_children(|mut parent| {
                    let id =
                        PointBundle::spawn(&mut parent, mouse_state.world_pos - mv.pos.truncate());
                    eplanet.rock_points.push(id);
                });
            }
        }
        EditingMode::EditingPlanet(planet_id) => {
            // Adds a new point in the most rational segment
            let (mut eplanet, mv) = eplanets.get_mut(planet_id).unwrap();
            let spawning_at = mouse_state.world_pos - mv.pos.truncate();
            let mut closest_point = None;
            let mut closest_dist = i32::MAX;
            let mut closest_ix = 0;
            for (ix, id) in eplanet.rock_points.iter().enumerate() {
                let (_, mv) = points.get(*id).unwrap();
                let dist = mv.pos.truncate().distance_squared(spawning_at);
                if closest_point.is_none() || dist < closest_dist {
                    closest_point = Some(*id);
                    closest_dist = dist;
                    closest_ix = ix as i32;
                }
            }
            let left_ix = (closest_ix - 1).rem_euclid(eplanet.rock_points.len() as i32);
            let right_ix = (closest_ix + 1).rem_euclid(eplanet.rock_points.len() as i32);
            let left_dist = points
                .get(eplanet.rock_points[left_ix as usize])
                .unwrap()
                .1
                .pos
                .truncate()
                .distance_squared(spawning_at);
            let right_dist = points
                .get(eplanet.rock_points[right_ix as usize])
                .unwrap()
                .1
                .pos
                .truncate()
                .distance_squared(spawning_at);
            let pos = if left_dist < right_dist {
                closest_ix as usize
            } else {
                right_ix as usize
            };
            commands.entity(planet_id).with_children(|mut parent| {
                let id = PointBundle::spawn(&mut parent, mouse_state.world_pos - mv.pos.truncate());
                eplanet.rock_points.insert(pos, id);
            });
        }
    }
}

/// Really just handles left press/release, which usually means selecting/deselecting points
/// NOTE: Does NOT handle changing from editing/creating -> free, see planet_state_input for that
pub(super) fn select_points(
    mut commands: Commands,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mouse_state: Res<MouseState>,
    key_buttons: Res<ButtonInput<KeyCode>>,
    mut points: Query<(
        Entity,
        &mut Point,
        &IntMoveable,
        &GlobalTransform,
        &Children,
    )>,
    select_markers: Query<&SelectMarker>,
) {
    // If there's no press / release, do nothing
    if !mouse_buttons.just_pressed(MouseButton::Left)
        && !mouse_buttons.just_released(MouseButton::Left)
    {
        return;
    }
    // Figure out what points are already selected, and what points are hovered (if any)
    let mut selected = vec![];
    let mut hovered = vec![];
    for (id, point, _, _, _) in points.iter() {
        if point.is_selected {
            selected.push(id);
        }
        if point.is_hovered {
            hovered.push(id);
        }
    }
    // Helper functions
    let select_point = |id: Entity,
                        comms: &mut Commands,
                        q: &mut Query<(
        Entity,
        &mut Point,
        &IntMoveable,
        &GlobalTransform,
        &Children,
    )>| {
        let (_, mut p, mv, gt, children) = q.get_mut(id).unwrap();
        p.is_selected = true;
        let gt2 = IVec2::new(gt.translation().x as i32, gt.translation().y as i32);
        let standard_off = gt2 - mouse_state.world_pos;
        let parent_tran = gt2 - mv.pos.truncate();
        p.drag_offset = Some(standard_off - parent_tran);
        if children.len() <= 1 {
            comms.entity(id).with_children(|parent| {
                parent.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::GOLD,
                            ..default()
                        },
                        transform: Transform {
                            scale: Vec3::ONE * (PointBundle::border_growth() + 0.5),
                            translation: Vec2::ZERO.extend(-0.11),
                            ..default()
                        },
                        ..default()
                    },
                    SelectMarker,
                    sprite_layer(),
                ));
            });
        }
    };
    let deselect_point = |id: Entity,
                          comms: &mut Commands,
                          q: &mut Query<(
        Entity,
        &mut Point,
        &IntMoveable,
        &GlobalTransform,
        &Children,
    )>| {
        let (_, mut p, _, _, children) = q.get_mut(id).unwrap();
        p.is_selected = false;
        p.drag_offset = None;
        for child in children {
            if select_markers.get(*child).is_ok() {
                comms.entity(*child).despawn_recursive();
                comms.entity(id).remove_children(&[*child]);
            }
        }
    };
    // Finally interpret the input
    if mouse_buttons.just_pressed(MouseButton::Left) {
        if !key_buttons.pressed(KeyCode::ShiftLeft) {
            let deselecting = selected
                .clone()
                .into_iter()
                .filter(|p| !hovered.contains(p));
            for id in deselecting {
                deselect_point(id, &mut commands, &mut points);
            }
        } else {
            for id in selected.iter() {
                // Selecting the already selected points restarts their drag with the new offset
                select_point(*id, &mut commands, &mut points);
            }
        }
        for id in hovered {
            select_point(id, &mut commands, &mut points);
        }
    } else if !mouse_buttons.pressed(MouseButton::Left) {
        for id in selected {
            let (_, mut p, _, _, _) = points.get_mut(id).unwrap();
            p.drag_offset = None;
        }
    }
}

pub(super) fn delete_points(
    mut commands: Commands,
    mut eplanets: Query<(Entity, &mut EPlanet)>,
    points: Query<(Entity, &Point, &Parent)>,
    key_buttons: Res<ButtonInput<KeyCode>>,
    gs: Res<GameState>,
    mut gs_writer: EventWriter<SetGameState>,
) {
    if key_buttons.pressed(KeyCode::Backspace) {
        // Despawn the point, and then remove it from it's parent rock/field list
        for (id, p, parent_ref) in points.iter() {
            if p.is_selected {
                commands.entity(id).despawn_recursive();
                let mut parent = eplanets.get_mut(parent_ref.get()).unwrap().1;
                parent.rock_points.retain(|x| *x != id);
                for field in parent.fields.iter_mut() {
                    field.field_points.retain(|x| *x != id);
                }
            }
        }
        // Check to see if any planets have fewer than three points. If so,
        // delete them
        let mut purged_planets = vec![];
        for (planet_id, eplanet) in eplanets.iter() {
            if eplanet.rock_points.len() < 3 {
                purged_planets.push(planet_id);
            }
        }
        for planet_id in purged_planets {
            // Also need to switch to free if we're deleting the creating or editing planet
            match gs.get_editing_mode() {
                Some(EditingMode::CreatingPlanet(id)) | Some(EditingMode::EditingPlanet(id)) => {
                    if planet_id == id {
                        gs_writer.send(SetGameState(EditingMode::Free.to_game_state()));
                    }
                }
                _ => (),
            }
            commands.entity(planet_id).despawn_recursive();
        }
    }
}

pub(super) fn move_points(
    mouse_state: Res<MouseState>,
    mut points: Query<(&Point, &mut IntMoveable)>,
) {
    for (p, mut mv) in points.iter_mut() {
        if let Some(offset) = p.drag_offset {
            mv.pos = (mouse_state.world_pos + offset).extend(mv.pos.z);
        }
    }
}
