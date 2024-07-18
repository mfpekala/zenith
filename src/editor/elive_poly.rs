use bevy::prelude::*;

use crate::meta::game_state::{EditingMode, GameState, SetMetaState};

use super::{epoint::EPointGroup, transitions::ERootEid};

#[derive(Component, Debug, Clone, Reflect)]
pub struct ELivePoly {
    /// Needed to solve the issue where on the tick where we send event to change game state
    /// we spawn the rock, and then the minimum is set to 2 and it's despawned
    needs_init: bool,
}
impl ELivePoly {
    pub fn new() -> Self {
        Self { needs_init: true }
    }
}

#[derive(Bundle)]
struct ELivePolyBundle {
    name: Name,
    elive_poly: ELivePoly,
    point_group: EPointGroup,
    spatial: SpatialBundle,
}

pub(super) fn spawn_live_poly(
    In(()): In<()>,
    mut commands: Commands,
    mut meta_writer: EventWriter<SetMetaState>,
    eroot: Res<ERootEid>,
) {
    let mut eid = Entity::PLACEHOLDER;
    commands.entity(eroot.0).with_children(|parent| {
        eid = parent
            .spawn(ELivePolyBundle {
                name: Name::new("live_poly"),
                elive_poly: ELivePoly::new(),
                point_group: EPointGroup::default(),
                spatial: SpatialBundle {
                    transform: Transform::from_translation(-Vec3::Z * 3.0),
                    ..default()
                },
            })
            .id();
    });
    meta_writer.send(SetMetaState(
        EditingMode::CreatingLivePoly(eid).to_meta_state(),
    ));
}

pub(super) fn update_live_polys(
    gs: Res<GameState>,
    mut live_polys_q: Query<(Entity, &mut ELivePoly, &mut EPointGroup)>,
) {
    // Update the minimum number of points in the point group for all rocks not being created
    // NOTE: The "needs_init" hack is a lil nasty but needed bc the SetMetaEvent may not
    //       actually update the gs until one tick _after_ the rock is created
    let creating_eid = gs.get_editing_mode().map(|emode| match emode {
        EditingMode::CreatingField(eid) => eid,
        _ => Entity::PLACEHOLDER,
    });
    for mut field_data in &mut live_polys_q {
        if Some(field_data.0) == creating_eid {
            field_data.1.needs_init = false;
        } else {
            if !field_data.1.needs_init {
                field_data.2.minimum = 3;
            }
        }
    }
}

pub(super) fn animate_live_polys(
    mut gzs: Gizmos,
    mut live_polys_q: Query<&EPointGroup, With<ELivePoly>>,
) {
    for pg in live_polys_q.iter_mut() {
        for ix in 0..pg.poses.len() {
            let p1 = pg.poses[ix];
            let p2 = pg.poses[(ix + 1).rem_euclid(pg.poses.len())];
            gzs.line_2d(p1.as_vec2(), p2.as_vec2(), Color::YELLOW);
        }
    }
}
