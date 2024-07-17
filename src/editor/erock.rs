use bevy::prelude::*;

use crate::{
    drawing::{
        animation::{MultiAnimationManager, SpriteInfo},
        mesh::ioutline_points,
    },
    environment::rock::RockKind,
    meta::game_state::{EditingMode, SetMetaState},
};

use super::{epoint::EPointGroup, transitions::ERootEid};

#[derive(Component, Debug, Clone)]
pub struct ERock {
    pub kind: RockKind,
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
                erock: ERock { kind },
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

pub(super) fn animate_rocks(mut rocks_q: Query<(&mut MultiAnimationManager, &EPointGroup)>) {
    for (mut multi, pg) in rocks_q.iter_mut() {
        let inner = multi.map.get_mut("inner").unwrap();
        inner.set_points(ioutline_points(&pg.poses, -6.0));
        let outer = multi.map.get_mut("outer").unwrap();
        outer.set_points(pg.poses.clone());
    }
}
