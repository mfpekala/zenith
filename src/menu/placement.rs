//! The point of these components is to allow me to think of placement and layout
//! in terms of the game screen size (320 x 180) but have it auto-scale to the
//! large rendering canvas

use bevy::prelude::*;

use crate::meta::consts::MENU_GROWTH;

#[derive(Component)]
pub struct GameRelativePlacement {
    pub pos: IVec3,
    pub scale: f32,
}
impl GameRelativePlacement {
    pub fn new(pos: IVec3, scale: f32) -> Self {
        Self { pos, scale }
    }
}

#[derive(Bundle)]
pub struct GameRelativePlacementBundle {
    game_relative: GameRelativePlacement,
    spatial: SpatialBundle,
}
impl GameRelativePlacementBundle {
    pub(super) fn new(pos: IVec3, scale: f32) -> Self {
        let spatial = SpatialBundle::from_transform(Transform {
            translation: pos.as_vec3() * MENU_GROWTH as f32,
            scale: Vec3::new(scale * MENU_GROWTH as f32, scale * MENU_GROWTH as f32, 1.0),
            ..default()
        });
        Self {
            spatial,
            game_relative: GameRelativePlacement::new(pos, scale),
        }
    }
}

pub(super) fn update_game_relative_placements(
    mut grs: Query<(&mut Transform, &GameRelativePlacement)>,
) {
    for (mut tran, placement) in grs.iter_mut() {
        tran.translation = placement.pos.as_vec3() * MENU_GROWTH as f32;
        tran.scale.x = placement.scale * MENU_GROWTH as f32;
        tran.scale.y = placement.scale * MENU_GROWTH as f32;
    }
}
