use bevy::{prelude::*, render::view::RenderLayers};
use serde::{Deserialize, Serialize};

use crate::{
    drawing::{
        animation::{AnimationManager, SpriteInfo},
        layering::sprite_layer,
    },
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
pub struct SegmentBundle {
    pub name: Name,
    pub parents: SegmentParents,
    pub animation: AnimationManager,
    pub render_layers: RenderLayers,
    pub spatial: SpatialBundle,
    pub save: SaveMarker,
}

/// Watches for keys to create a segment between two points
pub(super) fn create_segment(
    points: Query<(Entity, &EPoint, &UIdMarker)>,
    gs: Res<GameState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    ut: Res<UIdTranslator>,
    mut help_writer: EventWriter<HelpBarEvent>,
) {
    if !keyboard.any_just_pressed([KeyCode::KeyK]) {
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
    // TODO: way to order selection
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
        ordered_uids.push(*uid);
    }

    commands.entity(planet_id).with_children(|parent| {
        // Commands
        parent.spawn(SegmentBundle {
            name: Name::new("Segment"),
            parents: SegmentParents {
                left_uid: ordered_uids[0],
                right_uid: ordered_uids[1],
            },
            animation: AnimationManager::single_static(SpriteInfo {
                path: "sprites/goodies/spike.png".to_string(),
                size: UVec2::new(5, 5),
            }),
            render_layers: sprite_layer(),
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
    mut segments: Query<(&SegmentParents, &mut Transform), Without<EPoint>>,
    points: Query<&Transform, With<EPoint>>,
    ut: Res<UIdTranslator>,
) {
    for (parents, mut tran) in segments.iter_mut() {
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
        let center = (left_tran.translation + right_tran.translation) / 2.0;
        tran.translation = center;
        let diff = (right_tran.translation - left_tran.translation)
            .truncate()
            .normalize_or_zero();
        tran.rotation = Quat::from_rotation_arc_2d(Vec2::X, diff);
    }
}
