use bevy::prelude::*;

use crate::drawing::{
    animation::{AnimationManager, SpriteInfo},
    layering::light_layer_u8,
};

use super::{epoint::EPointBundle, transitions::ERootEid};

#[derive(Component, Debug, Clone)]
pub struct EReplenish;

#[derive(Bundle)]
pub(super) struct EReplenishBundle {
    ereplenish: EReplenish,
    point: EPointBundle,
}
impl EReplenishBundle {
    pub(super) fn new(world_pos: IVec2) -> Self {
        let mut point_bund = EPointBundle::new(world_pos);
        point_bund.name = Name::new("replenish");
        point_bund.multi.map.insert(
            "replenish".into(),
            AnimationManager::single_static(SpriteInfo {
                path: "sprites/replenish.png".into(),
                size: UVec2::new(12, 12),
                ..default()
            })
            .force_offset(-IVec3::Z),
        );
        point_bund.multi.map.insert(
            "replenish_light".into(),
            AnimationManager::single_repeating(
                SpriteInfo {
                    path: "sprites/replenishL.png".into(),
                    size: UVec2::new(16, 16),
                    ..default()
                },
                3,
            )
            .force_offset(-IVec3::Z)
            .force_render_layer(light_layer_u8()),
        );
        Self {
            ereplenish: EReplenish,
            point: point_bund,
        }
    }
}

pub(super) fn spawn_replenish(
    In(world_pos): In<IVec2>,
    eroot: Res<ERootEid>,
    mut commands: Commands,
) {
    commands.entity(eroot.0).with_children(|parent| {
        parent.spawn(EReplenishBundle::new(world_pos));
    });
}
