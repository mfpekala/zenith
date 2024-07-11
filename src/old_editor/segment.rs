use std::cmp::Ordering;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    drawing::animation::AnimationManager,
    environment::segment::SegmentKind,
    math::irect,
    meta::game_state::{EditingMode, GameState},
    uid::{UId, UIdMarker, UIdTranslator},
};

use super::{
    help::HelpBarEvent,
    point::{EPoint, EPointKind},
    save::SaveMarker,
};

#[derive(Component, Clone, Debug, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct SegmentParents {
    pub left_uid: UId,
    pub right_uid: UId,
}

#[derive(Bundle)]
struct ESegmentBundle {
    name: Name,
    kind: SegmentKind,
    anim: AnimationManager,
    parents: SegmentParents,
    spatial: SpatialBundle,
    save: SaveMarker,
}

/// Watches for keys to create a segment between two points
pub(super) fn create_segment(
    points: Query<(Entity, &EPoint, &UIdMarker)>,
    gs: Res<GameState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    ut: Res<UIdTranslator>,
    mut help_writer: EventWriter<HelpBarEvent>,
    existing_segments: Query<(Entity, &SegmentParents)>,
) {
    // Basic validation
    if !keyboard.any_just_pressed([KeyCode::KeyK, KeyCode::KeyJ]) {
        return;
    }
    let Some(EditingMode::EditingPlanet(planet_id)) = gs.get_editing_mode() else {
        return;
    };
    let selected_ids: Vec<UId> = points
        .iter()
        .filter(|p| p.1.is_selected)
        .map(|p| p.2 .0)
        .collect();
    if selected_ids.len() != 2 {
        help_writer.send(HelpBarEvent(format!(
            "Tried to add segment with {:?} points selected",
            selected_ids.len()
        )));
        return;
    }

    // Order the points
    let mut ordered_uids = vec![];
    for uid in selected_ids.iter() {
        let Some(eid) = ut.get_entity(*uid) else {
            help_writer.send(HelpBarEvent(format!(
                "Couldn't translate to point eid's while adding segment",
            )));
            return;
        };
        let Ok((_, epoint, _)) = points.get(eid) else {
            help_writer.send(HelpBarEvent(format!(
                "Cound't get point entity from eid while adding segment",
            )));
            return;
        };
        if epoint.kind != EPointKind::Rock {
            help_writer.send(HelpBarEvent(format!(
                "Tried to make a segment using a non-rock point",
            )));
            return;
        }
        ordered_uids.push((*uid, epoint.selection_order.unwrap_or(0)));
    }
    ordered_uids.sort_by(|a, b| {
        if a.1 < b.1 {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    });
    let ordered_uids: Vec<UId> = ordered_uids.into_iter().map(|pair| pair.0).collect();

    // See if we're toggling off (any segments already exist with these two points)
    let mut toggling_off = false;
    for (eid, segment) in existing_segments.iter() {
        if segment.left_uid == ordered_uids[0] && segment.right_uid == ordered_uids[1] {
            commands.entity(eid).despawn_recursive();
            toggling_off = true;
        }
    }
    if toggling_off {
        return;
    }

    // We're not! Actually spawn the thing
    let kind = if keyboard.just_pressed(KeyCode::KeyJ) {
        SegmentKind::Spring
    } else {
        SegmentKind::Spike
    };
    commands.entity(planet_id).with_children(|parent| {
        parent.spawn(ESegmentBundle {
            name: Name::new(kind.to_string()),
            kind,
            anim: kind.to_animation_manager(),
            parents: SegmentParents {
                left_uid: ordered_uids[0],
                right_uid: ordered_uids[1],
            },
            spatial: SpatialBundle::default(),
            save: SaveMarker,
        });
    });
}

/// Finds any segments who have non-existent parents and despawns them
pub(super) fn kill_segments(
    mut commands: Commands,
    segments: Query<(Entity, &SegmentParents)>,
    points: Query<Entity, With<EPoint>>,
    ut: Res<UIdTranslator>,
) {
    for (eid, parents) in segments.iter() {
        let mut should_die = false;
        for parent in [&parents.left_uid, &parents.right_uid] {
            match ut.get_entity(*parent) {
                Some(peid) => {
                    should_die = should_die || points.get(peid).is_err();
                }
                None => should_die = true,
            }
        }
        if should_die {
            commands.entity(eid).despawn_recursive();
        }
    }
}

/// Positions segments to be in the middle of their parents with the correct rotation
pub(super) fn position_segments(
    mut segments: Query<(&SegmentParents, &mut Transform, &mut AnimationManager), Without<EPoint>>,
    points: Query<&Transform, With<EPoint>>,
    ut: Res<UIdTranslator>,
) {
    for (parents, mut tran, mut anim) in segments.iter_mut() {
        let Some(left_eid) = ut.get_entity(parents.left_uid) else {
            continue;
        };
        let Some(right_eid) = ut.get_entity(parents.right_uid) else {
            continue;
        };
        let Ok(left_tran) = points.get(left_eid) else {
            continue;
        };
        let Ok(right_tran) = points.get(right_eid) else {
            continue;
        };
        let current_node = anim.current_node();
        let diff = (right_tran.translation - left_tran.translation).truncate();
        let diff_norm = diff.normalize_or_zero();
        let mut center = (left_tran.translation + right_tran.translation) / 2.0;
        let norm = Vec2::new(-diff_norm.y, diff_norm.x) * 0.9;
        center += (current_node.sprite.size.y as f32 * norm / 2.0).extend(0.0);
        tran.translation = center;
        let angle = diff_norm.y.atan2(diff_norm.x);
        anim.set_tran_angle(angle);
        anim.set_points(irect(
            (diff.length() as u32 / current_node.sprite.size.x) * current_node.sprite.size.x,
            current_node.sprite.size.y,
        ));
    }
}
