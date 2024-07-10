use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    drawing::animation::{AnimationManager, SpriteInfo},
    meta::game_state::{EditingMode, GameState},
    uid::{fresh_uid, UId, UIdMarker},
};

use super::{oneshots::EditorOneshots, point::EPoint, save::SaveMarker};

#[derive(Component, Debug, Clone, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct EStandaloneField {
    pub field_points: Vec<UId>,
    pub dir: Vec2,
}

#[derive(Event)]
pub struct CreateStandaloneFieldEvent(pub Vec2);

#[derive(Bundle)]
pub struct EStandaloneFieldBundle {
    pub name: Name,
    pub uid: UIdMarker,
    pub standalone: EStandaloneField,
    pub spatial: SpatialBundle,
    pub anim: AnimationManager,
    pub save: SaveMarker,
}
impl EStandaloneFieldBundle {
    pub fn from_standalone_field(standalone: EStandaloneField, locations: Vec<IVec2>) -> Self {
        Self {
            name: Name::new("standalone_field"),
            uid: UIdMarker(fresh_uid()),
            anim: AnimationManager::single_repeating(
                SpriteInfo {
                    path: "sprites/field/field_dyno.png".to_string(),
                    size: UVec2::new(8, 8),
                    ..default()
                },
                8,
            )
            .force_mat_rot(standalone.dir.to_angle())
            .force_points(locations)
            .force_ephemeral(),
            standalone,
            spatial: SpatialBundle::default(),
            save: SaveMarker,
        }
    }
}

pub(super) fn create_new_standalone_field(
    mut events: EventReader<CreateStandaloneFieldEvent>,
    gs: Res<GameState>,
    points_q: Query<(Entity, &EPoint)>,
    oneshots: Res<EditorOneshots>,
    mut commands: Commands,
) {
    let Some(CreateStandaloneFieldEvent(dir)) = events.read().last() else {
        return;
    };
    let Some(EditingMode::Free) = gs.get_editing_mode() else {
        return;
    };
    let eids = points_q
        .iter()
        .filter(|p| p.1.is_selected)
        .map(|p| p.0)
        .collect::<Vec<_>>();
    commands.run_system_with_input(oneshots.spawn_standalone_field, (eids, *dir));
}
