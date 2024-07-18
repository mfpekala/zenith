use bevy::prelude::*;

use crate::{
    drawing::animation::{AnimationManager, MultiAnimationManager, SpriteInfo},
    input::MouseState,
    meta::game_state::{EditingMode, SetMetaState},
    physics::dyno::{IntMoveable, IntMoveableBundle},
};

use super::{erock::ERock, transitions::ERootEid};

#[derive(Component, Debug, Clone, Reflect)]
pub struct EPoint {
    /// Points are squares. This is the length of a side
    pub size: f32,
}

/// Anything that requires multiple points
#[derive(Component, Debug, Clone, Reflect)]
pub struct EPointGroup {
    /// Ids of the points in this group
    pub eids: Vec<Entity>,
    /// Using space to simplify access (avoid queries)
    pub poses: Vec<IVec2>,
    /// Minimum number of points that must be in this group. If the group contains
    /// less than this many points, it will be despawned recursively
    pub minimum: u32,
    /// By default a "shiny" animation when all points in this group are selected
    /// This allows you to force it either on or off. Useful for forcing the editing rock
    /// to show up as shiny, even when only modifying one of it's points
    pub force_shiny: Option<bool>,
}
impl EPointGroup {
    /// Helper function to add a new point so I don't accidentally forget to add to poses/eids
    pub fn insert_point(&mut self, ix: usize, eid: Entity, pos: IVec2) {
        self.eids.insert(ix, eid);
        self.poses.insert(ix, pos);
    }
}

/// Given to point entities that are hovered
#[derive(Component, Debug, Clone, Reflect)]
pub struct EHovered;

/// Given to point entities that are selected
#[derive(Component, Debug, Clone, Reflect)]
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
    hover_q: Query<Entity, With<EHovered>>,
    mut rocks_q: Query<&mut EPointGroup, With<ERock>>,
    points_q: Query<&IntMoveable, With<EPoint>>,
    eroot: Res<ERootEid>,
    mut meta_writer: EventWriter<SetMetaState>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    // Handle the edge case where we're closing a rock
    if let EditingMode::CreatingRock(eid) = emode {
        let rock = rocks_q.get(eid).unwrap();
        if !rock.eids.is_empty() && hover_q.contains(rock.eids[0]) {
            meta_writer.send(SetMetaState(EditingMode::EditingRock(eid).to_meta_state()));
            // Don't actually spawn a point
            return;
        }
    }

    // First just spawn a point and record the id
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
    let point_bund = EPointBundle {
        name: Name::new("point"),
        point: EPoint { size: 6.0 },
        mv: IntMoveableBundle::new(world_pos.extend(0)),
    };
    commands.entity(eroot.0).with_children(|eroot| {
        let pc = eroot.spawn((point_bund, multi));
        // Then perform interesting work depending on the editor state
        match emode {
            EditingMode::Free => {
                // Nothing else to do
            }
            EditingMode::CreatingRock(eid) => {
                let mut pg = rocks_q.get_mut(eid).unwrap();
                pg.eids.push(pc.id());
            }
            EditingMode::EditingRock(eid) => {
                if keyboard.pressed(KeyCode::KeyF) {
                    // We're actually just spawning a regular point
                    // Nothing left to do.
                } else {
                    let mut pg = rocks_q.get_mut(eid).unwrap();
                    let mut closest_pos = IVec2::ZERO;
                    let mut closest_dist = i32::MAX;
                    let mut closest_ix = 0;
                    for (ix, pos) in pg.poses.iter().enumerate() {
                        let dist = pos.distance_squared(world_pos);
                        if dist < closest_dist {
                            closest_dist = dist;
                            closest_ix = ix;
                            closest_pos = *pos;
                        }
                    }
                    let anchor_vec = (world_pos - closest_pos).as_vec2();
                    let left_ix = (closest_ix - 1).rem_euclid(pg.eids.len() as usize);
                    let right_ix = (closest_ix + 1).rem_euclid(pg.eids.len() as usize);
                    let left_mv = points_q.get(pg.eids[left_ix]).unwrap();
                    let right_mv = points_q.get(pg.eids[right_ix]).unwrap();
                    let left_vec =
                        (left_mv.fpos.truncate() - world_pos.as_vec2()).normalize_or_zero();
                    let right_vec =
                        (right_mv.fpos.truncate() - world_pos.as_vec2()).normalize_or_zero();
                    let left_score = left_vec.dot(anchor_vec);
                    let right_score = right_vec.dot(anchor_vec);
                    let pos = if left_score < right_score {
                        right_ix as usize
                    } else {
                        closest_ix as usize
                    };
                    pg.insert_point(pos, pc.id(), world_pos)
                }
            }
        }
    });
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
            if let Some(mut commands) = commands.get_entity(eid) {
                commands.insert(EHovered);
            }
        } else {
            if let Some(mut commands) = commands.get_entity(eid) {
                commands.remove::<EHovered>();
            }
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

pub(super) fn cleanup_points(
    mut commands: Commands,
    mut groups_q: Query<(Entity, &mut EPointGroup)>,
    points_q: Query<(Entity, &IntMoveable), With<EPoint>>,
) {
    for (eid, mut group) in groups_q.iter_mut() {
        let mut new_eids = vec![];
        let mut new_poses = vec![];
        for pid in group.eids.iter() {
            if let Ok((eid, mv)) = points_q.get(*pid) {
                new_eids.push(eid);
                new_poses.push(mv.get_ipos().truncate());
            }
        }
        group.eids = new_eids;
        group.poses = new_poses;

        if (group.eids.len() as u32) < group.minimum {
            commands.entity(eid).despawn_recursive();
        }
    }
}
