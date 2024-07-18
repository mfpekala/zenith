use bevy::prelude::*;

use crate::{
    drawing::animation::{AnimationManager, SpriteInfo},
    math::ifield_norm,
    meta::game_state::{EditingMode, GameState, SetMetaState},
    physics::dyno::IntMoveable,
};

use super::{
    epoint::{EPoint, EPointGroup, ESelected},
    transitions::ERootEid,
};

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
pub(super) struct EFieldBundle {
    name: Name,
    efield: EField,
    point_group: EPointGroup,
    spatial: SpatialBundle,
    anim: AnimationManager,
}
impl EFieldBundle {
    pub(super) fn new(dir: Vec2, point_group: EPointGroup) -> Self {
        let anim = AnimationManager::single_repeating(
            SpriteInfo {
                path: "sprites/field/field_dyno.png".to_string(),
                size: UVec2::new(8, 8),
                ..default()
            },
            8,
        )
        .force_points(vec![]);
        EFieldBundle {
            name: Name::new("field"),
            efield: EField::new(dir),
            point_group,
            spatial: SpatialBundle {
                transform: Transform::from_translation(-Vec3::Z * 3.0),
                ..default()
            },
            anim,
        }
    }
}

pub(super) fn spawn_field(
    In(()): In<()>,
    mut commands: Commands,
    mut meta_writer: EventWriter<SetMetaState>,
    eroot: Res<ERootEid>,
) {
    let mut eid = Entity::PLACEHOLDER;
    commands.entity(eroot.0).with_children(|parent| {
        eid = parent
            .spawn(EFieldBundle::new(Vec2::ZERO, EPointGroup::default()))
            .id();
    });
    meta_writer.send(SetMetaState(
        EditingMode::CreatingField(eid).to_meta_state(),
    ));
}

pub(super) fn update_fields(
    gs: Res<GameState>,
    mut fields_q: Query<(Entity, &mut EField, &mut EPointGroup)>,
    selected_q: Query<&IntMoveable, (With<EPoint>, With<ESelected>)>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    // Update the minimum number of points in the point group for all rocks not being created
    // NOTE: The "needs_init" hack is a lil nasty but needed bc the SetMetaEvent may not
    //       actually update the gs until one tick _after_ the rock is created
    let creating_eid = gs.get_editing_mode().map(|emode| match emode {
        EditingMode::CreatingField(eid) => eid,
        _ => Entity::PLACEHOLDER,
    });
    for mut field_data in &mut fields_q {
        if Some(field_data.0) == creating_eid {
            field_data.1.needs_init = false;
            if field_data.2.poses.len() == 2 {
                // When creating the field, make it's dir according to the first two points you add
                field_data.1.dir = ifield_norm(field_data.2.poses[0], field_data.2.poses[1]);
            }
        } else {
            if !field_data.1.needs_init {
                field_data.2.minimum = 3;
            }
        }
    }
    // When editing a field, you can select two points (clockwise order) then press g to reset
    // the gravity of that field according to those two points
    if keyboard.just_pressed(KeyCode::KeyG) {
        if let Some(EditingMode::EditingField(eid)) = gs.get_editing_mode() {
            let mut field_data = fields_q.get_mut(eid).unwrap();
            if selected_q.iter().count() == 2 {
                let mut iter = selected_q.iter();
                let p1 = iter.next().unwrap().get_ipos().truncate();
                let p2 = iter.next().unwrap().get_ipos().truncate();
                field_data.1.dir = ifield_norm(p1, p2);
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
