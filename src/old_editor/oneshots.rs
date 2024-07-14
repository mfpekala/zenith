use bevy::{ecs::system::SystemId, prelude::*};

use crate::{physics::dyno::IntMoveable, uid::UIdMarker};

use super::{
    field::{EStandaloneField, EStandaloneFieldBundle},
    point::{EPoint, EPointKind},
    EditingSceneRoot,
};

#[derive(Resource)]
pub struct EditorOneshots {
    /// For spawning a field with (points, dir)
    pub spawn_standalone_field: SystemId<(Vec<Entity>, Vec2)>,
}

fn spawn_standalone_field(
    In((points, dir)): In<(Vec<Entity>, Vec2)>,
    points_q: Query<(&EPoint, &IntMoveable)>,
    mut commands: Commands,
    eroot: Query<Entity, With<EditingSceneRoot>>,
) {
    let Ok(eroot) = eroot.get_single() else {
        warn!("Hmm can't spawn a standalone field without an eroot. What the fuck happened?");
        return;
    };
    if points.len() <= 2 {
        warn!("Must have more than two points to make a field");
        return;
    }

    let mut selection_orders = vec![];
    let mut standalone = EStandaloneField {
        field_points: vec![],
        dir,
    };
    let mut locations = vec![];

    for eid in points.iter() {
        let Ok((epoint, mv)) = points_q.get(*eid) else {
            warn!("Found non-existent eid when spawning standalone field. Uh oh.");
            return;
        };
        let ix = match selection_orders.binary_search(&epoint.selection_order.unwrap_or_default()) {
            Ok(ix) | Err(ix) => ix,
        };
        selection_orders.insert(ix, epoint.selection_order.unwrap_or_default());
        locations.insert(ix, mv.fpos.truncate());
        standalone.field_points.insert(ix, *eid);
    }

    let bund = EStandaloneFieldBundle::from_standalone_field(standalone, locations);
    commands.entity(eroot).with_children(|parent| {
        parent.spawn(bund);
    });
}

pub(super) fn register_oneshots(app: &mut App) {
    let oneshots = EditorOneshots {
        spawn_standalone_field: app.world.register_system(spawn_standalone_field),
    };
    app.insert_resource(oneshots);
}
