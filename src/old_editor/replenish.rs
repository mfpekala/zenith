use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    drawing::{
        animation::{AnimationManager, MultiAnimationManager, SpriteInfo},
        layering::light_layer_u8,
    },
    input::MouseState,
};

use super::{
    point::{EPointBundle, EPointKind},
    EditingSceneRoot,
};

#[derive(Component, Clone, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct EReplenish;

#[derive(Bundle)]
pub(super) struct EReplenishBundle {
    replenish: EReplenish,
    point: EPointBundle,
}
impl EReplenishBundle {
    pub fn new(pos: IVec2) -> Self {
        let mut bund = EPointBundle::new(IVec2::ZERO, EPointKind::Free(5));
        let replenish = AnimationManager::single_static(SpriteInfo {
            path: "sprites/replenish.png".to_string(),
            size: UVec2::new(12, 12),
            ..default()
        })
        .force_ephemeral();
        let mut replenish_light = AnimationManager::single_repeating(
            SpriteInfo {
                path: "sprites/replenishL.png".to_string(),
                size: UVec2::new(16, 16),
                ..default()
            },
            3,
        )
        .force_ephemeral();
        replenish_light.set_render_layers(vec![light_layer_u8()]);
        bund.anim = MultiAnimationManager::from_pairs(vec![
            ("replenish", replenish),
            ("replenishL", replenish_light),
        ]);
        bund.moveable.pos = pos.extend(0);
        bund.spatial.transform.translation = bund.moveable.pos.as_vec3();
        Self {
            replenish: EReplenish,
            point: bund,
        }
    }
}

pub(super) fn spawn_replenish(
    mut commands: Commands,
    root: Query<Entity, With<EditingSceneRoot>>,
    mouse_state: Res<MouseState>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if !keyboard.just_pressed(KeyCode::KeyR) || keyboard.pressed(KeyCode::SuperLeft) {
        return;
    }
    let Ok(root) = root.get_single() else {
        return;
    };
    commands.entity(root).with_children(|parent| {
        parent.spawn(EReplenishBundle::new(mouse_state.world_pos));
    });
}
