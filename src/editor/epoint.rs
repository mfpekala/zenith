use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    drawing::animation::{AnimationManager, MultiAnimationManager, SpriteInfo},
    input::MouseState,
    meta::game_state::EditingMode,
    physics::dyno::{IntMoveable, IntMoveableBundle},
};

#[derive(Component, Serialize, Deserialize, bevy::reflect::TypePath, Debug, Clone)]
pub struct EPoint {
    /// Points are squares. This is the length of a side
    pub size: f32,
}

/// Given to point entities that are hovered
#[derive(Component, Serialize, Deserialize, bevy::reflect::TypePath, Debug, Clone)]
pub struct EHovered;

/// Given to point entities that are selected
#[derive(Component, Serialize, Deserialize, bevy::reflect::TypePath, Debug, Clone)]
pub struct ESelected {
    pub order: u32,
    pub offset: Vec2,
}

#[derive(Bundle)]
struct EPointBundle {
    name: Name,
    point: EPoint,
    mv: IntMoveableBundle,
}

/// Spawns a new point
pub(super) fn spawn_point(
    In((emode, world_pos)): In<(EditingMode, IVec2)>,
    mut commands: Commands,
) {
    match emode {
        EditingMode::Free => {
            let point_bund = EPointBundle {
                name: Name::new("point"),
                point: EPoint { size: 6.0 },
                mv: IntMoveableBundle::new(world_pos.extend(0)),
            };
            let anim = AnimationManager::from_static_pairs(vec![
                (
                    "none",
                    SpriteInfo {
                        path: "sprites/editor/point.png".into(),
                        size: UVec2::new(8, 8),
                        ..default()
                    },
                ),
                (
                    "hovered",
                    SpriteInfo {
                        path: "sprites/editor/point_hovered.png".into(),
                        size: UVec2::new(8, 8),
                        ..default()
                    },
                ),
                (
                    "selected",
                    SpriteInfo {
                        path: "sprites/editor/point_selected.png".into(),
                        size: UVec2::new(8, 8),
                        ..default()
                    },
                ),
            ]);
            let multi = MultiAnimationManager::well_lit(anim);
            commands.spawn((point_bund, multi));
        }
        EditingMode::CreatingPlanet(eid) => {
            todo!("CreatingPlanet spawn_point");
        }
        EditingMode::EditingPlanet(eid) => {
            todo!("EditingPlanet spawn_point");
        }
    }
}

/// Deletes a vector of points by Entity
pub(super) fn delete_points(In(eids): In<Vec<Entity>>, mut commands: Commands) {
    for eid in eids {
        if let Some(commands) = commands.get_entity(eid) {
            commands.despawn_recursive();
        }
    }
}

/// Inserts or removes the EPointHovered
pub(super) fn hover_points(
    mouse: Res<MouseState>,
    points_q: Query<(Entity, &EPoint, &IntMoveable)>,
    mut commands: Commands,
) {
    for (eid, point, mv) in points_q.iter() {
        let x_diff = mv.get_ipos().x - mouse.world_pos.x;
        let y_diff = mv.get_ipos().y - mouse.world_pos.y;
        if (x_diff.abs() as f32) < point.size / 2.0 && (y_diff.abs() as f32) < point.size / 2.0 {
            commands.entity(eid).insert(EHovered);
        } else {
            commands.entity(eid).remove::<EHovered>();
        }
    }
}

/// Select/deselect points
pub(super) fn select_points(
    mouse: Res<MouseState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    points_q: Query<(Entity, Option<&EHovered>, Option<&ESelected>, &IntMoveable), With<EPoint>>,
    mut commands: Commands,
) {
    if !mouse.button_input.just_pressed(MouseButton::Left) {
        return;
    }
    let mut next_order = 0;
    for data in points_q.iter() {
        if let Some(sel) = data.2 {
            next_order = next_order.max(sel.order + 1);
        }
    }
    if keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
        // Toggle only the hovered points
        for data in points_q.iter() {
            if data.1.is_none() {
                // Not hovered
                continue;
            }
            if data.2.is_none() {
                // Hovered and not selected, make selected
                commands.entity(data.0).insert(ESelected {
                    order: next_order,
                    offset: (data.3.get_ipos().truncate() - mouse.world_pos).as_vec2(),
                });
                next_order += 1;
            } else {
                // Hovered and selected, make deselected
                commands.entity(data.0).remove::<ESelected>();
            }
        }
    } else {
        // All hovered points become selected. The rest become deselected
        next_order = 0;
        for data in points_q.iter() {
            if data.1.is_none() {
                // Not hovered
                commands.entity(data.0).remove::<ESelected>();
            } else {
                // Hovered
                commands.entity(data.0).insert(ESelected {
                    order: next_order,
                    offset: (data.3.get_ipos().truncate() - mouse.world_pos).as_vec2(),
                });
                next_order += 1;
            }
        }
    }
}

pub(super) fn animate_points(
    mut points_q: Query<
        (
            &mut MultiAnimationManager,
            Option<&EHovered>,
            Option<&ESelected>,
        ),
        With<EPoint>,
    >,
) {
    for (mut multi, hovered, selected) in points_q.iter_mut() {
        let mut set_key = |key| {
            // println!("multi.map: {:?}", multi.map);
            let core = multi.map.get_mut("core").unwrap();
            core.set_key(key);
            let light = multi.map.get_mut("light").unwrap();
            light.set_key(key);
        };
        if hovered.is_none() && selected.is_none() {
            set_key("none");
        } else if hovered.is_some() && selected.is_none() {
            set_key("hovered");
        } else {
            set_key("selected");
        }
    }
}

pub(super) fn move_points(
    mouse: Res<MouseState>,
    mut points_q: Query<(&mut IntMoveable, &mut ESelected), With<EPoint>>,
) {
    if mouse.button_input.pressed(MouseButton::Left) {
        for (mut mv, sel) in points_q.iter_mut() {
            let two_d = mouse.world_pos.as_vec2() + sel.offset;
            mv.fpos = two_d.extend(mv.fpos.z);
        }
    } else {
        for (mv, mut sel) in points_q.iter_mut() {
            sel.offset = mv.fpos.truncate() - mouse.world_pos.as_vec2();
        }
    }
}
