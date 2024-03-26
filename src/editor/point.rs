use bevy::{prelude::*, render::view::RenderLayers};

use crate::{
    drawing::layering::sprite_layer,
    input::MouseState,
    meta::game_state::{
        EditingMode, EditingState, EditorState, GameState, MetaState, SetGameState,
    },
    physics::dyno::IntMoveable,
};

use super::planet::{EPlanet, EPlanetBundle};

#[derive(Component)]
pub struct Point {
    pub is_selected: bool,
    pub drag_offset: Option<IVec2>,
}
impl Point {
    pub fn new() -> Self {
        Self {
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

pub(super) fn spawn_points(
    mut commands: Commands,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mouse_state: Res<MouseState>,
    gs: Res<GameState>,
    mut eplanets: Query<(&mut EPlanet, &IntMoveable)>,
    points: Query<&GlobalTransform, With<Point>>,
    mut gs_writer: EventWriter<SetGameState>,
) {
    let planet_id = match gs.meta {
        MetaState::Editor(state) => match state.get_editing_mode() {
            Some(state) => match state {
                EditingMode::CreatingPlanet(id) => id,
                _ => return,
            },
            _ => return,
        },
        _ => return,
    };
    let (mut eplanet, mv) = eplanets.get_mut(planet_id).unwrap();
    if mouse_buttons.just_pressed(MouseButton::Right) {
        let closing = match eplanet.rock_points.get(0) {
            None => false,
            Some(pid) => {
                if let Ok(gt) = points.get(*pid) {
                    if (mouse_state.world_pos.as_vec2() - gt.translation().truncate())
                        .length_squared()
                        < 16.0
                    {
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
        };
        if closing {
            gs_writer.send(SetGameState(GameState {
                meta: MetaState::Editor(EditorState::Editing(EditingState {
                    mode: EditingMode::EditingPlanet(planet_id),
                })),
            }));
        } else {
            commands.entity(planet_id).with_children(|mut parent| {
                let id = PointBundle::spawn(&mut parent, mouse_state.world_pos - mv.pos.truncate());
                eplanet.rock_points.push(id);
            });
        }
    }
}

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
    for (id, point, _, gt, _) in points.iter() {
        if point.is_selected {
            selected.push(id);
        }
        let size = 4 as u32;
        let overlap_x = mouse_state
            .world_pos
            .x
            .abs_diff(gt.translation().x.round() as i32)
            < size;
        let overlap_y = mouse_state
            .world_pos
            .y
            .abs_diff(gt.translation().y.round() as i32)
            < size;
        if overlap_x && overlap_y {
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
    points: Query<(Entity, &Point)>,
    key_buttons: Res<ButtonInput<KeyCode>>,
) {
    if key_buttons.pressed(KeyCode::Backspace) {
        for (id, p) in points.iter() {
            if p.is_selected {
                commands.entity(id).despawn_recursive();
            }
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
