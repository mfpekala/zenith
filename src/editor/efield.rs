use bevy::prelude::*;

use crate::{
    drawing::animation::{AnimationManager, SpriteInfo},
    meta::game_state::{EditingMode, GameState, SetMetaState},
};

use super::{epoint::EPointGroup, transitions::ERootEid};

#[derive(Component, Debug, Clone, Reflect)]
pub struct EField {
    /// Needed to solve the issue where on the tick where we send event to change game state
    /// we spawn the rock, and then the minimum is set to 2 and it's despawned
    needs_init: bool,
    pub dir: Vec2,
}
impl EField {
    pub fn new(dir: Vec2) -> Self {
        Self {
            needs_init: true,
            dir,
        }
    }
}

#[derive(Bundle)]
struct EFieldBundle {
    name: Name,
    efield: EField,
    point_group: EPointGroup,
    spatial: SpatialBundle,
    anim: AnimationManager,
}

pub(super) fn spawn_field(
    In(()): In<()>,
    mut commands: Commands,
    mut meta_writer: EventWriter<SetMetaState>,
    eroot: Res<ERootEid>,
) {
    let anim = AnimationManager::single_repeating(
        SpriteInfo {
            path: "sprites/field/field_dyno.png".to_string(),
            size: UVec2::new(8, 8),
            ..default()
        },
        8,
    )
    .force_points(vec![]);
    let mut eid = Entity::PLACEHOLDER;
    commands.entity(eroot.0).with_children(|parent| {
        eid = parent
            .spawn(EFieldBundle {
                name: Name::new("field"),
                efield: EField::new(Vec2::ZERO),
                point_group: EPointGroup::default(),
                spatial: SpatialBundle {
                    transform: Transform::from_translation(-Vec3::Z * 3.0),
                    ..default()
                },
                anim,
            })
            .id();
    });
    meta_writer.send(SetMetaState(
        EditingMode::CreatingField(eid).to_meta_state(),
    ));
}

pub(super) fn update_fields(
    gs: Res<GameState>,
    mut fields_q: Query<(Entity, &mut EField, &mut EPointGroup)>,
) {
    // Update the minimum number of points in the point group for all rocks not being created
    // NOTE: The "needs_init" hack is a lil nasty but needed bc the SetMetaEvent may not
    //       actually update the gs until one tick _after_ the rock is created
    let creating_eid = gs.get_editing_mode().map(|emode| match emode {
        EditingMode::Free
        | EditingMode::EditingRock(_)
        | EditingMode::CreatingRock(_)
        | EditingMode::EditingField(_) => Entity::PLACEHOLDER,
        EditingMode::CreatingField(eid) => eid,
    });
    for mut field_data in &mut fields_q {
        if Some(field_data.0) == creating_eid {
            field_data.1.needs_init = false;
            if field_data.2.poses.len() == 2 {
                // When creating the field, make it's dir according to the first two points you add
                let par = field_data.2.poses[1] - field_data.2.poses[0];
                let perp = IVec2::new(-par.y, par.x);
                field_data.1.dir = perp.as_vec2().normalize_or_zero();
            }
        } else {
            if !field_data.1.needs_init {
                field_data.2.minimum = 3;
            }
        }
    }
}

pub(super) fn animate_fields(mut fields_q: Query<(&mut AnimationManager, &EPointGroup, &EField)>) {
    for (mut anim, pg, field) in fields_q.iter_mut() {
        anim.set_points(pg.poses.clone());
        anim.set_mat_rot(field.dir.to_angle());
    }
}
