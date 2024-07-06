use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    drawing::animation::{AnimationManager, AnimationNode, SpriteInfo},
    math::irect,
    meta::level_data::{ExportedSegment, Rehydrate},
    physics::collider::{ColliderTriggerStub, ColliderTriggerStubs},
    uid::fresh_uid,
};

#[derive(Component)]
pub struct SpikeMarker;

#[derive(Component)]
pub struct SpringMarker;

#[derive(Component, Clone, Copy, Debug, Default, Reflect, Serialize, Deserialize, PartialEq)]
#[reflect(Component, Serialize, Deserialize)]
pub enum SegmentKind {
    #[default]
    Spike,
    Spring,
}
impl std::fmt::Display for SegmentKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let txt = match *self {
            Self::Spike => "Spike",
            Self::Spring => "Spring",
        };
        write!(f, "{}", txt)
    }
}
impl SegmentKind {
    pub fn collider_height(&self) -> f32 {
        match *self {
            Self::Spike => 7.0,
            Self::Spring => 1.0,
        }
    }

    pub fn to_animation_manager(&self) -> AnimationManager {
        match *self {
            Self::Spike => {
                // Don't do it rust...
                AnimationManager::single_static(SpriteInfo {
                    path: "sprites/goodies/spike.png".to_string(),
                    size: UVec2::new(7, 7),
                    ..default()
                })
            }
            Self::Spring => AnimationManager::from_nodes(vec![
                (
                    "idle",
                    AnimationNode {
                        sprite: SpriteInfo {
                            path: "sprites/goodies/spring.png".to_string(),
                            size: UVec2::new(7, 7),
                            ..default()
                        },
                        length: 1,
                        ..default()
                    },
                ),
                (
                    "bounce",
                    AnimationNode {
                        sprite: SpriteInfo {
                            path: "sprites/goodies/spring_bounce.png".to_string(),
                            size: UVec2::new(7, 7),
                            ..default()
                        },
                        length: 6,
                        next: Some("idle".to_string()),
                        ..default()
                    },
                ),
            ]),
        }
    }
}

#[derive(Component, Debug)]
pub struct Segment {
    pub kind: SegmentKind,
    pub left_parent: IVec2,
    pub right_parent: IVec2,
}

#[derive(Bundle)]
pub struct SegmentBundle {
    pub name: Name,
    pub spatial: SpatialBundle,
    pub anim: AnimationManager,
    pub segment: Segment,
    pub triggers: ColliderTriggerStubs,
}

impl Rehydrate<SegmentBundle> for ExportedSegment {
    fn rehydrate(self) -> SegmentBundle {
        let diff = (self.right_parent - self.left_parent).as_vec2();
        let diff_norm = diff.normalize_or_zero();
        let mut center = (self.left_parent + self.right_parent).as_vec2() / 2.0;
        let norm = Vec2::new(-diff_norm.y, diff_norm.x) * 0.9;
        let mut anim = self.kind.to_animation_manager();
        let current_node = anim.current_node();
        center += current_node.sprite.size.y as f32 * norm / 2.0;
        let angle = diff_norm.y.atan2(diff_norm.x);

        let spatial =
            SpatialBundle::from_transform(Transform::from_translation(center.extend(0.0)));

        let segment = Segment {
            kind: self.kind,
            left_parent: self.left_parent,
            right_parent: self.right_parent,
        };

        let anim_points = irect(
            (diff.length() as u32 / current_node.sprite.size.x) * current_node.sprite.size.x,
            current_node.sprite.size.y,
        );
        let trigger_points = irect(
            (diff.length() as u32 / current_node.sprite.size.x) * current_node.sprite.size.x,
            self.kind.collider_height() as u32,
        );
        let trigger_points = trigger_points
            .into_iter()
            .map(|p| {
                let base = Vec2::new(angle.cos(), angle.sin());
                let p = center + base.rotate(p.as_vec2());
                IVec2::new(p.x.round() as i32, p.y.round() as i32)
            })
            .collect();
        anim.set_tran_angle(angle);
        anim.set_points(anim_points);
        let trigger = ColliderTriggerStub {
            uid: fresh_uid(),
            refresh_period: 0,
            points: trigger_points,
            active: true,
        };

        SegmentBundle {
            name: Name::new(self.kind.to_string()),
            spatial,
            anim,
            segment,
            triggers: ColliderTriggerStubs(vec![trigger]),
        }
    }
}
