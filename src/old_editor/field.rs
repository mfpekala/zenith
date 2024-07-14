use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    drawing::animation::{AnimationManager, SpriteInfo},
    meta::game_state::{EditingMode, GameState},
    physics::dyno::IntMoveable,
};

use super::{oneshots::EditorOneshots, point::EPoint, save::SaveMarker};

#[derive(Component, Debug, Clone, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct EStandaloneField {
    pub field_points: Vec<Entity>,
    pub dir: Vec2,
}

#[derive(Event)]
pub struct CreateStandaloneFieldEvent(pub Vec2);

#[derive(Bundle)]
pub struct EStandaloneFieldBundle {
    pub name: Name,
    pub standalone: EStandaloneField,
    pub spatial: SpatialBundle,
    pub anim: AnimationManager,
    pub save: SaveMarker,
}
impl EStandaloneFieldBundle {
    pub fn from_standalone_field(standalone: EStandaloneField, locations: Vec<IVec2>) -> Self {
        Self {
            name: Name::new("standalone_field"),
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

pub(super) fn update_standalone_fields(
    mut esf_q: Query<(Entity, &mut EStandaloneField, &mut AnimationManager)>,
    points_q: Query<(Entity, &EPoint, &IntMoveable)>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
) {
    for (eid, mut esf, mut anim) in esf_q.iter_mut() {
        let alive_points = esf
            .field_points
            .iter()
            .map(|pid| points_q.get(*pid))
            .filter_map(Result::ok)
            .collect::<Vec<_>>();
        if alive_points.len() <= 2 {
            commands.entity(eid).despawn_recursive();
            continue;
        }
        let any_selected = alive_points.iter().any(|(_, p, _)| p.is_selected);
        if any_selected && keyboard.just_pressed(KeyCode::KeyC) {
            commands.entity(eid).despawn_recursive();
        }
        let mut new_field_points = vec![];
        let mut point_poses = vec![];
        for (pid, _, mv) in alive_points.into_iter() {
            new_field_points.push(pid);
            point_poses.push(mv.fpos.truncate());
        }
        esf.field_points = new_field_points;
        anim.set_points(point_poses);
    }
}
