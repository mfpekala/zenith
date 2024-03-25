use bevy::prelude::*;

use crate::physics::dyno::IntMoveable;

pub(super) struct EPlanetField {
    field_points: Vec<Entity>,
    dir: Vec2,
}

#[derive(Component, Default)]
pub(super) struct EPlanet {
    pub rock_points: Vec<Entity>,
    pub fields: Vec<EPlanetField>,
}

#[derive(Bundle)]
pub(super) struct EPlanetBundle {
    eplanet: EPlanet,
    spatial: SpatialBundle,
    moveable: IntMoveable,
}
impl EPlanetBundle {
    pub fn spawn(commands: &mut Commands, pos: IVec2) -> Entity {
        let entity = commands.spawn(EPlanetBundle {
            eplanet: EPlanet::default(),
            spatial: SpatialBundle::from_transform(Transform::from_translation(
                pos.as_vec2().extend(0.0),
            )),
            moveable: IntMoveable::new(pos.extend(0)),
        });
        entity.id()
    }
}
