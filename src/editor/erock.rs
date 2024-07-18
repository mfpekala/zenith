use bevy::prelude::*;

use crate::{
    drawing::{
        animation::{MultiAnimationManager, SpriteInfo},
        mesh::ioutline_points,
    },
    environment::rock::RockKind,
    meta::game_state::{EditingMode, GameState, SetMetaState},
};

use super::{epoint::EPointGroup, transitions::ERootEid};

#[derive(Component, Debug, Clone, Reflect)]
pub struct ERock {
    /// Needed to solve the issue where on the tick where we send event to change game state
    /// we spawn the rock, and then the minimum is set to 2 and it's despawned
    needs_init: bool,
    pub kind: RockKind,
}
impl ERock {
    pub fn new(kind: RockKind) -> Self {
        Self {
            needs_init: true,
            kind,
        }
    }
}

#[derive(Bundle)]
pub struct ERockBundle {
    name: Name,
    erock: ERock,
    point_group: EPointGroup,
    spatial: SpatialBundle,
    multi: MultiAnimationManager,
}

pub(super) fn spawn_rock(
    In(()): In<()>,
    mut commands: Commands,
    mut meta_writer: EventWriter<SetMetaState>,
    eroot: Res<ERootEid>,
) {
    let kind = RockKind::default();
    let (inner, outer) = kind.to_sprite_infos();
    let multi = MultiAnimationManager::bordered_mesh(vec![], inner, outer, 6.0);
    let mut eid = Entity::PLACEHOLDER;
    commands.entity(eroot.0).with_children(|eroot| {
        eid = eroot
            .spawn(ERockBundle {
                name: Name::new("rock"),
                erock: ERock::new(kind),
                point_group: EPointGroup {
                    eids: vec![],
                    poses: vec![],
                    minimum: 0,
                    force_shiny: Some(true),
                },
                spatial: default(),
                multi,
            })
            .id();
    });
    meta_writer.send(SetMetaState(EditingMode::CreatingRock(eid).to_meta_state()));
}

pub(super) fn update_rocks(
    gs: Res<GameState>,
    mut rocks_q: Query<(Entity, &mut ERock, &mut EPointGroup)>,
) {
    // Update the minimum number of points in the point group for all rocks not being created
    // N
    let creating_eid = gs.get_editing_mode().map(|emode| match emode {
        EditingMode::Free | EditingMode::EditingRock(_) => Entity::PLACEHOLDER,
        EditingMode::CreatingRock(eid) => eid,
    });
    for mut rock_data in &mut rocks_q {
        if Some(rock_data.0) == creating_eid {
            rock_data.1.needs_init = false;
        } else {
            if !rock_data.1.needs_init {
                rock_data.2.minimum = 2;
            }
        }
    }
}

pub(super) fn animate_rocks(mut rocks_q: Query<(&mut MultiAnimationManager, &EPointGroup)>) {
    for (mut multi, pg) in rocks_q.iter_mut() {
        let inner = multi.map.get_mut("inner").unwrap();
        inner.set_points(ioutline_points(&pg.poses, -6.0));
        let outer = multi.map.get_mut("outer").unwrap();
        outer.set_points(pg.poses.clone());
    }
}
