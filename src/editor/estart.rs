use bevy::prelude::*;

use crate::{
    drawing::animation::{AnimationManager, SpriteInfo},
    physics::dyno::IntMoveable,
};

use super::{epoint::EPointBundle, transitions::ERootEid};

#[derive(Component, Debug, Clone)]
pub struct EStart;

#[derive(Bundle)]
pub(super) struct EStartBundle {
    estart: EStart,
    point: EPointBundle,
}
impl EStartBundle {
    pub(super) fn new(world_pos: IVec2) -> Self {
        let mut point_bund = EPointBundle::new(world_pos);
        point_bund.name = Name::new("start");
        point_bund.multi.map.insert(
            "start".into(),
            AnimationManager::single_repeating(
                SpriteInfo {
                    path: "sprites/start_goal/start18.png".into(),
                    size: UVec2::new(18, 18),
                    ..default()
                },
                10,
            )
            .force_offset(-IVec3::Z),
        );
        Self {
            estart: EStart,
            point: point_bund,
        }
    }
}

pub(super) fn spawn_start(
    In(world_pos): In<IVec2>,
    eroot: Res<ERootEid>,
    mut commands: Commands,
    mut existing: Query<&mut IntMoveable, With<EStart>>,
) {
    if let Ok(mut mv) = existing.get_single_mut() {
        mv.fpos.x = world_pos.x as f32;
        mv.fpos.y = world_pos.y as f32;
        return;
    }
    commands.entity(eroot.0).with_children(|parent| {
        parent.spawn(EStartBundle::new(world_pos));
    });
}
