use bevy::prelude::*;

use crate::{
    drawing::animation::{AnimationManager, SpriteInfo},
    physics::dyno::IntMoveable,
};

use super::{epoint::EPointBundle, transitions::ERootEid};

#[derive(Component, Debug, Clone)]
pub struct EGoal;

#[derive(Bundle)]
struct EGoalBundle {
    estart: EGoal,
    point: EPointBundle,
}
impl EGoalBundle {
    fn new(world_pos: IVec2) -> Self {
        let mut point_bund = EPointBundle::new(world_pos);
        point_bund.name = Name::new("goal");
        point_bund.multi.map.insert(
            "start".into(),
            AnimationManager::single_repeating(
                SpriteInfo {
                    path: "sprites/start_goal/goal18.png".into(),
                    size: UVec2::new(18, 18),
                    ..default()
                },
                10,
            )
            .force_offset(-IVec3::Z),
        );
        Self {
            estart: EGoal,
            point: point_bund,
        }
    }
}

pub(super) fn spawn_goal(
    In(world_pos): In<IVec2>,
    eroot: Res<ERootEid>,
    mut commands: Commands,
    mut existing: Query<&mut IntMoveable, With<EGoal>>,
) {
    if let Ok(mut mv) = existing.get_single_mut() {
        mv.fpos.x = world_pos.x as f32;
        mv.fpos.y = world_pos.y as f32;
        return;
    }
    commands.entity(eroot.0).with_children(|parent| {
        parent.spawn(EGoalBundle::new(world_pos));
    });
}
